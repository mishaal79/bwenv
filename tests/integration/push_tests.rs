//! Integration tests for push command
//!
//! Tests pushing .env files to Bitwarden Secrets Manager

use bwenv::bitwarden::{MockProvider, SecretsProvider};
use bwenv::env::parser::{read_env_file, write_env_file};
use std::collections::HashMap;
use tempfile::tempdir;

mod common;
use common::{EnvFileBuilder, TestProject};

#[tokio::test]
async fn test_push_creates_new_secrets() {
    let project = TestProject::new("Test Project");
    let provider = project.provider();

    // Create a .env file with secrets
    let (env_path, _temp_dir) = EnvFileBuilder::new()
        .entry("API_KEY", "secret123")
        .entry("DB_PASSWORD", "dbpass456")
        .entry("JWT_SECRET", "jwt789")
        .build_temp()
        .unwrap();

    // Read the env file
    let env_vars = read_env_file(&env_path).unwrap();

    // Push secrets to provider
    let results = provider
        .sync_secrets(&project.project.id, &env_vars, false)
        .await
        .unwrap();

    assert_eq!(results.len(), 3);

    // Verify secrets were created
    let secrets = provider.list_secrets(&project.project.id).await.unwrap();
    assert_eq!(secrets.len(), 3);

    let secret_map = provider.get_secrets_map(&project.project.id).await.unwrap();
    assert_eq!(secret_map.get("API_KEY"), Some(&"secret123".to_string()));
    assert_eq!(secret_map.get("DB_PASSWORD"), Some(&"dbpass456".to_string()));
    assert_eq!(secret_map.get("JWT_SECRET"), Some(&"jwt789".to_string()));
}

#[tokio::test]
async fn test_push_with_overwrite_updates_existing() {
    let mut secrets_map = HashMap::new();
    secrets_map.insert("API_KEY".to_string(), "old_value".to_string());
    secrets_map.insert("DB_PASSWORD".to_string(), "old_pass".to_string());

    let project = TestProject::new("Test Project").with_secrets(secrets_map);
    let provider = project.provider();

    // Create a .env file with updated values
    let (env_path, _temp_dir) = EnvFileBuilder::new()
        .entry("API_KEY", "new_value")
        .entry("DB_PASSWORD", "new_pass")
        .entry("NEW_KEY", "new_secret")
        .build_temp()
        .unwrap();

    let env_vars = read_env_file(&env_path).unwrap();

    // Push with overwrite
    provider
        .sync_secrets(&project.project.id, &env_vars, true)
        .await
        .unwrap();

    // Verify secrets were updated
    let secret_map = provider.get_secrets_map(&project.project.id).await.unwrap();
    assert_eq!(secret_map.get("API_KEY"), Some(&"new_value".to_string()));
    assert_eq!(secret_map.get("DB_PASSWORD"), Some(&"new_pass".to_string()));
    assert_eq!(secret_map.get("NEW_KEY"), Some(&"new_secret".to_string()));
    assert_eq!(secret_map.len(), 3);
}

#[tokio::test]
async fn test_push_without_overwrite_preserves_existing() {
    let mut secrets_map = HashMap::new();
    secrets_map.insert("API_KEY".to_string(), "old_value".to_string());

    let project = TestProject::new("Test Project").with_secrets(secrets_map);
    let provider = project.provider();

    // Create a .env file with conflicting value
    let (env_path, _temp_dir) = EnvFileBuilder::new()
        .entry("API_KEY", "new_value")
        .entry("NEW_KEY", "new_secret")
        .build_temp()
        .unwrap();

    let env_vars = read_env_file(&env_path).unwrap();

    // Push without overwrite
    provider
        .sync_secrets(&project.project.id, &env_vars, false)
        .await
        .unwrap();

    // Verify existing secret was preserved
    let secret_map = provider.get_secrets_map(&project.project.id).await.unwrap();
    assert_eq!(secret_map.get("API_KEY"), Some(&"old_value".to_string()));
    assert_eq!(secret_map.get("NEW_KEY"), Some(&"new_secret".to_string()));
}

#[tokio::test]
async fn test_push_empty_file() {
    let project = TestProject::new("Test Project");
    let provider = project.provider();

    let (env_path, _temp_dir) = EnvFileBuilder::new()
        .with_header(false)
        .build_temp()
        .unwrap();

    let env_vars = read_env_file(&env_path).unwrap();

    let results = provider
        .sync_secrets(&project.project.id, &env_vars, false)
        .await
        .unwrap();

    assert_eq!(results.len(), 0);

    let secrets = provider.list_secrets(&project.project.id).await.unwrap();
    assert_eq!(secrets.len(), 0);
}

#[tokio::test]
async fn test_push_with_empty_values() {
    let project = TestProject::new("Test Project");
    let provider = project.provider();

    let (env_path, _temp_dir) = EnvFileBuilder::new()
        .entry("KEY1", "")
        .entry("KEY2", "value2")
        .entry("KEY3", "")
        .build_temp()
        .unwrap();

    let env_vars = read_env_file(&env_path).unwrap();

    provider
        .sync_secrets(&project.project.id, &env_vars, false)
        .await
        .unwrap();

    let secret_map = provider.get_secrets_map(&project.project.id).await.unwrap();
    assert_eq!(secret_map.get("KEY1"), Some(&"".to_string()));
    assert_eq!(secret_map.get("KEY2"), Some(&"value2".to_string()));
    assert_eq!(secret_map.get("KEY3"), Some(&"".to_string()));
}

#[tokio::test]
async fn test_push_large_number_of_secrets() {
    let project = TestProject::new("Test Project");
    let provider = project.provider();

    let mut builder = EnvFileBuilder::new();
    for i in 0..100 {
        builder = builder.entry(format!("KEY_{}", i), format!("value_{}", i));
    }

    let (env_path, _temp_dir) = builder.build_temp().unwrap();
    let env_vars = read_env_file(&env_path).unwrap();

    let results = provider
        .sync_secrets(&project.project.id, &env_vars, false)
        .await
        .unwrap();

    assert_eq!(results.len(), 100);

    let secrets = provider.list_secrets(&project.project.id).await.unwrap();
    assert_eq!(secrets.len(), 100);
}

#[tokio::test]
async fn test_push_special_characters_in_values() {
    let project = TestProject::new("Test Project");
    let provider = project.provider();

    let (env_path, _temp_dir) = EnvFileBuilder::new()
        .entry("URL", "https://example.com/path?query=value&foo=bar")
        .entry("JSON", r#"{"key":"value","nested":{"a":1}}"#)
        .entry("SPACES", "value with spaces")
        .entry("QUOTES", r#"value with "quotes""#)
        .build_temp()
        .unwrap();

    let env_vars = read_env_file(&env_path).unwrap();

    provider
        .sync_secrets(&project.project.id, &env_vars, false)
        .await
        .unwrap();

    let secret_map = provider.get_secrets_map(&project.project.id).await.unwrap();
    assert_eq!(
        secret_map.get("URL"),
        Some(&"https://example.com/path?query=value&foo=bar".to_string())
    );
    assert!(secret_map.get("JSON").is_some());
    assert_eq!(secret_map.get("SPACES"), Some(&"value with spaces".to_string()));
}

#[tokio::test]
async fn test_push_idempotency() {
    let project = TestProject::new("Test Project");
    let provider = project.provider();

    let (env_path, _temp_dir) = EnvFileBuilder::new()
        .entry("API_KEY", "secret123")
        .build_temp()
        .unwrap();

    let env_vars = read_env_file(&env_path).unwrap();

    // Push once
    provider
        .sync_secrets(&project.project.id, &env_vars, true)
        .await
        .unwrap();

    // Push again with same data
    provider
        .sync_secrets(&project.project.id, &env_vars, true)
        .await
        .unwrap();

    // Should still only have one secret
    let secrets = provider.list_secrets(&project.project.id).await.unwrap();
    assert_eq!(secrets.len(), 1);

    let secret_map = provider.get_secrets_map(&project.project.id).await.unwrap();
    assert_eq!(secret_map.get("API_KEY"), Some(&"secret123".to_string()));
}

#[tokio::test]
async fn test_push_to_nonexistent_project() {
    let provider = MockProvider::new();

    let mut env_vars = HashMap::new();
    env_vars.insert("KEY1".to_string(), "value1".to_string());

    let result = provider
        .sync_secrets("nonexistent_project", &env_vars, false)
        .await;

    assert!(result.is_err());
}
