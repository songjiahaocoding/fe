use std::{collections::BTreeSet, fs, path::PathBuf};

use globset::{Glob, GlobSet, GlobSetBuilder};
use regex::Regex;
use serde_json::Value;
use walkdir::WalkDir;

use crate::{
    cli::{BatchCommand, BatchFiles},
    edit,
    error::{FormatEditError, Result},
    format,
    path::{JsonPath, PathSegment},
    query,
};

pub struct BatchPlan {
    pub files: Vec<FileChange>,
    pub changes: usize,
}

pub struct FileChange {
    pub path: PathBuf,
    pub before: String,
    pub after: String,
    pub changes: usize,
}

pub fn build(command: BatchCommand) -> Result<BatchPlan> {
    match command {
        BatchCommand::Set {
            files,
            path,
            value,
            value_file,
            raw,
        } => {
            let value = format::parse_input_value(value, value_file.as_deref(), raw)?;
            build_for_files(files, move |document| batch_set(document, &path, &value))
        }
        BatchCommand::Put {
            files,
            path,
            key,
            value,
            value_file,
            raw,
            overwrite,
            if_missing,
        } => {
            let value = format::parse_input_value(value, value_file.as_deref(), raw)?;
            build_for_files(files, move |document| {
                batch_put(document, &path, &key, &value, overwrite, if_missing)
            })
        }
        BatchCommand::Delete {
            files,
            path,
            key,
            key_regex,
        } => {
            let key_regex = key_regex
                .map(|value| {
                    Regex::new(&value).map_err(|err| FormatEditError::InvalidRegex(err.to_string()))
                })
                .transpose()?;
            build_for_files(files, move |document| {
                batch_delete(document, &path, key.as_deref(), key_regex.as_ref())
            })
        }
        BatchCommand::Replace {
            files,
            path,
            pattern,
            replacement,
        } => {
            let regex = Regex::new(&pattern)
                .map_err(|err| FormatEditError::InvalidRegex(err.to_string()))?;
            build_for_files(files, move |document| {
                batch_replace(document, &path, &regex, &replacement)
            })
        }
        BatchCommand::Append {
            files,
            path,
            value,
            value_file,
            raw,
        } => {
            let value = format::parse_input_value(value, value_file.as_deref(), raw)?;
            build_for_files(files, move |document| batch_append(document, &path, &value))
        }
    }
}

fn build_for_files(
    selection: BatchFiles,
    mut apply: impl FnMut(&mut Value) -> Result<usize>,
) -> Result<BatchPlan> {
    let paths = discover_files(&selection)?;
    let mut files = Vec::new();
    let mut total = 0;
    for path in paths {
        let document_format = format::parse_format(&path, selection.format)?;
        let before = fs::read_to_string(&path)?;
        let mut document = format::load_document(&path, document_format)?;
        let changes = apply(&mut document)?;
        if changes == 0 {
            continue;
        }
        let after = format::serialize(document_format, &document)?;
        if before != after {
            total += changes;
            files.push(FileChange {
                path,
                before,
                after,
                changes,
            });
        }
    }
    Ok(BatchPlan {
        files,
        changes: total,
    })
}

pub fn apply(plan: &BatchPlan) -> Result<()> {
    for file in &plan.files {
        fs::write(&file.path, &file.after)?;
    }
    Ok(())
}

pub fn print_preview(plan: &BatchPlan) {
    for file in &plan.files {
        let name = file.path.display().to_string();
        let diff = similar::TextDiff::from_lines(&file.before, &file.after)
            .unified_diff()
            .context_radius(3)
            .header(&name, &name)
            .to_string();
        print!("{diff}");
    }
    println!(
        "Summary: {} file(s) changed · {} structured change(s) · no files written",
        plan.files.len(),
        plan.changes
    );
}

fn discover_files(selection: &BatchFiles) -> Result<Vec<PathBuf>> {
    let mut paths = BTreeSet::new();
    paths.extend(selection.files.iter().cloned());
    if let Some(root) = &selection.root {
        let default_includes;
        let include_patterns = if selection.include.is_empty() {
            default_includes = vec![
                "**/*.json".to_string(),
                "**/*.yaml".to_string(),
                "**/*.yml".to_string(),
            ];
            &default_includes
        } else {
            &selection.include
        };
        let includes = build_globs(include_patterns)?;
        let excludes = build_globs(&selection.exclude)?;
        for entry in WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_map(|entry| entry.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let relative = entry.path().strip_prefix(root).unwrap_or(entry.path());
            if includes.is_match(relative) && !excludes.is_match(relative) {
                paths.insert(entry.path().to_path_buf());
            }
        }
    }
    if paths.is_empty() {
        return Err(FormatEditError::BatchNoFiles);
    }
    Ok(paths.into_iter().collect())
}

fn build_globs(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let glob =
            Glob::new(pattern).map_err(|err| FormatEditError::InvalidRegex(err.to_string()))?;
        builder.add(glob);
    }
    builder
        .build()
        .map_err(|err| FormatEditError::InvalidRegex(err.to_string()))
}

fn matches(document: &Value, raw_path: &str) -> Result<Vec<JsonPath>> {
    let path = raw_path.parse::<JsonPath>()?;
    let mut output = Vec::new();
    collect_matches(document, path.segments(), Vec::new(), &mut output);
    if output.is_empty() {
        return Err(FormatEditError::PathNotFound(raw_path.to_string()));
    }
    Ok(output.into_iter().map(JsonPath::from_segments).collect())
}

fn collect_matches(
    value: &Value,
    segments: &[PathSegment],
    prefix: Vec<PathSegment>,
    output: &mut Vec<Vec<PathSegment>>,
) {
    if segments.is_empty() {
        output.push(prefix);
        return;
    }
    match &segments[0] {
        PathSegment::Key(key) => {
            if let Value::Object(object) = value
                && let Some(child) = object.get(key)
            {
                let mut next = prefix;
                next.push(PathSegment::Key(key.clone()));
                collect_matches(child, &segments[1..], next, output);
            }
        }
        PathSegment::Index(index) => {
            if let Value::Array(array) = value {
                let resolved = if *index < 0 {
                    array.len() as i64 + index
                } else {
                    *index
                };
                if resolved >= 0 && resolved < array.len() as i64 {
                    let mut next = prefix;
                    next.push(PathSegment::Index(resolved));
                    collect_matches(&array[resolved as usize], &segments[1..], next, output);
                }
            }
        }
        PathSegment::Wildcard => match value {
            Value::Object(object) => {
                for (key, child) in object {
                    let mut next = prefix.clone();
                    next.push(PathSegment::Key(key.clone()));
                    collect_matches(child, &segments[1..], next, output);
                }
            }
            Value::Array(array) => {
                for (index, child) in array.iter().enumerate() {
                    let mut next = prefix.clone();
                    next.push(PathSegment::Index(index as i64));
                    collect_matches(child, &segments[1..], next, output);
                }
            }
            _ => {}
        },
    }
}

fn batch_set(document: &mut Value, path: &str, value: &Value) -> Result<usize> {
    let paths = matches(document, path)?;
    for path in &paths {
        edit::set(document, path, value.clone(), false)?;
    }
    Ok(paths.len())
}

fn batch_put(
    document: &mut Value,
    scope: &str,
    key: &str,
    value: &Value,
    overwrite: bool,
    if_missing: bool,
) -> Result<usize> {
    let scopes = matches(document, scope)?;
    let mut targets = Vec::new();
    for scope in scopes {
        let object = query::get(document, &scope)[0]
            .as_object()
            .ok_or_else(|| FormatEditError::BatchTargetNotObject(scope.to_string()))?;
        if object.contains_key(key) {
            if if_missing {
                continue;
            }
            if !overwrite {
                return Err(FormatEditError::KeyAlreadyExists(format!(
                    "{}.{}",
                    scope, key
                )));
            }
        }
        let mut segments = scope.segments().to_vec();
        segments.push(PathSegment::Key(key.to_string()));
        targets.push(JsonPath::from_segments(segments));
    }
    for target in &targets {
        edit::set(document, target, value.clone(), true)?;
    }
    Ok(targets.len())
}

fn batch_delete(
    document: &mut Value,
    scope: &str,
    key: Option<&str>,
    key_regex: Option<&Regex>,
) -> Result<usize> {
    let scopes = matches(document, scope)?;
    let mut targets = Vec::new();
    if key.is_none() && key_regex.is_none() {
        targets = scopes;
    } else {
        for scope in scopes {
            let object = query::get(document, &scope)[0]
                .as_object()
                .ok_or_else(|| FormatEditError::BatchTargetNotObject(scope.to_string()))?;
            for candidate in object.keys() {
                let selected = key.is_some_and(|value| value == candidate)
                    || key_regex.is_some_and(|regex| regex.is_match(candidate));
                if selected {
                    let mut segments = scope.segments().to_vec();
                    segments.push(PathSegment::Key(candidate.clone()));
                    targets.push(JsonPath::from_segments(segments));
                }
            }
        }
    }
    // Matches are collected in document order. Reverse deletion keeps array
    // indexes stable when several elements share the same parent.
    targets.reverse();
    for target in &targets {
        edit::delete(document, target)?;
    }
    Ok(targets.len())
}

fn batch_replace(
    document: &mut Value,
    path: &str,
    regex: &Regex,
    replacement: &str,
) -> Result<usize> {
    let paths = matches(document, path)?;
    let mut replacements = Vec::new();
    for path in paths {
        let value = query::get(document, &path)[0]
            .as_str()
            .ok_or_else(|| FormatEditError::BatchTargetNotString(path.to_string()))?;
        let replaced = regex.replace_all(value, replacement).into_owned();
        if replaced != value {
            replacements.push((path, replaced));
        }
    }
    for (path, value) in &replacements {
        edit::set(document, path, Value::String(value.clone()), false)?;
    }
    Ok(replacements.len())
}

fn batch_append(document: &mut Value, path: &str, value: &Value) -> Result<usize> {
    let paths = matches(document, path)?;
    for path in &paths {
        edit::append(document, path, value.clone(), false)?;
    }
    Ok(paths.len())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn sets_all_wildcard_matches() {
        let mut document = json!({"services": [{"enabled": true}, {"enabled": true}]});
        assert_eq!(
            batch_set(&mut document, "$.services[*].enabled", &json!(false)).unwrap(),
            2
        );
        assert_eq!(
            document,
            json!({"services": [{"enabled": false}, {"enabled": false}]})
        );
    }

    #[test]
    fn deletes_object_members_selected_by_key_regex() {
        let mut document = json!({"services": [{"x-old": 1, "keep": 2}, {"x-other": 3}]});
        let regex = Regex::new("^x-").unwrap();
        assert_eq!(
            batch_delete(&mut document, "$.services[*]", None, Some(&regex)).unwrap(),
            2
        );
        assert_eq!(document, json!({"services": [{"keep": 2}, {}]}));
    }

    #[test]
    fn replaces_only_selected_string_values() {
        let mut document = json!({"services": [{"image": "old/api"}, {"image": "old/jobs"}]});
        let regex = Regex::new("^old/").unwrap();
        assert_eq!(
            batch_replace(&mut document, "$.services[*].image", &regex, "new/").unwrap(),
            2
        );
        assert_eq!(
            document,
            json!({"services": [{"image": "new/api"}, {"image": "new/jobs"}]})
        );
    }

    #[test]
    fn puts_values_into_all_selected_objects() {
        let mut document = json!({"services": [{"name": "api"}, {"name": "jobs"}]});
        assert_eq!(
            batch_put(
                &mut document,
                "$.services[*]",
                "timeout",
                &json!(30),
                false,
                false
            )
            .unwrap(),
            2
        );
        assert_eq!(
            document,
            json!({"services": [{"name": "api", "timeout": 30}, {"name": "jobs", "timeout": 30}]})
        );
    }

    #[test]
    fn put_requires_an_explicit_existing_key_policy() {
        let mut document = json!({"services": [{"timeout": 10}]});
        let error = batch_put(
            &mut document,
            "$.services[*]",
            "timeout",
            &json!(30),
            false,
            false,
        )
        .unwrap_err();
        assert!(matches!(error, FormatEditError::KeyAlreadyExists(_)));
    }

    #[test]
    fn appends_to_all_selected_arrays() {
        let mut document = json!({"groups": [{"items": [1]}, {"items": [2]}]});
        assert_eq!(
            batch_append(&mut document, "$.groups[*].items", &json!(3)).unwrap(),
            2
        );
        assert_eq!(
            document,
            json!({"groups": [{"items": [1, 3]}, {"items": [2, 3]}]})
        );
    }

    #[test]
    fn deletes_multiple_array_elements_without_index_shift() {
        let mut document = json!({"items": ["a", "b", "c"]});
        assert_eq!(
            batch_delete(&mut document, "$.items[*]", None, None).unwrap(),
            3
        );
        assert_eq!(document, json!({"items": []}));
    }
}
