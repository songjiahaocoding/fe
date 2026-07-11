pub mod batch;
pub mod cli;
pub mod edit;
pub mod error;
pub mod format;
pub mod path;
pub mod query;

use std::ffi::OsString;

use clap::Parser;
use cli::{Cli, Command, ErrorFormat, PreviewCommand};
use error::Result;
use format::{load_document, parse_format, parse_input_value, print_value, save_document};

pub fn run() -> i32 {
    let error_format = requested_error_format(std::env::args_os());
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            if err.use_stderr() && error_format == ErrorFormat::Json {
                print_parse_error_json(&err);
            } else if let Err(print_err) = err.print() {
                eprintln!("failed to print CLI error: {print_err}");
            }
            return if err.use_stderr() { 2 } else { 0 };
        }
    };
    let error_format = cli.error_format;
    match execute(cli) {
        Ok(()) => 0,
        Err(err) => {
            print_error(&err, error_format);
            1
        }
    }
}

fn requested_error_format<I>(args: I) -> ErrorFormat
where
    I: IntoIterator<Item = OsString>,
{
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        if arg == "--error-format=json" {
            return ErrorFormat::Json;
        }
        if arg == "--error-format" {
            return match args.next().and_then(|value| value.into_string().ok()) {
                Some(value) if value == "json" => ErrorFormat::Json,
                _ => ErrorFormat::Text,
            };
        }
    }
    ErrorFormat::Text
}

fn print_parse_error_json(err: &clap::Error) {
    let payload = serde_json::json!({
        "error": "argument_error",
        "message": err.to_string(),
        "kind": format!("{:?}", err.kind()),
    });
    eprintln!("{payload}");
}

fn print_error(err: &error::FormatEditError, format: ErrorFormat) {
    match format {
        ErrorFormat::Text => eprintln!("{err}"),
        ErrorFormat::Json => eprintln!("{}", err.to_json()),
    }
}

fn execute(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Version => {
            println!("fe {}", env!("CARGO_PKG_VERSION"));
        }
        Command::Preview { command } => execute_preview(command)?,
        Command::Batch { command } => {
            let plan = batch::build(command)?;
            batch::apply(&plan)?;
        }
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

fn execute_preview(command: PreviewCommand) -> Result<()> {
    use std::fs;

    let (file, format, document) = match command {
        PreviewCommand::Batch { command } => {
            let plan = batch::build(command)?;
            batch::print_preview(&plan);
            return Ok(());
        }
        PreviewCommand::Set {
            file,
            path,
            value,
            value_file,
            format,
            raw,
            no_create,
        } => {
            let format = parse_format(file.as_path(), format)?;
            let mut document = load_document(file.as_path(), format)?;
            let path = path.parse()?;
            let value = parse_input_value(value, value_file.as_deref(), raw)?;
            edit::set(&mut document, &path, value, !no_create)?;
            (file, format, document)
        }
        PreviewCommand::Delete {
            file,
            path,
            format,
            ignore_missing,
        } => {
            let format = parse_format(file.as_path(), format)?;
            let mut document = load_document(file.as_path(), format)?;
            let path = path.parse()?;
            match edit::delete(&mut document, &path) {
                Ok(()) => {}
                Err(error::FormatEditError::PathNotFound(_)) if ignore_missing => {}
                Err(err) => return Err(err),
            }
            (file, format, document)
        }
        PreviewCommand::Append {
            file,
            path,
            value,
            value_file,
            format,
            raw,
            create,
        } => {
            let format = parse_format(file.as_path(), format)?;
            let mut document = load_document(file.as_path(), format)?;
            let path = path.parse()?;
            let value = parse_input_value(value, value_file.as_deref(), raw)?;
            edit::append(&mut document, &path, value, create)?;
            (file, format, document)
        }
        PreviewCommand::Insert {
            file,
            path,
            value,
            value_file,
            format,
            raw,
        } => {
            let format = parse_format(file.as_path(), format)?;
            let mut document = load_document(file.as_path(), format)?;
            let path = path.parse()?;
            let value = parse_input_value(value, value_file.as_deref(), raw)?;
            edit::insert(&mut document, &path, value)?;
            (file, format, document)
        }
    };

    let before = fs::read_to_string(&file)?;
    let after = format::serialize(format, &document)?;
    let display = file.display().to_string();
    let diff = similar::TextDiff::from_lines(&before, &after)
        .unified_diff()
        .context_radius(3)
        .header(&display, &display)
        .to_string();

    if diff.is_empty() {
        println!("No changes: {display}");
    } else {
        print!("{diff}");
    }
    Ok(())
}
