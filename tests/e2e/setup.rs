//! E2E Test Setup and Teardown
//!
//! Provides TestContext for managing test lifecycle with real Bitwarden instance.

use anyhow::{Context, Result, bail};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use chrono::Utc;

use bwenv::bitwarden::{SdkProvider, SecretsProvider};

pub type TestResult<T = ()> = Result<T>;

/// Test context managing Bitwarden project lifecycle
pub struct TestContext {
    /// Real Bitwarden SDK provider
    provider: SdkProvider,
    /// Test project name (unique per test run)
    project_name: String,
    /// Created project ID (for cleanup)
    project_id: Option<String>,
    /// Temporary directory for test files
    temp_dir: TempDir,
    /// Access token for authentication
    access_token: String,
}

impl TestContext {
    /// Create a new test context with unique project name
    pub async fn new() -> TestResult<Self> {
        let access_token = std::env::var("BITWARDEN_ACCESS_TOKEN")
            .context("BITWARDEN_ACCESS_TOKEN not set. Create .env.test with your token.")?;

        // Create unique project name with timestamp
        let timestamp = Utc::now().timestamp();
        let project_name = format!("bwenv-e2e-test-{}", timestamp);

        let provider = SdkProvider::new(access_token.clone())
            .await
            .context("Failed to initialize Bitwarden SDK")?;

        let temp_dir = tempfile::tempdir()
            .context("Failed to create temporary directory")?;

        Ok(Self {
            provider,
            project_name,
            project_id: None,
            temp_dir,
            access_token,
        })
    }

    /// Create test project in Bitwarden
    pub async fn setup_project(&mut self) -> TestResult<String> {
        // Note: SDK doesn't expose project creation yet
        // Users must manually create a project and set project name in env
        // For now, we'll use an existing project
        let project_name = std::env::var("BITWARDEN_TEST_PROJECT")
            .unwrap_or_else(|_| "E2E-Test".to_string());

        let project = self.provider.get_project_by_name(&project_name).await?
            .context(format!("Test project '{}' not found. Please create it manually in Bitwarden Secrets Manager.", project_name))?;

        self.project_id = Some(project.id.clone());
        self.project_name = project.name.clone();

        Ok(project.id)
    }

    /// Clean up test project (delete all secrets)
    pub async fn cleanup(&self) -> TestResult<()> {
        if let Some(project_id) = &self.project_id {
            // List all secrets in project
            let secrets = self.provider.list_secrets(project_id).await?;

            // Delete each secret
            for secret in secrets {
                if let Err(e) = self.provider.delete_secret(&secret.id).await {
                    eprintln!("Warning: Failed to delete secret {}: {}", secret.id, e);
                }
            }

            println!("Cleaned up {} test secrets from project", secrets.len());
        }

        Ok(())
    }

    /// Get provider reference
    pub fn provider(&self) -> &SdkProvider {
        &self.provider
    }

    /// Get project name
    pub fn project_name(&self) -> &str {
        &self.project_name
    }

    /// Get project ID (must call setup_project first)
    pub fn project_id(&self) -> TestResult<&str> {
        self.project_id.as_deref()
            .context("Project not set up. Call setup_project() first.")
    }

    /// Get temp directory path
    pub fn temp_dir(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Create a test .env file in temp directory
    pub fn create_test_env(&self, name: &str, vars: &HashMap<String, String>) -> TestResult<PathBuf> {
        let path = self.temp_dir.path().join(name);
        let mut content = String::new();

        content.push_str("# Test environment variables\n\n");
        for (key, value) in vars {
            content.push_str(&format!("{}={}\n", key, value));
        }

        fs::write(&path, content)
            .context(format!("Failed to write test env file: {}", name))?;

        Ok(path)
    }

    /// Read .env file and parse into HashMap
    pub fn read_env_file(&self, path: &Path) -> TestResult<HashMap<String, String>> {
        let content = fs::read_to_string(path)
            .context(format!("Failed to read .env file: {}", path.display()))?;

        let mut vars = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some(pos) = trimmed.find('=') {
                let key = trimmed[..pos].trim().to_string();
                let value = trimmed[pos + 1..].trim().to_string();
                vars.insert(key, value);
            }
        }

        Ok(vars)
    }

    /// Get access token for CLI commands
    pub fn access_token(&self) -> &str {
        &self.access_token
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        // Best effort cleanup on drop
        // Note: This is sync, so we can't await
        // Real cleanup should be done explicitly with cleanup() method
        eprintln!("TestContext dropped. Remember to call cleanup() explicitly for async cleanup.");
    }
}

/// Assert two .env files have equivalent content (ignoring order and comments)
pub fn assert_env_files_equivalent(path1: &Path, path2: &Path) -> TestResult<()> {
    let content1 = fs::read_to_string(path1)?;
    let content2 = fs::read_to_string(path2)?;

    let mut vars1 = HashMap::new();
    let mut vars2 = HashMap::new();

    for line in content1.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            if let Some(pos) = trimmed.find('=') {
                vars1.insert(trimmed[..pos].trim().to_string(), trimmed[pos + 1..].trim().to_string());
            }
        }
    }

    for line in content2.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            if let Some(pos) = trimmed.find('=') {
                vars2.insert(trimmed[..pos].trim().to_string(), trimmed[pos + 1..].trim().to_string());
            }
        }
    }

    if vars1 != vars2 {
        bail!(
            "Env files differ:\nFile 1: {:?}\nFile 2: {:?}",
            vars1,
            vars2
        );
    }

    Ok(())
}

/// Assert HashMap contains expected key-value pairs
pub fn assert_vars_contain(actual: &HashMap<String, String>, expected: &HashMap<String, String>) -> TestResult<()> {
    for (key, value) in expected {
        match actual.get(key) {
            Some(actual_value) => {
                if actual_value != value {
                    bail!("Key '{}' has value '{}' but expected '{}'", key, actual_value, value);
                }
            }
            None => {
                bail!("Missing expected key: '{}'", key);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_env_files_equivalent() {
        let temp_dir = tempfile::tempdir().unwrap();

        let file1 = temp_dir.path().join("test1.env");
        let file2 = temp_dir.path().join("test2.env");

        fs::write(&file1, "KEY1=value1\nKEY2=value2\n").unwrap();
        fs::write(&file2, "# Comment\nKEY2=value2\nKEY1=value1\n").unwrap();

        // Should be equivalent (different order, comments ignored)
        assert!(assert_env_files_equivalent(&file1, &file2).is_ok());
    }

    #[test]
    fn test_assert_vars_contain() {
        let mut actual = HashMap::new();
        actual.insert("KEY1".to_string(), "value1".to_string());
        actual.insert("KEY2".to_string(), "value2".to_string());

        let mut expected = HashMap::new();
        expected.insert("KEY1".to_string(), "value1".to_string());

        // Should pass (actual contains all expected)
        assert!(assert_vars_contain(&actual, &expected).is_ok());

        // Should fail (missing key)
        expected.insert("KEY3".to_string(), "value3".to_string());
        assert!(assert_vars_contain(&actual, &expected).is_err());
    }
}
