pub mod cli;
pub mod edit;
pub mod error;
pub mod format;
pub mod path;
pub mod query;

use clap::Parser;
use cli::{Cli, Command, ErrorFormat};
use error::Result;
use format::{load_document, parse_format, parse_input_value, print_value, save_document};

pub fn run() -> i32 {
    let cli = Cli::parse();
    let error_format = cli.error_format;
    match execute(cli) {
        Ok(()) => 0,
        Err(err) => {
            print_error(&err, error_format);
            1
        }
    }
}

fn print_error(err: &error::FormatEditError, format: ErrorFormat) {
    match format {
        ErrorFormat::Text => eprintln!("{err}"),
        ErrorFormat::Json => eprintln!("{}", err.to_json()),
    }
}

fn execute(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Get {
            file,
            path,
            format,
            raw,
        } => {
            let format = parse_format(file.as_path(), format)?;
            let document = load_document(file.as_path(), format)?;
            let path = path.parse()?;
            let matches = query::get(&document, &path);

            if matches.is_empty() {
                return Err(error::FormatEditError::PathNotFound(path.to_string()));
            }

            if matches.len() == 1 {
                print_value(matches[0], format, raw)?;
            } else {
                let values = matches.into_iter().cloned().collect::<Vec<_>>();
                print_value(&serde_json::Value::Array(values), format, false)?;
            }
        }
        Command::Exists { file, path, format } => {
            let format = parse_format(file.as_path(), format)?;
            let document = load_document(file.as_path(), format)?;
            let path = path.parse()?;
            if query::exists(&document, &path) {
                println!("true");
            } else {
                println!("false");
                std::process::exit(1);
            }
        }
        Command::Set {
            file,
            path,
            value,
            value_file,
            format,
            raw,
            write: _,
            dry_run,
            no_create,
        } => {
            let format = parse_format(file.as_path(), format)?;
            let mut document = load_document(file.as_path(), format)?;
            let path = path.parse()?;
            let value = parse_input_value(value, value_file.as_deref(), raw)?;
            edit::set(&mut document, &path, value, !no_create)?;
            save_document(file.as_path(), format, &document, !dry_run)?;
        }
        Command::Delete {
            file,
            path,
            format,
            write: _,
            dry_run,
            ignore_missing,
        } => {
            let format = parse_format(file.as_path(), format)?;
            let mut document = load_document(file.as_path(), format)?;
            let path = path.parse()?;
            match edit::delete(&mut document, &path) {
                Ok(()) => save_document(file.as_path(), format, &document, !dry_run)?,
                Err(error::FormatEditError::PathNotFound(_)) if ignore_missing => {
                    save_document(file.as_path(), format, &document, !dry_run)?;
                }
                Err(err) => return Err(err),
            }
        }
        Command::Append {
            file,
            path,
            value,
            value_file,
            format,
            raw,
            write: _,
            dry_run,
            create,
        } => {
            let format = parse_format(file.as_path(), format)?;
            let mut document = load_document(file.as_path(), format)?;
            let path = path.parse()?;
            let value = parse_input_value(value, value_file.as_deref(), raw)?;
            edit::append(&mut document, &path, value, create)?;
            save_document(file.as_path(), format, &document, !dry_run)?;
        }
        Command::Insert {
            file,
            path,
            value,
            value_file,
            format,
            raw,
            write: _,
            dry_run,
        } => {
            let format = parse_format(file.as_path(), format)?;
            let mut document = load_document(file.as_path(), format)?;
            let path = path.parse()?;
            let value = parse_input_value(value, value_file.as_deref(), raw)?;
            edit::insert(&mut document, &path, value)?;
            save_document(file.as_path(), format, &document, !dry_run)?;
        }
    }

    Ok(())
}
