use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_file(name: &str, contents: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!("fe-test-{}-{unique}-{name}", std::process::id()));
    fs::write(&path, contents).unwrap();
    path
}

#[test]
fn append_writes_by_default() {
    let file = temp_file("append-default.json", r#"{"apps":[],"source":"x"}"#);

    let output = Command::new(env!("CARGO_BIN_EXE_fe"))
        .args([
            "append",
            "--format",
            "json",
            file.to_str().unwrap(),
            "$.apps",
            r#""item1""#,
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    let written = fs::read_to_string(file).unwrap();
    assert!(written.contains(r#""item1""#));
}

#[test]
fn dry_run_prints_without_writing() {
    let file = temp_file("dry-run.json", r#"{"apps":[],"source":"x"}"#);

    let output = Command::new(env!("CARGO_BIN_EXE_fe"))
        .args([
            "append",
            "--format",
            "json",
            "--dry-run",
            file.to_str().unwrap(),
            "$.apps",
            r#""item1""#,
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains(r#""item1""#));
    assert_eq!(
        fs::read_to_string(file).unwrap(),
        r#"{"apps":[],"source":"x"}"#
    );
}

#[test]
fn extensionless_json_is_detected_from_contents() {
    let file = temp_file("extensionless", r#"{"title":"hello"}"#);

    let output = Command::new(env!("CARGO_BIN_EXE_fe"))
        .args(["get", file.to_str().unwrap(), "$.title", "--raw"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "hello\n");
}

#[test]
fn version_flag_prints_package_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_fe"))
        .arg("--version")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("fe {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn version_subcommand_prints_package_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_fe"))
        .arg("version")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("fe {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn stdout_alias_is_shown_in_mutation_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_fe"))
        .args(["set", "--help"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--dry-run"));
    assert!(stdout.contains("--stdout"));
}

#[test]
fn parse_errors_respect_json_error_format() {
    let file = temp_file("parse-error.json", r#"{"server":{"port":3000}}"#);

    let output = Command::new(env!("CARGO_BIN_EXE_fe"))
        .args([
            "--error-format",
            "json",
            "get",
            file.to_str().unwrap(),
            "$.server.port",
            "--not-real",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(stderr["error"], "argument_error");
    assert_eq!(stderr["kind"], "UnknownArgument");
}
