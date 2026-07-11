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

fn temp_dir(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!("fe-test-{}-{unique}-{name}", std::process::id()));
    fs::create_dir_all(&path).unwrap();
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
fn preview_set_prints_minimal_diff_without_writing() {
    let original = "{\n  \"server\": {\n    \"port\": 3000,\n    \"host\": \"localhost\"\n  }\n}\n";
    let file = temp_file("preview-set.json", original);

    let output = Command::new(env!("CARGO_BIN_EXE_fe"))
        .args([
            "preview",
            "set",
            file.to_str().unwrap(),
            "$.server.port",
            "8080",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("-    \"port\": 3000"));
    assert!(stdout.contains("+    \"port\": 8080"));
    assert_eq!(fs::read_to_string(file).unwrap(), original);
}

#[test]
fn preview_is_documented_in_top_level_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_fe"))
        .arg("--help")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("preview"));
    assert!(stdout.contains("minimal diff"));
}

#[test]
fn preview_batch_put_shows_all_files_without_writing() {
    let first_original = "{\n  \"services\": [\n    {\n      \"name\": \"api\"\n    }\n  ]\n}\n";
    let second_original = "{\n  \"services\": [\n    {\n      \"name\": \"jobs\"\n    }\n  ]\n}\n";
    let first = temp_file("preview-batch-first.json", first_original);
    let second = temp_file("preview-batch-second.json", second_original);

    let output = Command::new(env!("CARGO_BIN_EXE_fe"))
        .args([
            "preview",
            "batch",
            "put",
            "--file",
            first.to_str().unwrap(),
            "--file",
            second.to_str().unwrap(),
            "$.services[*]",
            "timeout",
            "30",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(first.to_str().unwrap()));
    assert!(stdout.contains(second.to_str().unwrap()));
    assert!(stdout.contains("2 file(s) changed · 2 structured change(s)"));
    assert_eq!(fs::read_to_string(first).unwrap(), first_original);
    assert_eq!(fs::read_to_string(second).unwrap(), second_original);
}

#[test]
fn batch_set_updates_wildcard_matches_in_multiple_files() {
    let first = temp_file(
        "batch-set-first.json",
        "{\n  \"services\": [\n    {\n      \"enabled\": true\n    }\n  ]\n}\n",
    );
    let second = temp_file(
        "batch-set-second.json",
        "{\n  \"services\": [\n    {\n      \"enabled\": true\n    },\n    {\n      \"enabled\": true\n    }\n  ]\n}\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_fe"))
        .args([
            "batch",
            "set",
            "--file",
            first.to_str().unwrap(),
            "--file",
            second.to_str().unwrap(),
            "$.services[*].enabled",
            "false",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!fs::read_to_string(first).unwrap().contains("true"));
    assert!(!fs::read_to_string(second).unwrap().contains("true"));
}

#[test]
fn preview_batch_delete_by_key_regex_does_not_write() {
    let original =
        "{\n  \"services\": [\n    {\n      \"x-old\": 1,\n      \"keep\": 2\n    }\n  ]\n}\n";
    let file = temp_file("preview-batch-delete.json", original);

    let output = Command::new(env!("CARGO_BIN_EXE_fe"))
        .args([
            "preview",
            "batch",
            "delete",
            "--file",
            file.to_str().unwrap(),
            "$.services[*]",
            "--key-regex",
            "^x-",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("-      \"x-old\": 1,"));
    assert_eq!(fs::read_to_string(file).unwrap(), original);
}

#[test]
fn batch_help_lists_structured_operations() {
    let output = Command::new(env!("CARGO_BIN_EXE_fe"))
        .args(["batch", "--help"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for command in ["set", "put", "delete", "replace", "append"] {
        assert!(stdout.contains(command), "missing {command} in batch help");
    }
}

#[test]
fn preview_batch_honors_root_include_and_exclude() {
    let root = temp_dir("batch-globs");
    let included = root.join("included.json");
    let excluded_dir = root.join("vendor");
    let excluded = excluded_dir.join("excluded.json");
    fs::create_dir_all(&excluded_dir).unwrap();
    let original = "{\n  \"image\": \"old/api\"\n}\n";
    fs::write(&included, original).unwrap();
    fs::write(&excluded, original).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_fe"))
        .args([
            "preview",
            "batch",
            "replace",
            "--root",
            root.to_str().unwrap(),
            "--include",
            "**/*.json",
            "--exclude",
            "vendor/**",
            "$.image",
            "^old/",
            "new/",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(included.to_str().unwrap()));
    assert!(!stdout.contains(excluded.to_str().unwrap()));
    assert_eq!(fs::read_to_string(included).unwrap(), original);
    assert_eq!(fs::read_to_string(excluded).unwrap(), original);
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
