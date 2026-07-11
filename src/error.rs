use std::io;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, FormatEditError>;

#[derive(Debug, Error)]
pub enum FormatEditError {
    #[error("unsupported file format for {0}; use --format json or --format yaml")]
    UnknownFormat(String),
    #[error("failed to read or write file: {0}")]
    File(#[from] io::Error),
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid YAML: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("invalid path: {0}")]
    InvalidPath(String),
    #[error("path not found: {0}")]
    PathNotFound(String),
    #[error("path type mismatch at {path}: expected {expected}, found {found}")]
    TypeMismatch {
        path: String,
        expected: &'static str,
        found: &'static str,
    },
    #[error("path contains wildcard and cannot be used for mutation: {0}")]
    WildcardMutation(String),
    #[error("value is required; pass VALUE or --value-file")]
    MissingValue,
    #[error("VALUE and --value-file cannot be used together")]
    ConflictingValueSources,
    #[error("invalid value; expected JSON or YAML (JSON: {json}; YAML: {yaml})")]
    InvalidValue { json: String, yaml: String },
    #[error("array index {index} is out of bounds at {path} (len {len})")]
    IndexOutOfBounds {
        path: String,
        index: i64,
        len: usize,
    },
    #[error("insert path must end with an array index, such as $.items[0]")]
    InsertNeedsIndex,
    #[error("batch selection did not find any files; pass --file or --root with --include")]
    BatchNoFiles,
    #[error("invalid regular expression: {0}")]
    InvalidRegex(String),
    #[error("batch target is not an object: {0}")]
    BatchTargetNotObject(String),
    #[error("batch target is not a string: {0}")]
    BatchTargetNotString(String),
    #[error("key already exists at {0}; use --overwrite or --if-missing")]
    KeyAlreadyExists(String),
}

impl FormatEditError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnknownFormat(_) => "unknown_format",
            Self::File(_) => "file_error",
            Self::Json(_) => "invalid_json",
            Self::Yaml(_) => "invalid_yaml",
            Self::InvalidPath(_) => "invalid_path",
            Self::PathNotFound(_) => "path_not_found",
            Self::TypeMismatch { .. } => "type_mismatch",
            Self::WildcardMutation(_) => "wildcard_mutation",
            Self::MissingValue => "missing_value",
            Self::ConflictingValueSources => "conflicting_value_sources",
            Self::InvalidValue { .. } => "invalid_value",
            Self::IndexOutOfBounds { .. } => "index_out_of_bounds",
            Self::InsertNeedsIndex => "insert_needs_index",
            Self::BatchNoFiles => "batch_no_files",
            Self::InvalidRegex(_) => "invalid_regex",
            Self::BatchTargetNotObject(_) => "batch_target_not_object",
            Self::BatchTargetNotString(_) => "batch_target_not_string",
            Self::KeyAlreadyExists(_) => "key_already_exists",
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        let mut object = serde_json::Map::new();
        object.insert(
            "error".to_string(),
            serde_json::Value::String(self.code().to_string()),
        );
        object.insert(
            "message".to_string(),
            serde_json::Value::String(self.to_string()),
        );

        match self {
            Self::UnknownFormat(path) => {
                object.insert("path".to_string(), serde_json::Value::String(path.clone()));
            }
            Self::File(err) => {
                object.insert(
                    "detail".to_string(),
                    serde_json::Value::String(err.to_string()),
                );
            }
            Self::Json(err) => {
                object.insert(
                    "detail".to_string(),
                    serde_json::Value::String(err.to_string()),
                );
            }
            Self::Yaml(err) => {
                object.insert(
                    "detail".to_string(),
                    serde_json::Value::String(err.to_string()),
                );
            }
            Self::InvalidPath(reason) => {
                object.insert(
                    "reason".to_string(),
                    serde_json::Value::String(reason.clone()),
                );
            }
            Self::PathNotFound(path) => {
                object.insert("path".to_string(), serde_json::Value::String(path.clone()));
            }
            Self::TypeMismatch {
                path,
                expected,
                found,
            } => {
                object.insert("path".to_string(), serde_json::Value::String(path.clone()));
                object.insert(
                    "expected".to_string(),
                    serde_json::Value::String((*expected).to_string()),
                );
                object.insert(
                    "found".to_string(),
                    serde_json::Value::String((*found).to_string()),
                );
            }
            Self::WildcardMutation(path) => {
                object.insert("path".to_string(), serde_json::Value::String(path.clone()));
            }
            Self::InvalidValue { json, yaml } => {
                object.insert("json".to_string(), serde_json::Value::String(json.clone()));
                object.insert("yaml".to_string(), serde_json::Value::String(yaml.clone()));
            }
            Self::IndexOutOfBounds { path, index, len } => {
                object.insert("path".to_string(), serde_json::Value::String(path.clone()));
                object.insert(
                    "index".to_string(),
                    serde_json::Value::Number((*index).into()),
                );
                object.insert("len".to_string(), serde_json::Value::Number((*len).into()));
            }
            Self::InvalidRegex(reason) => {
                object.insert(
                    "reason".to_string(),
                    serde_json::Value::String(reason.clone()),
                );
            }
            Self::BatchTargetNotObject(path)
            | Self::BatchTargetNotString(path)
            | Self::KeyAlreadyExists(path) => {
                object.insert("path".to_string(), serde_json::Value::String(path.clone()));
            }
            Self::MissingValue
            | Self::ConflictingValueSources
            | Self::InsertNeedsIndex
            | Self::BatchNoFiles => {}
        }

        serde_json::Value::Object(object)
    }
}

pub fn value_type(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::FormatEditError;

    #[test]
    fn serializes_type_mismatch_as_machine_readable_json() {
        let error = FormatEditError::TypeMismatch {
            path: "$.server.host".to_string(),
            expected: "object",
            found: "string",
        };

        assert_eq!(
            error.to_json(),
            json!({
                "error": "type_mismatch",
                "message": "path type mismatch at $.server.host: expected object, found string",
                "path": "$.server.host",
                "expected": "object",
                "found": "string"
            })
        );
    }
}
