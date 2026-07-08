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
#[command(version)]
pub struct Cli {
    #[arg(long, value_enum, default_value = "text", global = true)]
    pub error_format: ErrorFormat,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Print the fe version.
    Version,
    /// Read one or more values at a JSONPath-style path.
    Get {
        file: PathBuf,
        path: String,
        #[arg(long, value_enum)]
        format: Option<FormatArg>,
        #[arg(long, help = "Print scalar values without JSON/YAML quoting")]
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
        #[arg(long, help = "Treat VALUE as a raw string instead of JSON/YAML")]
        raw: bool,
        #[arg(long, help = "Write changes back to FILE (default)")]
        write: bool,
        #[arg(
            long,
            visible_alias = "stdout",
            conflicts_with = "write",
            help = "Print the changed document instead of writing FILE"
        )]
        dry_run: bool,
        #[arg(long, help = "Fail if any path segment is missing")]
        no_create: bool,
    },
    /// Delete a key or array element.
    Delete {
        file: PathBuf,
        path: String,
        #[arg(long, value_enum)]
        format: Option<FormatArg>,
        #[arg(long, help = "Write changes back to FILE (default)")]
        write: bool,
        #[arg(
            long,
            visible_alias = "stdout",
            conflicts_with = "write",
            help = "Print the changed document instead of writing FILE"
        )]
        dry_run: bool,
        #[arg(long, help = "Succeed when the path is already missing")]
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
        #[arg(long, help = "Treat VALUE as a raw string instead of JSON/YAML")]
        raw: bool,
        #[arg(long, help = "Write changes back to FILE (default)")]
        write: bool,
        #[arg(
            long,
            visible_alias = "stdout",
            conflicts_with = "write",
            help = "Print the changed document instead of writing FILE"
        )]
        dry_run: bool,
        #[arg(long, help = "Create the target array if it is missing")]
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
        #[arg(long, help = "Treat VALUE as a raw string instead of JSON/YAML")]
        raw: bool,
        #[arg(long, help = "Write changes back to FILE (default)")]
        write: bool,
        #[arg(
            long,
            visible_alias = "stdout",
            conflicts_with = "write",
            help = "Print the changed document instead of writing FILE"
        )]
        dry_run: bool,
    },
}
