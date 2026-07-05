use serde_json::{Map, Value};

use crate::{
    error::{FormatEditError, Result, value_type},
    path::{JsonPath, PathSegment},
};

pub fn set(root: &mut Value, path: &JsonPath, value: Value, create: bool) -> Result<()> {
    reject_wildcard(path)?;
    set_at(root, path.segments(), value, create, "$".to_string())
}

pub fn delete(root: &mut Value, path: &JsonPath) -> Result<()> {
    reject_wildcard(path)?;
    let segments = path.segments();
    if segments.is_empty() {
        *root = Value::Null;
        return Ok(());
    }

    let (parent_segments, final_segment) = segments.split_at(segments.len() - 1);
    let parent = get_mut_at(root, parent_segments, "$".to_string())?;
    let final_segment = &final_segment[0];
    let parent_path = path_for_segments(parent_segments);

    match final_segment {
        PathSegment::Key(key) => match parent {
            Value::Object(object) => object.remove(key).map(|_| ()).ok_or_else(|| {
                FormatEditError::PathNotFound(join_path(&parent_path, final_segment))
            }),
            other => Err(type_mismatch(&parent_path, "object", other)),
        },
        PathSegment::Index(index) => match parent {
            Value::Array(array) => {
                let index = resolve_existing_index(*index, array.len(), &parent_path)?;
                array.remove(index);
                Ok(())
            }
            other => Err(type_mismatch(&parent_path, "array", other)),
        },
        PathSegment::Wildcard => unreachable!("wildcards are rejected above"),
    }
}

pub fn append(root: &mut Value, path: &JsonPath, value: Value, create: bool) -> Result<()> {
    reject_wildcard(path)?;

    let target = match get_mut_at(root, path.segments(), "$".to_string()) {
        Ok(target) => target,
        Err(FormatEditError::PathNotFound(_)) if create => {
            set(root, path, Value::Array(Vec::new()), true)?;
            get_mut_at(root, path.segments(), "$".to_string())?
        }
        Err(err) => return Err(err),
    };

    match target {
        Value::Array(array) => {
            array.push(value);
            Ok(())
        }
        other => Err(type_mismatch(&path.to_string(), "array", other)),
    }
}

pub fn insert(root: &mut Value, path: &JsonPath, value: Value) -> Result<()> {
    reject_wildcard(path)?;
    let segments = path.segments();
    let Some(PathSegment::Index(index)) = segments.last() else {
        return Err(FormatEditError::InsertNeedsIndex);
    };

    let parent_segments = &segments[..segments.len() - 1];
    let parent_path = path_for_segments(parent_segments);
    let parent = get_mut_at(root, parent_segments, "$".to_string())?;

    match parent {
        Value::Array(array) => {
            let index = resolve_insert_index(*index, array.len(), &parent_path)?;
            array.insert(index, value);
            Ok(())
        }
        other => Err(type_mismatch(&parent_path, "array", other)),
    }
}

fn set_at(
    current: &mut Value,
    segments: &[PathSegment],
    value: Value,
    create: bool,
    current_path: String,
) -> Result<()> {
    if segments.is_empty() {
        *current = value;
        return Ok(());
    }

    if segments.len() == 1 {
        return set_final(current, &segments[0], value, create, &current_path);
    }

    let segment = &segments[0];
    let next_segment = &segments[1];
    match segment {
        PathSegment::Key(key) => {
            if current.is_null() && create {
                *current = Value::Object(Map::new());
            }
            match current {
                Value::Object(object) => {
                    if !object.contains_key(key) {
                        if !create {
                            return Err(FormatEditError::PathNotFound(join_path(
                                &current_path,
                                segment,
                            )));
                        }
                        object.insert(key.clone(), container_for(next_segment));
                    }
                    let child = object.get_mut(key).expect("key was just inserted");
                    set_at(
                        child,
                        &segments[1..],
                        value,
                        create,
                        join_path(&current_path, segment),
                    )
                }
                other => Err(type_mismatch(&current_path, "object", other)),
            }
        }
        PathSegment::Index(index) => {
            if current.is_null() && create {
                *current = Value::Array(Vec::new());
            }
            match current {
                Value::Array(array) => {
                    let index =
                        resolve_or_create_index(*index, array.len(), create, &current_path)?;
                    if index == array.len() {
                        array.push(container_for(next_segment));
                    }
                    set_at(
                        &mut array[index],
                        &segments[1..],
                        value,
                        create,
                        join_path(&current_path, segment),
                    )
                }
                other => Err(type_mismatch(&current_path, "array", other)),
            }
        }
        PathSegment::Wildcard => unreachable!("wildcards are rejected above"),
    }
}

fn set_final(
    current: &mut Value,
    segment: &PathSegment,
    value: Value,
    create: bool,
    current_path: &str,
) -> Result<()> {
    match segment {
        PathSegment::Key(key) => {
            if current.is_null() && create {
                *current = Value::Object(Map::new());
            }
            match current {
                Value::Object(object) => {
                    if !create && !object.contains_key(key) {
                        return Err(FormatEditError::PathNotFound(join_path(
                            current_path,
                            segment,
                        )));
                    }
                    object.insert(key.clone(), value);
                    Ok(())
                }
                other => Err(type_mismatch(current_path, "object", other)),
            }
        }
        PathSegment::Index(index) => {
            if current.is_null() && create {
                *current = Value::Array(Vec::new());
            }
            match current {
                Value::Array(array) => {
                    let index = resolve_or_create_index(*index, array.len(), create, current_path)?;
                    if index == array.len() {
                        array.push(value);
                    } else {
                        array[index] = value;
                    }
                    Ok(())
                }
                other => Err(type_mismatch(current_path, "array", other)),
            }
        }
        PathSegment::Wildcard => unreachable!("wildcards are rejected above"),
    }
}

fn get_mut_at<'value>(
    current: &'value mut Value,
    segments: &[PathSegment],
    current_path: String,
) -> Result<&'value mut Value> {
    if segments.is_empty() {
        return Ok(current);
    }

    let segment = &segments[0];
    match segment {
        PathSegment::Key(key) => match current {
            Value::Object(object) => {
                let child = object.get_mut(key).ok_or_else(|| {
                    FormatEditError::PathNotFound(join_path(&current_path, segment))
                })?;
                get_mut_at(child, &segments[1..], join_path(&current_path, segment))
            }
            other => Err(type_mismatch(&current_path, "object", other)),
        },
        PathSegment::Index(index) => match current {
            Value::Array(array) => {
                let index = resolve_existing_index(*index, array.len(), &current_path)?;
                get_mut_at(
                    &mut array[index],
                    &segments[1..],
                    join_path(&current_path, segment),
                )
            }
            other => Err(type_mismatch(&current_path, "array", other)),
        },
        PathSegment::Wildcard => unreachable!("wildcards are rejected above"),
    }
}

fn reject_wildcard(path: &JsonPath) -> Result<()> {
    if path.is_deterministic() {
        Ok(())
    } else {
        Err(FormatEditError::WildcardMutation(path.to_string()))
    }
}

fn container_for(next_segment: &PathSegment) -> Value {
    match next_segment {
        PathSegment::Index(_) => Value::Array(Vec::new()),
        PathSegment::Key(_) | PathSegment::Wildcard => Value::Object(Map::new()),
    }
}

fn resolve_existing_index(index: i64, len: usize, path: &str) -> Result<usize> {
    let resolved = if index < 0 { len as i64 + index } else { index };
    if resolved < 0 || resolved >= len as i64 {
        Err(FormatEditError::IndexOutOfBounds {
            path: path.to_string(),
            index,
            len,
        })
    } else {
        Ok(resolved as usize)
    }
}

fn resolve_or_create_index(index: i64, len: usize, create: bool, path: &str) -> Result<usize> {
    let resolved = if index < 0 { len as i64 + index } else { index };
    if resolved < 0 || resolved > len as i64 || (resolved == len as i64 && !create) {
        Err(FormatEditError::IndexOutOfBounds {
            path: path.to_string(),
            index,
            len,
        })
    } else {
        Ok(resolved as usize)
    }
}

fn resolve_insert_index(index: i64, len: usize, path: &str) -> Result<usize> {
    let resolved = if index < 0 { len as i64 + index } else { index };
    if resolved < 0 || resolved > len as i64 {
        Err(FormatEditError::IndexOutOfBounds {
            path: path.to_string(),
            index,
            len,
        })
    } else {
        Ok(resolved as usize)
    }
}

fn type_mismatch(path: &str, expected: &'static str, found: &Value) -> FormatEditError {
    FormatEditError::TypeMismatch {
        path: path.to_string(),
        expected,
        found: value_type(found),
    }
}

fn path_for_segments(segments: &[PathSegment]) -> String {
    let mut path = "$".to_string();
    for segment in segments {
        path = join_path(&path, segment);
    }
    path
}

fn join_path(path: &str, segment: &PathSegment) -> String {
    format!("{path}{segment}")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{edit, path::JsonPath};

    #[test]
    fn set_creates_nested_objects_and_arrays() {
        let mut value = json!({});
        let path = "$.server.hosts[0].port".parse::<JsonPath>().unwrap();
        edit::set(&mut value, &path, json!(8080), true).unwrap();
        assert_eq!(value, json!({ "server": { "hosts": [{ "port": 8080 }] } }));
    }

    #[test]
    fn delete_removes_object_key() {
        let mut value = json!({ "server": { "port": 8080, "host": "localhost" } });
        let path = "$.server.port".parse::<JsonPath>().unwrap();
        edit::delete(&mut value, &path).unwrap();
        assert_eq!(value, json!({ "server": { "host": "localhost" } }));
    }

    #[test]
    fn append_can_create_missing_array() {
        let mut value = json!({});
        let path = "$.plugins".parse::<JsonPath>().unwrap();
        edit::append(&mut value, &path, json!({ "name": "auth" }), true).unwrap();
        assert_eq!(value, json!({ "plugins": [{ "name": "auth" }] }));
    }

    #[test]
    fn insert_places_value_before_index() {
        let mut value = json!({ "items": ["b", "c"] });
        let path = "$.items[0]".parse::<JsonPath>().unwrap();
        edit::insert(&mut value, &path, json!("a")).unwrap();
        assert_eq!(value, json!({ "items": ["a", "b", "c"] }));
    }
}
