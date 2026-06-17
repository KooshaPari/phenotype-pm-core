//! Integration tests for `trace-gate` binary.
//!
//! Verifies exit codes using fixture manifests and fixture source files.

use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(path)
}

/// When one FR is missing from source, gate must exit 1.
#[test]
fn exit_1_when_fr_missing() {
    let mut cmd = Command::cargo_bin("trace-gate").unwrap();
    cmd.arg("--manifest")
        .arg(fixture("manifest_partial.toml"))
        .arg("--src")
        .arg(fixture("src"));

    cmd.assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("FAIL"))
        .stderr(predicate::str::contains("1 FR(s) not covered"));
}

/// When all FRs in the manifest are covered, gate must exit 0.
#[test]
fn exit_0_when_all_covered() {
    let mut cmd = Command::cargo_bin("trace-gate").unwrap();
    cmd.arg("--manifest")
        .arg(fixture("manifest_all.toml"))
        .arg("--src")
        .arg(fixture("src"));

    cmd.assert().success().code(0);
}

/// --json flag emits valid JSON summary.
#[test]
fn json_flag_emits_valid_json() {
    let mut cmd = Command::cargo_bin("trace-gate").unwrap();
    cmd.arg("--manifest")
        .arg(fixture("manifest_all.toml"))
        .arg("--src")
        .arg(fixture("src"))
        .arg("--json");

    let output = cmd.assert().success().get_output().stdout.clone();
    // Find the JSON block (after the human-readable header lines).
    let stdout = String::from_utf8(output).unwrap();
    // The JSON blob starts with '{'.
    let json_start = stdout.find('{').expect("JSON output not found");
    let json_str = &stdout[json_start..];
    let parsed: serde_json::Value =
        serde_json::from_str(json_str).expect("output is not valid JSON");
    assert!(parsed.get("requirements").is_some());
    assert!(parsed.get("all_covered").is_some());
}

/// Empty manifest exits 0 (nothing to check).
#[test]
fn exit_0_on_empty_manifest() {
    use std::io::Write;
    let dir = tempfile::tempdir().unwrap();
    let manifest_path = dir.path().join("trace-gate.toml");
    let src_path = dir.path().join("src");
    std::fs::create_dir_all(&src_path).unwrap();
    std::fs::File::create(&manifest_path)
        .unwrap()
        .write_all(b"# empty\n")
        .unwrap();

    let mut cmd = Command::cargo_bin("trace-gate").unwrap();
    cmd.arg("--manifest")
        .arg(&manifest_path)
        .arg("--src")
        .arg(&src_path);

    cmd.assert().success().code(0);
}

/// Non-existent manifest exits 2 (usage error, not gate failure).
#[test]
fn exit_2_on_missing_manifest() {
    let mut cmd = Command::cargo_bin("trace-gate").unwrap();
    cmd.arg("--manifest")
        .arg("/tmp/does_not_exist_at_all.toml")
        .arg("--src")
        .arg("src");

    cmd.assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("cannot read manifest"));
}

/// --push flag prints stub payload without affecting exit code.
#[test]
fn push_flag_prints_stub_without_affecting_exit() {
    let mut cmd = Command::cargo_bin("trace-gate").unwrap();
    cmd.arg("--manifest")
        .arg(fixture("manifest_all.toml"))
        .arg("--src")
        .arg(fixture("src"))
        .arg("--push")
        .arg("https://tracera.example.invalid/api/v1/coverage/ingest");

    cmd.assert()
        .success()
        .code(0)
        .stderr(predicate::str::contains("would push"));
}
