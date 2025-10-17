//! Integration tests for status command
//!
//! Tests detecting drift between local .env and remote secrets

use bwenv::bitwarden::SecretsProvider;
use bwenv::env::parser::read_env_file;
use std::collections::HashMap;

mod common;
use common::{EnvFileBuilder, TestProject};

#[tokio::test]
async fn test_status_no_drift_when_synced() {
    let mut secrets = HashMap::new();
    secrets.insert("API_KEY".to_string(), "secret123".to_string());
    secrets.insert("DB_PASSWORD".to_string(), "dbpass456".to_string());

    let project = TestProject::new("Test Project").with_secrets(secrets.clone());
    let provider = project.provider();

    // Create local .env with same content
    let (env_path, _temp_dir) = EnvFileBuilder::new()
        .entries(secrets.clone())
        .build_temp()
        .unwrap();

    let local = read_env_file(&env_path).unwrap();
    let remote = provider.get_secrets_map(&project.project.id).await.unwrap();

    // Should be identical
    assert_eq!(local, remote);
}

#[tokio::test]
async fn test_status_detects_local_additions() {
    let mut remote_secrets = HashMap::new();
    remote_secrets.insert("API_KEY".to_string(), "secret123".to_string());

    let project = TestProject::new("Test Project").with_secrets(remote_secrets);
    let provider = project.provider();

    // Local has additional key
    let (env_path, _temp_dir) = EnvFileBuilder::new()
        .entry("API_KEY", "secret123")
        .entry("LOCAL_ONLY", "local_value")
        .build_temp()
        .unwrap();

    let local = read_env_file(&env_path).unwrap();
    let remote = provider.get_secrets_map(&project.project.id).await.unwrap();

    // Find keys only in local
    let local_only: Vec<_> = local
        .keys()
        .filter(|k| !remote.contains_key(*k))
        .collect();

    assert_eq!(local_only.len(), 1);
    assert!(local_only.contains(&&"LOCAL_ONLY".to_string()));
}

#[tokio::test]
async fn test_status_detects_remote_additions() {
    let mut remote_secrets = HashMap::new();
    remote_secrets.insert("API_KEY".to_string(), "secret123".to_string());
    remote_secrets.insert("REMOTE_ONLY".to_string(), "remote_value".to_string());

    let project = TestProject::new("Test Project").with_secrets(remote_secrets);
    let provider = project.provider();

    // Local missing one key
    let (env_path, _temp_dir) = EnvFileBuilder::new()
        .entry("API_KEY", "secret123")
        .build_temp()
        .unwrap();

    let local = read_env_file(&env_path).unwrap();
    let remote = provider.get_secrets_map(&project.project.id).await.unwrap();

    // Find keys only in remote
    let remote_only: Vec<_> = remote
        .keys()
        .filter(|k| !local.contains_key(*k))
        .collect();

    assert_eq!(remote_only.len(), 1);
    assert!(remote_only.contains(&&"REMOTE_ONLY".to_string()));
}

#[tokio::test]
async fn test_status_detects_value_changes() {
    let mut remote_secrets = HashMap::new();
    remote_secrets.insert("API_KEY".to_string(), "remote_value".to_string());
    remote_secrets.insert("DB_HOST".to_string(), "localhost".to_string());

    let project = TestProject::new("Test Project").with_secrets(remote_secrets);
    let provider = project.provider();

    // Local has different value for API_KEY
    let (env_path, _temp_dir) = EnvFileBuilder::new()
        .entry("API_KEY", "local_value")
        .entry("DB_HOST", "localhost")
        .build_temp()
        .unwrap();

    let local = read_env_file(&env_path).unwrap();
    let remote = provider.get_secrets_map(&project.project.id).await.unwrap();

    // Find keys with different values
    let modified: Vec<_> = local
        .keys()
        .filter(|k| remote.get(*k) != local.get(*k))
        .collect();

    assert_eq!(modified.len(), 1);
    assert!(modified.contains(&&"API_KEY".to_string()));
}

#[tokio::test]
async fn test_status_empty_local() {
    let mut remote_secrets = HashMap::new();
    remote_secrets.insert("API_KEY".to_string(), "secret123".to_string());

    let project = TestProject::new("Test Project").with_secrets(remote_secrets);
    let provider = project.provider();

    // Empty local file
    let (env_path, _temp_dir) = EnvFileBuilder::new()
        .with_header(false)
        .build_temp()
        .unwrap();

    let local = read_env_file(&env_path).unwrap();
    let remote = provider.get_secrets_map(&project.project.id).await.unwrap();

    assert_eq!(local.len(), 0);
    assert_eq!(remote.len(), 1);

    let remote_only: Vec<_> = remote.keys().collect();
    assert_eq!(remote_only.len(), 1);
}

#[tokio::test]
async fn test_status_empty_remote() {
    let project = TestProject::new("Empty Project");
    let provider = project.provider();

    let (env_path, _temp_dir) = EnvFileBuilder::new()
        .entry("LOCAL_KEY", "local_value")
        .build_temp()
        .unwrap();

    let local = read_env_file(&env_path).unwrap();
    let remote = provider.get_secrets_map(&project.project.id).await.unwrap();

    assert_eq!(local.len(), 1);
    assert_eq!(remote.len(), 0);

    let local_only: Vec<_> = local.keys().collect();
    assert_eq!(local_only.len(), 1);
}

#[tokio::test]
async fn test_status_both_empty() {
    let project = TestProject::new("Empty Project");
    let provider = project.provider();

    let (env_path, _temp_dir) = EnvFileBuilder::new()
        .with_header(false)
        .build_temp()
        .unwrap();

    let local = read_env_file(&env_path).unwrap();
    let remote = provider.get_secrets_map(&project.project.id).await.unwrap();

    assert_eq!(local.len(), 0);
    assert_eq!(remote.len(), 0);
}

#[tokio::test]
async fn test_status_complex_drift() {
    let mut remote_secrets = HashMap::new();
    remote_secrets.insert("SHARED_SAME".to_string(), "same_value".to_string());
    remote_secrets.insert("SHARED_DIFF".to_string(), "remote_value".to_string());
    remote_secrets.insert("REMOTE_ONLY".to_string(), "remote_only".to_string());

    let project = TestProject::new("Test Project").with_secrets(remote_secrets);
    let provider = project.provider();

    let (env_path, _temp_dir) = EnvFileBuilder::new()
        .entry("SHARED_SAME", "same_value")
        .entry("SHARED_DIFF", "local_value")
        .entry("LOCAL_ONLY", "local_only")
        .build_temp()
        .unwrap();

    let local = read_env_file(&env_path).unwrap();
    let remote = provider.get_secrets_map(&project.project.id).await.unwrap();

    // Find differences
    let local_only: Vec<_> = local
        .keys()
        .filter(|k| !remote.contains_key(*k))
        .collect();

    let remote_only: Vec<_> = remote
        .keys()
        .filter(|k| !local.contains_key(*k))
        .collect();

    let modified: Vec<_> = local
        .keys()
        .filter(|k| remote.contains_key(*k) && remote.get(*k) != local.get(*k))
        .collect();

    assert_eq!(local_only.len(), 1);
    assert!(local_only.contains(&&"LOCAL_ONLY".to_string()));

    assert_eq!(remote_only.len(), 1);
    assert!(remote_only.contains(&&"REMOTE_ONLY".to_string()));

    assert_eq!(modified.len(), 1);
    assert!(modified.contains(&&"SHARED_DIFF".to_string()));
}

#[tokio::test]
async fn test_status_case_sensitivity() {
    let mut remote_secrets = HashMap::new();
    remote_secrets.insert("API_KEY".to_string(), "value".to_string());

    let project = TestProject::new("Test Project").with_secrets(remote_secrets);
    let provider = project.provider();

    let (env_path, _temp_dir) = EnvFileBuilder::new()
        .entry("api_key", "value") // Different case
        .build_temp()
        .unwrap();

    let local = read_env_file(&env_path).unwrap();
    let remote = provider.get_secrets_map(&project.project.id).await.unwrap();

    // Keys should be treated as different (case-sensitive)
    assert!(!local.contains_key("API_KEY"));
    assert!(!remote.contains_key("api_key"));
}

#[tokio::test]
async fn test_status_whitespace_in_values() {
    let mut remote_secrets = HashMap::new();
    remote_secrets.insert("KEY1".to_string(), "value".to_string());
    remote_secrets.insert("KEY2".to_string(), " value ".to_string());

    let project = TestProject::new("Test Project").with_secrets(remote_secrets);
    let provider = project.provider();

    let (env_path, _temp_dir) = EnvFileBuilder::new()
        .entry("KEY1", " value ")
        .entry("KEY2", "value")
        .build_temp()
        .unwrap();

    let local = read_env_file(&env_path).unwrap();
    let remote = provider.get_secrets_map(&project.project.id).await.unwrap();

    // Both should show as modified due to whitespace differences
    let modified: Vec<_> = local
        .keys()
        .filter(|k| remote.contains_key(*k) && remote.get(*k) != local.get(*k))
        .collect();

    assert_eq!(modified.len(), 2);
}
