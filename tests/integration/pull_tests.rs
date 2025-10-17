//! Integration tests for pull command
//!
//! Tests pulling secrets from Bitwarden to .env files

use bwenv::bitwarden::SecretsProvider;
use bwenv::env::parser::{read_env_file, write_env_file};
use std::collections::HashMap;
use tempfile::tempdir;

mod common;
use common::{assert_env_files_equivalent, TestProject};

#[tokio::test]
async fn test_pull_retrieves_all_secrets() {
    let mut secrets = HashMap::new();
    secrets.insert("API_KEY".to_string(), "secret123".to_string());
    secrets.insert("DB_PASSWORD".to_string(), "dbpass456".to_string());
    secrets.insert("JWT_SECRET".to_string(), "jwt789".to_string());

    let project = TestProject::new("Test Project").with_secrets(secrets);
    let provider = project.provider();

    // Pull secrets
    let secret_map = provider.get_secrets_map(&project.project.id).await.unwrap();

    // Write to .env file
    let temp_dir = tempdir().unwrap();
    let env_path = temp_dir.path().join(".env");
    write_env_file(&env_path, &secret_map, false).unwrap();

    // Verify file was written correctly
    let read_back = read_env_file(&env_path).unwrap();
    assert_eq!(read_back.len(), 3);
    assert_eq!(read_back.get("API_KEY"), Some(&"secret123".to_string()));
    assert_eq!(read_back.get("DB_PASSWORD"), Some(&"dbpass456".to_string()));
    assert_eq!(read_back.get("JWT_SECRET"), Some(&"jwt789".to_string()));
}

#[tokio::test]
async fn test_pull_to_new_file() {
    let mut secrets = HashMap::new();
    secrets.insert("KEY1".to_string(), "value1".to_string());
    secrets.insert("KEY2".to_string(), "value2".to_string());

    let project = TestProject::new("Test Project").with_secrets(secrets);
    let provider = project.provider();

    let temp_dir = tempdir().unwrap();
    let env_path = temp_dir.path().join(".env");

    // Verify file doesn't exist
    assert!(!env_path.exists());

    // Pull and write
    let secret_map = provider.get_secrets_map(&project.project.id).await.unwrap();
    write_env_file(&env_path, &secret_map, false).unwrap();

    // Verify file was created
    assert!(env_path.exists());

    let content = std::fs::read_to_string(&env_path).unwrap();
    assert!(content.contains("KEY1=value1"));
    assert!(content.contains("KEY2=value2"));
}

#[tokio::test]
async fn test_pull_with_merge_preserves_local() {
    let mut secrets = HashMap::new();
    secrets.insert("REMOTE_KEY".to_string(), "remote_value".to_string());
    secrets.insert("SHARED_KEY".to_string(), "remote_shared".to_string());

    let project = TestProject::new("Test Project").with_secrets(secrets);
    let provider = project.provider();

    let temp_dir = tempdir().unwrap();
    let env_path = temp_dir.path().join(".env");

    // Create existing .env with local keys
    let mut local_vars = HashMap::new();
    local_vars.insert("LOCAL_KEY".to_string(), "local_value".to_string());
    local_vars.insert("SHARED_KEY".to_string(), "local_shared".to_string());
    write_env_file(&env_path, &local_vars, false).unwrap();

    // Pull with merge
    let secret_map = provider.get_secrets_map(&project.project.id).await.unwrap();
    write_env_file(&env_path, &secret_map, true).unwrap();

    // Verify merge result
    let result = read_env_file(&env_path).unwrap();
    assert_eq!(result.len(), 3);
    assert_eq!(result.get("LOCAL_KEY"), Some(&"local_value".to_string()));
    assert_eq!(result.get("REMOTE_KEY"), Some(&"remote_value".to_string()));
    // Remote should overwrite shared key
    assert_eq!(result.get("SHARED_KEY"), Some(&"remote_shared".to_string()));
}

#[tokio::test]
async fn test_pull_without_merge_overwrites_local() {
    let mut secrets = HashMap::new();
    secrets.insert("REMOTE_KEY".to_string(), "remote_value".to_string());

    let project = TestProject::new("Test Project").with_secrets(secrets);
    let provider = project.provider();

    let temp_dir = tempdir().unwrap();
    let env_path = temp_dir.path().join(".env");

    // Create existing .env
    let mut local_vars = HashMap::new();
    local_vars.insert("LOCAL_KEY".to_string(), "local_value".to_string());
    write_env_file(&env_path, &local_vars, false).unwrap();

    // Pull without merge
    let secret_map = provider.get_secrets_map(&project.project.id).await.unwrap();
    write_env_file(&env_path, &secret_map, false).unwrap();

    // Verify local key was removed
    let result = read_env_file(&env_path).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result.get("REMOTE_KEY"), Some(&"remote_value".to_string()));
    assert!(result.get("LOCAL_KEY").is_none());
}

#[tokio::test]
async fn test_pull_empty_project() {
    let project = TestProject::new("Empty Project");
    let provider = project.provider();

    let secret_map = provider.get_secrets_map(&project.project.id).await.unwrap();
    assert_eq!(secret_map.len(), 0);

    let temp_dir = tempdir().unwrap();
    let env_path = temp_dir.path().join(".env");
    write_env_file(&env_path, &secret_map, false).unwrap();

    // File should exist but only contain headers
    assert!(env_path.exists());
    let content = std::fs::read_to_string(&env_path).unwrap();
    assert!(content.contains("# Environment variables"));

    let vars = read_env_file(&env_path).unwrap();
    assert_eq!(vars.len(), 0);
}

#[tokio::test]
async fn test_pull_with_empty_values() {
    let mut secrets = HashMap::new();
    secrets.insert("KEY1".to_string(), "".to_string());
    secrets.insert("KEY2".to_string(), "value2".to_string());
    secrets.insert("KEY3".to_string(), "".to_string());

    let project = TestProject::new("Test Project").with_secrets(secrets);
    let provider = project.provider();

    let temp_dir = tempdir().unwrap();
    let env_path = temp_dir.path().join(".env");

    let secret_map = provider.get_secrets_map(&project.project.id).await.unwrap();
    write_env_file(&env_path, &secret_map, false).unwrap();

    let result = read_env_file(&env_path).unwrap();
    assert_eq!(result.get("KEY1"), Some(&"".to_string()));
    assert_eq!(result.get("KEY2"), Some(&"value2".to_string()));
    assert_eq!(result.get("KEY3"), Some(&"".to_string()));
}

#[tokio::test]
async fn test_pull_large_number_of_secrets() {
    let mut secrets = HashMap::new();
    for i in 0..100 {
        secrets.insert(format!("KEY_{}", i), format!("value_{}", i));
    }

    let project = TestProject::new("Test Project").with_secrets(secrets);
    let provider = project.provider();

    let temp_dir = tempdir().unwrap();
    let env_path = temp_dir.path().join(".env");

    let secret_map = provider.get_secrets_map(&project.project.id).await.unwrap();
    write_env_file(&env_path, &secret_map, false).unwrap();

    let result = read_env_file(&env_path).unwrap();
    assert_eq!(result.len(), 100);

    for i in 0..100 {
        assert_eq!(
            result.get(&format!("KEY_{}", i)),
            Some(&format!("value_{}", i))
        );
    }
}

#[tokio::test]
async fn test_pull_special_characters() {
    let mut secrets = HashMap::new();
    secrets.insert(
        "URL".to_string(),
        "https://example.com/path?query=value&foo=bar".to_string(),
    );
    secrets.insert(
        "JSON".to_string(),
        r#"{"key":"value","nested":{"a":1}}"#.to_string(),
    );
    secrets.insert("SPACES".to_string(), "value with spaces".to_string());

    let project = TestProject::new("Test Project").with_secrets(secrets);
    let provider = project.provider();

    let temp_dir = tempdir().unwrap();
    let env_path = temp_dir.path().join(".env");

    let secret_map = provider.get_secrets_map(&project.project.id).await.unwrap();
    write_env_file(&env_path, &secret_map, false).unwrap();

    let result = read_env_file(&env_path).unwrap();
    assert_eq!(
        result.get("URL"),
        Some(&"https://example.com/path?query=value&foo=bar".to_string())
    );
    assert!(result.get("JSON").is_some());
    assert_eq!(result.get("SPACES"), Some(&"value with spaces".to_string()));
}

#[tokio::test]
async fn test_pull_idempotency() {
    let mut secrets = HashMap::new();
    secrets.insert("API_KEY".to_string(), "secret123".to_string());

    let project = TestProject::new("Test Project").with_secrets(secrets);
    let provider = project.provider();

    let temp_dir = tempdir().unwrap();
    let env_path = temp_dir.path().join(".env");

    // Pull once
    let secret_map1 = provider.get_secrets_map(&project.project.id).await.unwrap();
    write_env_file(&env_path, &secret_map1, false).unwrap();

    let content1 = read_env_file(&env_path).unwrap();

    // Pull again
    let secret_map2 = provider.get_secrets_map(&project.project.id).await.unwrap();
    write_env_file(&env_path, &secret_map2, false).unwrap();

    let content2 = read_env_file(&env_path).unwrap();

    // Results should be identical
    assert_eq!(content1, content2);
}

#[tokio::test]
async fn test_pull_roundtrip_preserves_data() {
    let mut original = HashMap::new();
    original.insert("KEY1".to_string(), "value1".to_string());
    original.insert("KEY2".to_string(), "value2".to_string());
    original.insert("KEY3".to_string(), "value3".to_string());

    let project = TestProject::new("Test Project").with_secrets(original.clone());
    let provider = project.provider();

    let temp_dir = tempdir().unwrap();
    let env_path = temp_dir.path().join(".env");

    // Pull to file
    let secret_map = provider.get_secrets_map(&project.project.id).await.unwrap();
    write_env_file(&env_path, &secret_map, false).unwrap();

    // Read back
    let roundtrip = read_env_file(&env_path).unwrap();

    // Should be identical to original
    assert_eq!(original, roundtrip);
}
