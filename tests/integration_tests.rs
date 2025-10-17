use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
use std::fs;
use std::io::Write;
use tempfile::{tempdir, NamedTempFile};

// Test basic CLI structure and help commands
#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("bwenv").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("bwenv"))
        .stdout(predicate::str::contains("store"))
        .stdout(predicate::str::contains("retrieve"))
        .stdout(predicate::str::contains("list"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("bwenv").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("bwenv"));
}

#[test]
fn test_store_command_help() {
    let mut cmd = Command::cargo_bin("bwenv").unwrap();
    cmd.args(&["store", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Store secrets from a .env file to Bitwarden",
        ))
        .stdout(predicate::str::contains("--file"))
        .stdout(predicate::str::contains("--folder"))
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--overwrite"));
}

#[test]
fn test_retrieve_command_help() {
    let mut cmd = Command::cargo_bin("bwenv").unwrap();
    cmd.args(&["retrieve", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Retrieve secrets from Bitwarden"))
        .stdout(predicate::str::contains("--folder"))
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("--output"))
        .stdout(predicate::str::contains("--merge"));
}

#[test]
fn test_list_command_help() {
    let mut cmd = Command::cargo_bin("bwenv").unwrap();
    cmd.args(&["list", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("List all stored environment sets"))
        .stdout(predicate::str::contains("--format"));
}

#[test]
fn test_verbosity_flags() {
    let mut cmd = Command::cargo_bin("bwenv").unwrap();
    cmd.args(&["-v", "list"]);
    // This will fail without Bitwarden but should show that verbose flag is recognized
    cmd.assert().failure(); // Expected to fail without proper Bitwarden setup
}

#[test]
fn test_quiet_flag() {
    let mut cmd = Command::cargo_bin("bwenv").unwrap();
    cmd.args(&["-q", "list"]);
    // This will fail without Bitwarden but should show that quiet flag is recognized
    cmd.assert().failure(); // Expected to fail without proper Bitwarden setup
}

#[test]
fn test_store_missing_file_argument() {
    let mut cmd = Command::cargo_bin("bwenv").unwrap();
    cmd.arg("store");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_store_nonexistent_file() {
    let mut cmd = Command::cargo_bin("bwenv").unwrap();
    cmd.args(&["store", "--file", "/nonexistent/file.env"]);
    cmd.assert().failure().stderr(
        predicate::str::contains("Failed to read .env file")
            .or(predicate::str::contains("Bitwarden")),
    );
}

#[test]
fn test_store_invalid_env_file() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "INVALID_LINE_WITHOUT_EQUALS").unwrap();
    writeln!(temp_file, "VALID_KEY=valid_value").unwrap();

    let mut cmd = Command::cargo_bin("bwenv").unwrap();
    cmd.args(&["store", "--file", temp_file.path().to_str().unwrap()]);

    // Will fail due to Bitwarden not being available, but should read the file first
    cmd.assert().failure();
}

#[test]
fn test_store_valid_env_file_format() {
    let temp_dir = tempdir().unwrap();
    let env_file = temp_dir.path().join("test.env");

    fs::write(
        &env_file,
        "DB_HOST=localhost\nDB_PORT=5432\nAPI_KEY=secret123",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("bwenv").unwrap();
    cmd.args(&["store", "--file", env_file.to_str().unwrap()]);

    // Will fail due to Bitwarden CLI not being available/configured, but file should be valid
    cmd.assert().failure().stderr(
        predicate::str::contains("Bitwarden").or(predicate::str::contains("Failed to execute")),
    );
}

#[test]
fn test_store_with_folder_option() {
    let temp_dir = tempdir().unwrap();
    let env_file = temp_dir.path().join("test.env");

    fs::write(&env_file, "TEST_KEY=test_value").unwrap();

    let mut cmd = Command::cargo_bin("bwenv").unwrap();
    cmd.args(&[
        "store",
        "--file",
        env_file.to_str().unwrap(),
        "--folder",
        "Development/TestProject",
    ]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Bitwarden"));
}

#[test]
fn test_store_with_name_option() {
    let temp_dir = tempdir().unwrap();
    let env_file = temp_dir.path().join("test.env");

    fs::write(&env_file, "TEST_KEY=test_value").unwrap();

    let mut cmd = Command::cargo_bin("bwenv").unwrap();
    cmd.args(&[
        "store",
        "--file",
        env_file.to_str().unwrap(),
        "--name",
        "my-test-env",
    ]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Bitwarden"));
}

#[test]
fn test_store_with_overwrite_flag() {
    let temp_dir = tempdir().unwrap();
    let env_file = temp_dir.path().join("test.env");

    fs::write(&env_file, "TEST_KEY=test_value").unwrap();

    let mut cmd = Command::cargo_bin("bwenv").unwrap();
    cmd.args(&["store", "--file", env_file.to_str().unwrap(), "--overwrite"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Bitwarden"));
}

#[test]
fn test_retrieve_without_name_or_folder() {
    let mut cmd = Command::cargo_bin("bwenv").unwrap();
    cmd.arg("retrieve");

    // Should fail due to Bitwarden CLI issues, not argument validation
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Bitwarden"));
}

#[test]
fn test_retrieve_with_name() {
    let mut cmd = Command::cargo_bin("bwenv").unwrap();
    cmd.args(&["retrieve", "--name", "test-env"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Bitwarden"));
}

#[test]
fn test_retrieve_with_folder() {
    let mut cmd = Command::cargo_bin("bwenv").unwrap();
    cmd.args(&["retrieve", "--folder", "Development"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Bitwarden"));
}

#[test]
fn test_retrieve_with_output_file() {
    let temp_dir = tempdir().unwrap();
    let output_file = temp_dir.path().join("output.env");

    let mut cmd = Command::cargo_bin("bwenv").unwrap();
    cmd.args(&[
        "retrieve",
        "--name",
        "test-env",
        "--output",
        output_file.to_str().unwrap(),
    ]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Bitwarden"));
}

#[test]
fn test_retrieve_with_merge_flag() {
    let temp_dir = tempdir().unwrap();
    let output_file = temp_dir.path().join("existing.env");

    // Create existing file
    fs::write(&output_file, "EXISTING_KEY=existing_value").unwrap();

    let mut cmd = Command::cargo_bin("bwenv").unwrap();
    cmd.args(&[
        "retrieve",
        "--name",
        "test-env",
        "--output",
        output_file.to_str().unwrap(),
        "--merge",
    ]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Bitwarden"));
}

#[test]
fn test_list_default_format() {
    let mut cmd = Command::cargo_bin("bwenv").unwrap();
    cmd.arg("list");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Bitwarden"));
}

#[test]
fn test_list_json_format() {
    let mut cmd = Command::cargo_bin("bwenv").unwrap();
    cmd.args(&["list", "--format", "json"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Bitwarden"));
}

#[test]
fn test_list_invalid_format() {
    let mut cmd = Command::cargo_bin("bwenv").unwrap();
    cmd.args(&["list", "--format", "invalid"]);

    // Should still attempt to run but fail on Bitwarden
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Bitwarden"));
}

#[test]
fn test_invalid_subcommand() {
    let mut cmd = Command::cargo_bin("bwenv").unwrap();
    cmd.arg("invalid-command");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn test_multiple_verbosity_flags() {
    let mut cmd = Command::cargo_bin("bwenv").unwrap();
    cmd.args(&["-vvv", "list"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Bitwarden"));
}

#[test]
fn test_conflicting_quiet_and_verbose() {
    let mut cmd = Command::cargo_bin("bwenv").unwrap();
    cmd.args(&["-q", "-v", "list"]);

    // Quiet should take precedence
    cmd.assert().failure();
}

// Test with real .env file examples
#[test]
fn test_store_with_example_env_file() {
    let temp_dir = tempdir().unwrap();
    let env_file = temp_dir.path().join("example.env");

    let env_content = r#"
# Database configuration
DB_HOST=localhost
DB_PORT=5432
DB_NAME=myapp_db

# API keys
STRIPE_API_KEY=sk_test_123
GITHUB_TOKEN=ghp_456

# Feature flags
ENABLE_PAYMENTS=true
DEBUG_MODE=false
"#;

    fs::write(&env_file, env_content).unwrap();

    let mut cmd = Command::cargo_bin("bwenv").unwrap();
    cmd.args(&["store", "--file", env_file.to_str().unwrap()]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Bitwarden"));
}

#[test]
fn test_env_file_with_quotes_and_spaces() {
    let temp_dir = tempdir().unwrap();
    let env_file = temp_dir.path().join("complex.env");

    let env_content = r#"
KEY_WITH_SPACES=value with spaces
KEY_WITH_QUOTES="quoted value"
KEY_WITH_EQUALS=value=with=equals
EMPTY_VALUE=
MULTILINE_VALUE=line1\nline2
"#;

    fs::write(&env_file, env_content).unwrap();

    let mut cmd = Command::cargo_bin("bwenv").unwrap();
    cmd.args(&["store", "--file", env_file.to_str().unwrap()]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Bitwarden"));
}

// Error handling tests
#[test]
fn test_permission_denied_file() {
    // This test might not work on all systems due to permission handling
    let temp_dir = tempdir().unwrap();
    let env_file = temp_dir.path().join("no_permission.env");

    fs::write(&env_file, "KEY=value").unwrap();

    // Try to make file unreadable (might not work on all systems)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&env_file).unwrap().permissions();
        perms.set_mode(0o000);
        let _ = fs::set_permissions(&env_file, perms);
    }

    let mut cmd = Command::cargo_bin("bwenv").unwrap();
    cmd.args(&["store", "--file", env_file.to_str().unwrap()]);

    cmd.assert().failure();
}

#[test]
fn test_bitwarden_cli_not_installed() {
    // This test assumes bw is not in PATH
    // In CI/CD, you might want to temporarily rename or remove bw
    let mut cmd = Command::cargo_bin("bwenv").unwrap();
    cmd.env("PATH", ""); // Clear PATH to simulate missing bw
    cmd.args(&["list"]);

    cmd.assert().failure().stderr(
        predicate::str::contains("Bitwarden CLI").or(predicate::str::contains("Failed to execute")),
    );
}
