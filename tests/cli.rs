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
