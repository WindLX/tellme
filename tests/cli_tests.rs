use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

struct TestEnv {
    config_dir: TempDir,
    temp_dir: TempDir,
    pid: String,
}

impl TestEnv {
    fn new() -> Self {
        let config_dir = tempfile::tempdir().expect("failed to create config dir");
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let pid = std::process::id().to_string();
        Self {
            config_dir,
            temp_dir,
            pid,
        }
    }

    fn cmd(&self) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_tellme"));
        cmd.env("TELLME_CONFIG_DIR", self.config_dir.path())
            .env("TELLME_TEMP_DIR", self.temp_dir.path())
            .env("TELLME_SHELL_PID", &self.pid);
        cmd
    }
}

#[test]
fn test_status_workflow() {
    let env = TestEnv::new();

    // Default status should be disabled (or init state)
    // Actually, update: src/config.rs says if status file read fails, it writes "disabled".
    env.cmd()
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("DISABLED"));

    // Enable it
    env.cmd()
        .arg("on")
        .assert()
        .success()
        .stdout(predicate::str::contains("ENABLED"));

    // Check status again
    env.cmd()
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("ENABLED"));

    // Disable it
    env.cmd()
        .arg("off")
        .assert()
        .success()
        .stdout(predicate::str::contains("DISABLED"));
}

#[test]
fn test_capture_workflow() {
    let env = TestEnv::new();

    // Enable recording
    env.cmd().arg("on").assert().success();

    // 1. Check should-prepare
    env.cmd()
        .args(&["internal", "--should-prepare", "echo hello"])
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));

    // 2. Prepare
    // This should output the path to the log file
    let assert = env
        .cmd()
        .args(&["internal", "--prepare", "echo hello"])
        .assert()
        .success();

    let output = assert.get_output();
    let log_path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let log_path = PathBuf::from(&log_path_str);

    // Verify internally created files exist
    // The pattern is somewhat internal, but we know it stores in temp_dir
    assert!(log_path.exists());

    // Simulate the shell writing to this file
    fs::write(&log_path, "hello world output\x1b[31m colored\x1b[0m")
        .expect("failed to write to log");

    // 3. Capture command (tellme without args)
    // We'll output to a specific file to verify content
    let result_file = env.temp_dir.path().join("result.log");

    env.cmd()
        .arg("-o")
        .arg(&result_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("Output saved to"));

    assert!(result_file.exists());
    let content = fs::read_to_string(&result_file).expect("failed to read result");

    // Verify content
    assert!(content.contains("Command:\necho hello"));
    assert!(content.contains("hello world output colored"));
    // Should be stripped of color by default
    assert!(!content.contains("\x1b[31m"));
}

#[test]
fn test_capture_raw() {
    let env = TestEnv::new();
    env.cmd().arg("on").assert().success();

    // Prepare
    let assert = env
        .cmd()
        .args(&["internal", "--prepare", "echo color"])
        .assert()
        .success();
    let log_path_str = String::from_utf8_lossy(&assert.get_output().stdout)
        .trim()
        .to_string();
    let log_path = PathBuf::from(log_path_str);

    // Simulate shell writing ANSI
    fs::write(&log_path, "\x1b[31mRED\x1b[0m").expect("failed to write log");

    // Capture with --raw
    let result_file = env.temp_dir.path().join("raw.log");
    env.cmd()
        .arg("--raw")
        .arg("-o")
        .arg(&result_file)
        .assert()
        .success();

    let content = fs::read_to_string(&result_file).expect("failed to read result");
    assert!(content.contains("\x1b[31mRED\x1b[0m"));
}

#[test]
fn test_skip_commands() {
    let env = TestEnv::new();
    env.cmd().arg("on").assert().success();

    // "tellme" is in default skip list
    env.cmd()
        .args(&["internal", "--should-prepare", "tellme status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("false"));

    // Add "secret_cmd" to skip list
    env.cmd()
        .args(&["config", "--add", "secret_cmd"])
        .assert()
        .success();

    // Verify it is skipped
    env.cmd()
        .args(&["internal", "--should-prepare", "secret_cmd"])
        .assert()
        .success()
        .stdout(predicate::str::contains("false"));

    // Verify normal command is still ok
    env.cmd()
        .args(&["internal", "--should-prepare", "ls"])
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}
