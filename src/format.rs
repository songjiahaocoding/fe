use std::{fs, io::Write, path::Path};

use serde_json::Value;

use crate::{
    cli::FormatArg,
    error::{FormatEditError, Result},
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DocumentFormat {
    Json,
    Yaml,
}

pub fn parse_format(path: &Path, format: Option<FormatArg>) -> Result<DocumentFormat> {
    if let Some(format) = format {
        return Ok(match format {
            FormatArg::Json => DocumentFormat::Json,
            FormatArg::Yaml => DocumentFormat::Yaml,
        });
    }

    match path.extension().and_then(|extension| extension.to_str()) {
        Some("json") => Ok(DocumentFormat::Json),
        Some("yaml" | "yml") => Ok(DocumentFormat::Yaml),
        _ => Err(FormatEditError::UnknownFormat(path.display().to_string())),
    }
}

pub fn load_document(path: &Path, format: DocumentFormat) -> Result<Value> {
    let input = fs::read_to_string(path)?;
    match format {
        DocumentFormat::Json => Ok(serde_json::from_str(&input)?),
        DocumentFormat::Yaml => Ok(serde_yaml::from_str(&input)?),
    }
}

pub fn save_document(
    path: &Path,
    format: DocumentFormat,
    value: &Value,
    write: bool,
) -> Result<()> {
    let output = serialize(format, value)?;
    if write {
        fs::write(path, output)?;
    } else {
        print!("{output}");
        std::io::stdout().flush()?;
    }
    Ok(())
}

pub fn print_value(value: &Value, format: DocumentFormat, raw: bool) -> Result<()> {
    if raw {
        match value {
            Value::String(text) => println!("{text}"),
            Value::Null => println!("null"),
            Value::Bool(value) => println!("{value}"),
            Value::Number(value) => println!("{value}"),
            _ => print!("{}", serialize(format, value)?),
        }
    } else {
        print!("{}", serialize(format, value)?);
    }
    Ok(())
}

pub fn parse_input_value(
    value: Option<String>,
    value_file: Option<&Path>,
    raw: bool,
) -> Result<Value> {
    let input = match (value, value_file) {
        (Some(_), Some(_)) => return Err(FormatEditError::ConflictingValueSources),
        (Some(value), None) => value,
        (None, Some(path)) => fs::read_to_string(path)?,
        (None, None) => return Err(FormatEditError::MissingValue),
    };

    if raw {
        return Ok(Value::String(input));
    }

    match serde_json::from_str(&input) {
        Ok(value) => Ok(value),
        Err(json_error) => match serde_yaml::from_str(&input) {
            Ok(value) => Ok(value),
            Err(yaml_error) => Err(FormatEditError::InvalidValue {
                json: json_error.to_string(),
                yaml: yaml_error.to_string(),
            }),
        },
    }
}

fn serialize(format: DocumentFormat, value: &Value) -> Result<String> {
    match format {
        DocumentFormat::Json => Ok(format!("{}\n", serde_json::to_string_pretty(value)?)),
        DocumentFormat::Yaml => Ok(serde_yaml::to_string(value)?),
    }
}
