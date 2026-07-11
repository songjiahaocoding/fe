use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

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
    /// Preview a structured edit as a minimal diff without writing the file.
    Preview {
        #[command(subcommand)]
        command: PreviewCommand,
    },
    /// Apply one structured operation to multiple nodes or files.
    Batch {
        #[command(subcommand)]
        command: BatchCommand,
    },
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

#[derive(Debug, Subcommand)]
pub enum PreviewCommand {
    /// Preview a batch edit across matching nodes and files.
    Batch {
        #[command(subcommand)]
        command: BatchCommand,
    },
    /// Preview replacing or creating a value at a deterministic path.
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
        #[arg(long, help = "Fail if any path segment is missing")]
        no_create: bool,
    },
    /// Preview deleting a key or array element.
    Delete {
        file: PathBuf,
        path: String,
        #[arg(long, value_enum)]
        format: Option<FormatArg>,
        #[arg(long, help = "Succeed when the path is already missing")]
        ignore_missing: bool,
    },
    /// Preview appending a value to an array.
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
        #[arg(long, help = "Create the target array if it is missing")]
        create: bool,
    },
    /// Preview inserting a value before an array index.
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
    },
}

#[derive(Debug, Args)]
pub struct BatchFiles {
    /// Edit an explicit file. May be passed more than once.
    #[arg(long = "file", value_name = "FILE")]
    pub files: Vec<PathBuf>,
    /// Search for files below this directory.
    #[arg(long, value_name = "DIR")]
    pub root: Option<PathBuf>,
    /// Include files matching this glob, relative to --root. May be repeated.
    #[arg(long, value_name = "GLOB", requires = "root")]
    pub include: Vec<String>,
    /// Exclude files matching this glob, relative to --root. May be repeated.
    #[arg(long, value_name = "GLOB", requires = "root")]
    pub exclude: Vec<String>,
    #[arg(long, value_enum)]
    pub format: Option<FormatArg>,
}

#[derive(Debug, Subcommand)]
pub enum BatchCommand {
    /// Set every value matched by PATH.
    Set {
        #[command(flatten)]
        files: BatchFiles,
        path: String,
        value: Option<String>,
        #[arg(long)]
        value_file: Option<PathBuf>,
        #[arg(long)]
        raw: bool,
    },
    /// Add or update a key/value pair in every object matched by PATH.
    Put {
        #[command(flatten)]
        files: BatchFiles,
        path: String,
        key: String,
        value: Option<String>,
        #[arg(long)]
        value_file: Option<PathBuf>,
        #[arg(long)]
        raw: bool,
        /// Replace the value when KEY already exists.
        #[arg(long, conflicts_with = "if_missing")]
        overwrite: bool,
        /// Skip objects where KEY already exists.
        #[arg(long)]
        if_missing: bool,
    },
    /// Delete every node matched by PATH, or matching keys below each object.
    Delete {
        #[command(flatten)]
        files: BatchFiles,
        path: String,
        #[arg(long, conflicts_with = "key_regex")]
        key: Option<String>,
        #[arg(long, conflicts_with = "key")]
        key_regex: Option<String>,
    },
    /// Replace text inside every string value matched by PATH.
    Replace {
        #[command(flatten)]
        files: BatchFiles,
        path: String,
        pattern: String,
        replacement: String,
    },
    /// Append a value to every array matched by PATH.
    Append {
        #[command(flatten)]
        files: BatchFiles,
        path: String,
        value: Option<String>,
        #[arg(long)]
        value_file: Option<PathBuf>,
        #[arg(long)]
        raw: bool,
    },
}
