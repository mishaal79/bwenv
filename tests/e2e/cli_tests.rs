//! CLI E2E Tests - Real Bitwarden Integration
//!
//! These tests execute the actual bwenv CLI binary against a real Bitwarden Secrets Manager instance.
//! They verify end-to-end functionality including authentication, API communication, and file operations.

use super::setup::{TestContext, TestResult, assert_env_files_equivalent, assert_vars_contain};
use std::collections::HashMap;
use std::process::Command;
use std::fs;

/// Get path to bwenv binary
fn bwenv_binary() -> String {
    std::env::var("BWENV_BINARY")
        .unwrap_or_else(|_| "./target/release/bwenv".to_string())
}

/// Execute bwenv CLI command with access token
fn run_bwenv(ctx: &TestContext, args: &[&str]) -> TestResult<std::process::Output> {
    let output = Command::new(bwenv_binary())
        .env("BITWARDEN_ACCESS_TOKEN", ctx.access_token())
        .args(args)
        .output()?;

    Ok(output)
}

// ============================================================================
// Push Command Tests
// ============================================================================

#[tokio::test]
async fn test_push_basic_secrets() -> TestResult<()> {
    let mut ctx = TestContext::new().await?;
    ctx.setup_project().await?;

    // Create test .env file
    let mut vars = HashMap::new();
    vars.insert("DB_HOST".to_string(), "localhost".to_string());
    vars.insert("DB_PORT".to_string(), "5432".to_string());
    vars.insert("API_KEY".to_string(), "test_key_123".to_string());

    let env_path = ctx.create_test_env("test.env", &vars)?;

    // Push to Bitwarden
    let output = run_bwenv(&ctx, &[
        "push",
        "--project", ctx.project_name(),
        "--input", env_path.to_str().unwrap(),
    ])?;

    assert!(output.status.success(), "Push command failed: {:?}", String::from_utf8_lossy(&output.stderr));

    // Verify secrets were created
    let secrets = ctx.provider().get_secrets_map(ctx.project_id()?).await?;
    assert_vars_contain(&secrets, &vars)?;

    ctx.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_push_overwrite_existing_secrets() -> TestResult<()> {
    let mut ctx = TestContext::new().await?;
    ctx.setup_project().await?;

    // Push initial secrets
    let mut vars1 = HashMap::new();
    vars1.insert("KEY1".to_string(), "initial_value".to_string());
    let env_path1 = ctx.create_test_env("initial.env", &vars1)?;

    run_bwenv(&ctx, &[
        "push",
        "--project", ctx.project_name(),
        "--input", env_path1.to_str().unwrap(),
    ])?;

    // Push updated secrets with --overwrite
    let mut vars2 = HashMap::new();
    vars2.insert("KEY1".to_string(), "updated_value".to_string());
    vars2.insert("KEY2".to_string(), "new_value".to_string());
    let env_path2 = ctx.create_test_env("updated.env", &vars2)?;

    let output = run_bwenv(&ctx, &[
        "push",
        "--project", ctx.project_name(),
        "--input", env_path2.to_str().unwrap(),
        "--overwrite",
    ])?;

    assert!(output.status.success(), "Push with overwrite failed: {:?}", String::from_utf8_lossy(&output.stderr));

    // Verify updated values
    let secrets = ctx.provider().get_secrets_map(ctx.project_id()?).await?;
    assert_eq!(secrets.get("KEY1").unwrap(), "updated_value");
    assert_eq!(secrets.get("KEY2").unwrap(), "new_value");

    ctx.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_push_empty_env_file() -> TestResult<()> {
    let mut ctx = TestContext::new().await?;
    ctx.setup_project().await?;

    let vars = HashMap::new();
    let env_path = ctx.create_test_env("empty.env", &vars)?;

    let output = run_bwenv(&ctx, &[
        "push",
        "--project", ctx.project_name(),
        "--input", env_path.to_str().unwrap(),
    ])?;

    // Should succeed but report 0 secrets
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0") || stdout.contains("No secrets"));

    ctx.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_push_nonexistent_file() -> TestResult<()> {
    let ctx = TestContext::new().await?;

    let output = run_bwenv(&ctx, &[
        "push",
        "--project", "test",
        "--input", "/nonexistent/file.env",
    ])?;

    assert!(!output.status.success(), "Should fail with nonexistent file");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found") || stderr.contains("No such file"));

    Ok(())
}

#[tokio::test]
async fn test_push_nonexistent_project() -> TestResult<()> {
    let ctx = TestContext::new().await?;

    let mut vars = HashMap::new();
    vars.insert("KEY".to_string(), "value".to_string());
    let env_path = ctx.create_test_env("test.env", &vars)?;

    let output = run_bwenv(&ctx, &[
        "push",
        "--project", "nonexistent-project-999",
        "--input", env_path.to_str().unwrap(),
    ])?;

    assert!(!output.status.success(), "Should fail with nonexistent project");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found") || stderr.contains("Project"));

    Ok(())
}

// ============================================================================
// Pull Command Tests
// ============================================================================

#[tokio::test]
async fn test_pull_basic_secrets() -> TestResult<()> {
    let mut ctx = TestContext::new().await?;
    ctx.setup_project().await?;

    // First push some secrets
    let mut vars = HashMap::new();
    vars.insert("DATABASE_URL".to_string(), "postgres://localhost/test".to_string());
    vars.insert("REDIS_URL".to_string(), "redis://localhost:6379".to_string());

    let push_path = ctx.create_test_env("push.env", &vars)?;
    run_bwenv(&ctx, &[
        "push",
        "--project", ctx.project_name(),
        "--input", push_path.to_str().unwrap(),
    ])?;

    // Now pull to a new file
    let pull_path = ctx.temp_dir().join("pulled.env");
    let output = run_bwenv(&ctx, &[
        "pull",
        "--project", ctx.project_name(),
        "--output", pull_path.to_str().unwrap(),
    ])?;

    assert!(output.status.success(), "Pull command failed: {:?}", String::from_utf8_lossy(&output.stderr));
    assert!(pull_path.exists(), "Pulled .env file should exist");

    // Verify content
    let pulled_vars = ctx.read_env_file(&pull_path)?;
    assert_vars_contain(&pulled_vars, &vars)?;

    ctx.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_pull_force_overwrite() -> TestResult<()> {
    let mut ctx = TestContext::new().await?;
    ctx.setup_project().await?;

    // Push secrets
    let mut vars = HashMap::new();
    vars.insert("KEY".to_string(), "from_bitwarden".to_string());
    let push_path = ctx.create_test_env("push.env", &vars)?;
    run_bwenv(&ctx, &["push", "--project", ctx.project_name(), "--input", push_path.to_str().unwrap()])?;

    // Create existing file
    let pull_path = ctx.temp_dir().join("existing.env");
    fs::write(&pull_path, "KEY=old_value\n")?;

    // Pull without --force should fail
    let output = run_bwenv(&ctx, &[
        "pull",
        "--project", ctx.project_name(),
        "--output", pull_path.to_str().unwrap(),
    ])?;
    assert!(!output.status.success(), "Should fail without --force");

    // Pull with --force should succeed
    let output = run_bwenv(&ctx, &[
        "pull",
        "--project", ctx.project_name(),
        "--output", pull_path.to_str().unwrap(),
        "--force",
    ])?;
    assert!(output.status.success(), "Pull with --force should succeed");

    // Verify content was overwritten
    let pulled_vars = ctx.read_env_file(&pull_path)?;
    assert_eq!(pulled_vars.get("KEY").unwrap(), "from_bitwarden");

    ctx.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_pull_empty_project() -> TestResult<()> {
    let mut ctx = TestContext::new().await?;
    ctx.setup_project().await?;
    // Don't push any secrets - project is empty

    let pull_path = ctx.temp_dir().join("empty_pull.env");
    let output = run_bwenv(&ctx, &[
        "pull",
        "--project", ctx.project_name(),
        "--output", pull_path.to_str().unwrap(),
    ])?;

    // Should succeed but report no secrets
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No secrets") || stdout.contains("0"));

    ctx.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_pull_nonexistent_project() -> TestResult<()> {
    let ctx = TestContext::new().await?;

    let pull_path = ctx.temp_dir().join("pull.env");
    let output = run_bwenv(&ctx, &[
        "pull",
        "--project", "nonexistent-project-xyz",
        "--output", pull_path.to_str().unwrap(),
    ])?;

    assert!(!output.status.success(), "Should fail with nonexistent project");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found") || stderr.contains("Project"));

    Ok(())
}

// ============================================================================
// Roundtrip Tests (Data Integrity)
// ============================================================================

#[tokio::test]
async fn test_roundtrip_push_pull_integrity() -> TestResult<()> {
    let mut ctx = TestContext::new().await?;
    ctx.setup_project().await?;

    // Create complex test data
    let mut vars = HashMap::new();
    vars.insert("DATABASE_URL".to_string(), "postgresql://user:pass@host:5432/db".to_string());
    vars.insert("JWT_SECRET".to_string(), "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9".to_string());
    vars.insert("API_ENDPOINTS".to_string(), "https://api1.com,https://api2.com".to_string());
    vars.insert("SPECIAL_CHARS".to_string(), "!@#$%^&*()_+-=[]{}|;:,.<>?".to_string());
    vars.insert("EMPTY_VALUE".to_string(), "".to_string());

    let original_path = ctx.create_test_env("original.env", &vars)?;

    // Push
    let push_output = run_bwenv(&ctx, &[
        "push",
        "--project", ctx.project_name(),
        "--input", original_path.to_str().unwrap(),
    ])?;
    assert!(push_output.status.success(), "Push failed");

    // Pull
    let pulled_path = ctx.temp_dir().join("pulled.env");
    let pull_output = run_bwenv(&ctx, &[
        "pull",
        "--project", ctx.project_name(),
        "--output", pulled_path.to_str().unwrap(),
    ])?;
    assert!(pull_output.status.success(), "Pull failed");

    // Verify data integrity
    assert_env_files_equivalent(&original_path, &pulled_path)?;

    ctx.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_roundtrip_update_workflow() -> TestResult<()> {
    let mut ctx = TestContext::new().await?;
    ctx.setup_project().await?;

    // Initial push
    let mut vars1 = HashMap::new();
    vars1.insert("VERSION".to_string(), "1.0.0".to_string());
    vars1.insert("FEATURE_A".to_string(), "enabled".to_string());
    let env1 = ctx.create_test_env("v1.env", &vars1)?;
    run_bwenv(&ctx, &["push", "--project", ctx.project_name(), "--input", env1.to_str().unwrap()])?;

    // Update and push again
    let mut vars2 = HashMap::new();
    vars2.insert("VERSION".to_string(), "2.0.0".to_string());
    vars2.insert("FEATURE_A".to_string(), "enabled".to_string());
    vars2.insert("FEATURE_B".to_string(), "beta".to_string());
    let env2 = ctx.create_test_env("v2.env", &vars2)?;
    run_bwenv(&ctx, &["push", "--project", ctx.project_name(), "--input", env2.to_str().unwrap(), "--overwrite"])?;

    // Pull and verify latest state
    let pulled = ctx.temp_dir().join("latest.env");
    run_bwenv(&ctx, &["pull", "--project", ctx.project_name(), "--output", pulled.to_str().unwrap()])?;

    let pulled_vars = ctx.read_env_file(&pulled)?;
    assert_eq!(pulled_vars.get("VERSION").unwrap(), "2.0.0");
    assert_eq!(pulled_vars.get("FEATURE_B").unwrap(), "beta");

    ctx.cleanup().await?;
    Ok(())
}

// ============================================================================
// List Command Tests
// ============================================================================

#[tokio::test]
async fn test_list_projects() -> TestResult<()> {
    let ctx = TestContext::new().await?;

    let output = run_bwenv(&ctx, &["list"])?;
    assert!(output.status.success(), "List command failed: {:?}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show at least some output (projects or "no projects")
    assert!(!stdout.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_list_specific_project() -> TestResult<()> {
    let mut ctx = TestContext::new().await?;
    ctx.setup_project().await?;

    // Push some secrets first
    let mut vars = HashMap::new();
    vars.insert("KEY1".to_string(), "value1".to_string());
    vars.insert("KEY2".to_string(), "value2".to_string());
    let env_path = ctx.create_test_env("test.env", &vars)?;
    run_bwenv(&ctx, &["push", "--project", ctx.project_name(), "--input", env_path.to_str().unwrap()])?;

    let output = run_bwenv(&ctx, &["list", "--project", ctx.project_name()])?;
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("KEY1") || stdout.contains("2")); // Either shows keys or count

    ctx.cleanup().await?;
    Ok(())
}

// ============================================================================
// Validate Command Tests
// ============================================================================

#[tokio::test]
async fn test_validate_valid_env_file() -> TestResult<()> {
    let ctx = TestContext::new().await?;

    let mut vars = HashMap::new();
    vars.insert("VALID_KEY".to_string(), "valid_value".to_string());
    let env_path = ctx.create_test_env("valid.env", &vars)?;

    let output = run_bwenv(&ctx, &["validate", env_path.to_str().unwrap()])?;
    assert!(output.status.success(), "Validate should succeed for valid file");

    Ok(())
}

#[tokio::test]
async fn test_validate_invalid_env_file() -> TestResult<()> {
    let ctx = TestContext::new().await?;

    let invalid_path = ctx.temp_dir().join("invalid.env");
    fs::write(&invalid_path, "INVALID LINE WITHOUT EQUALS\nVALID=value\n")?;

    let output = run_bwenv(&ctx, &["validate", invalid_path.to_str().unwrap()])?;
    assert!(!output.status.success(), "Validate should fail for invalid file");

    Ok(())
}

#[tokio::test]
async fn test_validate_nonexistent_file() -> TestResult<()> {
    let ctx = TestContext::new().await?;

    let output = run_bwenv(&ctx, &["validate", "/nonexistent/file.env"])?;
    assert!(!output.status.success(), "Validate should fail for nonexistent file");

    Ok(())
}

// ============================================================================
// Error Scenario Tests
// ============================================================================

#[tokio::test]
async fn test_invalid_access_token() -> TestResult<()> {
    let output = Command::new(bwenv_binary())
        .env("BITWARDEN_ACCESS_TOKEN", "invalid.token.here")
        .args(&["list"])
        .output()?;

    assert!(!output.status.success(), "Should fail with invalid token");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("auth") || stderr.contains("token") || stderr.contains("failed"));

    Ok(())
}

#[tokio::test]
async fn test_missing_access_token() -> TestResult<()> {
    let output = Command::new(bwenv_binary())
        .env_remove("BITWARDEN_ACCESS_TOKEN")
        .args(&["list"])
        .output()?;

    assert!(!output.status.success(), "Should fail without access token");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("BITWARDEN_ACCESS_TOKEN") || stderr.contains("token"));

    Ok(())
}
