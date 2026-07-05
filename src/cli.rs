use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum FormatArg {
    Json,
    Yaml,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, ValueEnum)]
pub enum ErrorFormat {
    Text,
    Json,
}

#[derive(Debug, Parser)]
#[command(name = "fe")]
#[command(about = "Agent-friendly structured file editor for JSON and YAML")]
pub struct Cli {
    #[arg(long, value_enum, default_value = "text", global = true)]
    pub error_format: ErrorFormat,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Read one or more values at a JSONPath-style path.
    Get {
        file: PathBuf,
        path: String,
        #[arg(long, value_enum)]
        format: Option<FormatArg>,
        #[arg(long)]
        raw: bool,
    },
    /// Return success when a path exists.
    Exists {
        file: PathBuf,
        path: String,
        #[arg(long, value_enum)]
        format: Option<FormatArg>,
    },
    /// Replace or create a value at a deterministic path.
    Set {
        file: PathBuf,
        path: String,
        value: Option<String>,
        #[arg(long)]
        value_file: Option<PathBuf>,
        #[arg(long, value_enum)]
        format: Option<FormatArg>,
        #[arg(long)]
        raw: bool,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        no_create: bool,
    },
    /// Delete a key or array element.
    Delete {
        file: PathBuf,
        path: String,
        #[arg(long, value_enum)]
        format: Option<FormatArg>,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        ignore_missing: bool,
    },
    /// Append a value to an array.
    Append {
        file: PathBuf,
        path: String,
        value: Option<String>,
        #[arg(long)]
        value_file: Option<PathBuf>,
        #[arg(long, value_enum)]
        format: Option<FormatArg>,
        #[arg(long)]
        raw: bool,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        create: bool,
    },
    /// Insert a value before an array index, such as $.items[0].
    Insert {
        file: PathBuf,
        path: String,
        value: Option<String>,
        #[arg(long)]
        value_file: Option<PathBuf>,
        #[arg(long, value_enum)]
        format: Option<FormatArg>,
        #[arg(long)]
        raw: bool,
        #[arg(long)]
        write: bool,
    },
}
