//! Mock SecretsProvider implementation for testing
//!
//! In-memory mock implementation for deterministic testing

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::provider::{Project, Secret, SecretsProvider};
use crate::{AppError, Result};

/// Mock implementation of SecretsProvider for testing
#[derive(Clone)]
pub struct MockProvider {
    state: Arc<Mutex<MockState>>,
}

#[derive(Default)]
struct MockState {
    projects: HashMap<String, Project>,
    secrets: HashMap<String, Secret>,
    next_secret_id: usize,
    next_project_id: usize,
}

impl MockProvider {
    /// Create a new empty mock provider
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(MockState::default())),
        }
    }

    /// Create a mock provider with predefined projects and secrets
    pub fn with_data(projects: Vec<Project>, secrets: Vec<Secret>) -> Self {
        let mut state = MockState::default();

        for project in projects {
            state.projects.insert(project.id.clone(), project);
        }

        for secret in secrets {
            state.secrets.insert(secret.id.clone(), secret);
        }

        state.next_secret_id = state.secrets.len() + 1;
        state.next_project_id = state.projects.len() + 1;

        Self {
            state: Arc::new(Mutex::new(state)),
        }
    }

    /// Add a project to the mock provider
    pub fn add_project(&self, project: Project) {
        let mut state = self.state.lock().unwrap();
        state.projects.insert(project.id.clone(), project);
    }

    /// Add a secret to the mock provider
    pub fn add_secret(&self, secret: Secret) {
        let mut state = self.state.lock().unwrap();
        state.secrets.insert(secret.id.clone(), secret);
    }

    /// Get all secrets (for testing purposes)
    pub fn get_all_secrets(&self) -> Vec<Secret> {
        let state = self.state.lock().unwrap();
        state.secrets.values().cloned().collect()
    }

    /// Clear all data
    pub fn clear(&self) {
        let mut state = self.state.lock().unwrap();
        state.projects.clear();
        state.secrets.clear();
        state.next_secret_id = 1;
        state.next_project_id = 1;
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SecretsProvider for MockProvider {
    async fn list_projects(&self) -> Result<Vec<Project>> {
        let state = self.state.lock().unwrap();
        Ok(state.projects.values().cloned().collect())
    }

    async fn get_project(&self, project_id: &str) -> Result<Option<Project>> {
        let state = self.state.lock().unwrap();
        Ok(state.projects.get(project_id).cloned())
    }

    async fn get_project_by_name(&self, name: &str) -> Result<Option<Project>> {
        let state = self.state.lock().unwrap();
        Ok(state.projects.values().find(|p| p.name == name).cloned())
    }

    async fn list_secrets(&self, project_id: &str) -> Result<Vec<Secret>> {
        let state = self.state.lock().unwrap();
        Ok(state
            .secrets
            .values()
            .filter(|s| s.project_id == project_id)
            .cloned()
            .collect())
    }

    async fn get_secret(&self, secret_id: &str) -> Result<Option<Secret>> {
        let state = self.state.lock().unwrap();
        Ok(state.secrets.get(secret_id).cloned())
    }

    async fn create_secret(
        &self,
        project_id: &str,
        key: &str,
        value: &str,
        note: Option<&str>,
    ) -> Result<Secret> {
        let mut state = self.state.lock().unwrap();

        // Verify project exists
        if !state.projects.contains_key(project_id) {
            return Err(AppError::ItemNotFound(format!(
                "Project not found: {}",
                project_id
            )));
        }

        // Check for duplicate key in the same project
        let duplicate = state
            .secrets
            .values()
            .any(|s| s.project_id == project_id && s.key == key);

        if duplicate {
            return Err(AppError::InvalidArguments(format!(
                "Secret with key '{}' already exists in project",
                key
            )));
        }

        let secret_id = format!("mock_secret_{}", state.next_secret_id);
        state.next_secret_id += 1;

        let secret = Secret {
            id: secret_id.clone(),
            key: key.to_string(),
            value: value.to_string(),
            note: note.map(|s| s.to_string()),
            project_id: project_id.to_string(),
        };

        state.secrets.insert(secret_id, secret.clone());
        Ok(secret)
    }

    async fn update_secret(
        &self,
        secret_id: &str,
        key: &str,
        value: &str,
        note: Option<&str>,
    ) -> Result<Secret> {
        let mut state = self.state.lock().unwrap();

        let existing = state
            .secrets
            .get(secret_id)
            .ok_or_else(|| AppError::ItemNotFound(format!("Secret not found: {}", secret_id)))?
            .clone();

        // Check for duplicate key if key is changing
        if key != existing.key {
            let duplicate = state
                .secrets
                .values()
                .any(|s| s.id != secret_id && s.project_id == existing.project_id && s.key == key);

            if duplicate {
                return Err(AppError::InvalidArguments(format!(
                    "Secret with key '{}' already exists in project",
                    key
                )));
            }
        }

        let updated = Secret {
            id: secret_id.to_string(),
            key: key.to_string(),
            value: value.to_string(),
            note: note.map(|s| s.to_string()),
            project_id: existing.project_id,
        };

        state.secrets.insert(secret_id.to_string(), updated.clone());
        Ok(updated)
    }

    async fn delete_secret(&self, secret_id: &str) -> Result<()> {
        let mut state = self.state.lock().unwrap();

        if state.secrets.remove(secret_id).is_none() {
            return Err(AppError::ItemNotFound(format!(
                "Secret not found: {}",
                secret_id
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_project() -> Project {
        Project {
            id: "proj_1".to_string(),
            name: "Test Project".to_string(),
            organization_id: "org_1".to_string(),
        }
    }

    #[tokio::test]
    async fn test_mock_provider_list_empty_projects() {
        let provider = MockProvider::new();
        let projects = provider.list_projects().await.unwrap();
        assert_eq!(projects.len(), 0);
    }

    #[tokio::test]
    async fn test_mock_provider_add_and_list_projects() {
        let provider = MockProvider::new();
        let project = create_test_project();
        provider.add_project(project.clone());

        let projects = provider.list_projects().await.unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Test Project");
    }

    #[tokio::test]
    async fn test_mock_provider_get_project() {
        let provider = MockProvider::new();
        let project = create_test_project();
        provider.add_project(project.clone());

        let found = provider.get_project("proj_1").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Test Project");
    }

    #[tokio::test]
    async fn test_mock_provider_get_project_by_name() {
        let provider = MockProvider::new();
        let project = create_test_project();
        provider.add_project(project.clone());

        let found = provider.get_project_by_name("Test Project").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "proj_1");
    }

    #[tokio::test]
    async fn test_mock_provider_create_secret() {
        let provider = MockProvider::new();
        let project = create_test_project();
        provider.add_project(project);

        let secret = provider
            .create_secret("proj_1", "API_KEY", "secret123", None)
            .await
            .unwrap();

        assert_eq!(secret.key, "API_KEY");
        assert_eq!(secret.value, "secret123");
        assert_eq!(secret.project_id, "proj_1");
    }

    #[tokio::test]
    async fn test_mock_provider_create_secret_with_note() {
        let provider = MockProvider::new();
        let project = create_test_project();
        provider.add_project(project);

        let secret = provider
            .create_secret("proj_1", "API_KEY", "secret123", Some("Production key"))
            .await
            .unwrap();

        assert_eq!(secret.note, Some("Production key".to_string()));
    }

    #[tokio::test]
    async fn test_mock_provider_create_secret_nonexistent_project() {
        let provider = MockProvider::new();

        let result = provider
            .create_secret("nonexistent", "API_KEY", "secret123", None)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_provider_create_duplicate_secret() {
        let provider = MockProvider::new();
        let project = create_test_project();
        provider.add_project(project);

        provider
            .create_secret("proj_1", "API_KEY", "secret123", None)
            .await
            .unwrap();

        let result = provider
            .create_secret("proj_1", "API_KEY", "secret456", None)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_provider_list_secrets() {
        let provider = MockProvider::new();
        let project = create_test_project();
        provider.add_project(project);

        provider
            .create_secret("proj_1", "KEY1", "value1", None)
            .await
            .unwrap();
        provider
            .create_secret("proj_1", "KEY2", "value2", None)
            .await
            .unwrap();

        let secrets = provider.list_secrets("proj_1").await.unwrap();
        assert_eq!(secrets.len(), 2);
    }

    #[tokio::test]
    async fn test_mock_provider_get_secret() {
        let provider = MockProvider::new();
        let project = create_test_project();
        provider.add_project(project);

        let created = provider
            .create_secret("proj_1", "API_KEY", "secret123", None)
            .await
            .unwrap();

        let found = provider.get_secret(&created.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().key, "API_KEY");
    }

    #[tokio::test]
    async fn test_mock_provider_update_secret() {
        let provider = MockProvider::new();
        let project = create_test_project();
        provider.add_project(project);

        let created = provider
            .create_secret("proj_1", "API_KEY", "secret123", None)
            .await
            .unwrap();

        let updated = provider
            .update_secret(&created.id, "API_KEY", "new_secret", Some("Updated"))
            .await
            .unwrap();

        assert_eq!(updated.value, "new_secret");
        assert_eq!(updated.note, Some("Updated".to_string()));
    }

    #[tokio::test]
    async fn test_mock_provider_update_secret_change_key() {
        let provider = MockProvider::new();
        let project = create_test_project();
        provider.add_project(project);

        let created = provider
            .create_secret("proj_1", "API_KEY", "secret123", None)
            .await
            .unwrap();

        let updated = provider
            .update_secret(&created.id, "NEW_KEY", "secret123", None)
            .await
            .unwrap();

        assert_eq!(updated.key, "NEW_KEY");
    }

    #[tokio::test]
    async fn test_mock_provider_update_nonexistent_secret() {
        let provider = MockProvider::new();

        let result = provider
            .update_secret("nonexistent", "KEY", "value", None)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_provider_delete_secret() {
        let provider = MockProvider::new();
        let project = create_test_project();
        provider.add_project(project);

        let created = provider
            .create_secret("proj_1", "API_KEY", "secret123", None)
            .await
            .unwrap();

        provider.delete_secret(&created.id).await.unwrap();

        let secrets = provider.list_secrets("proj_1").await.unwrap();
        assert_eq!(secrets.len(), 0);
    }

    #[tokio::test]
    async fn test_mock_provider_delete_nonexistent_secret() {
        let provider = MockProvider::new();

        let result = provider.delete_secret("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_provider_get_secrets_map() {
        let provider = MockProvider::new();
        let project = create_test_project();
        provider.add_project(project);

        provider
            .create_secret("proj_1", "KEY1", "value1", None)
            .await
            .unwrap();
        provider
            .create_secret("proj_1", "KEY2", "value2", None)
            .await
            .unwrap();

        let map = provider.get_secrets_map("proj_1").await.unwrap();
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("KEY1"), Some(&"value1".to_string()));
        assert_eq!(map.get("KEY2"), Some(&"value2".to_string()));
    }

    #[tokio::test]
    async fn test_mock_provider_sync_secrets_create() {
        let provider = MockProvider::new();
        let project = create_test_project();
        provider.add_project(project);

        let mut secrets = HashMap::new();
        secrets.insert("KEY1".to_string(), "value1".to_string());
        secrets.insert("KEY2".to_string(), "value2".to_string());

        let results = provider
            .sync_secrets("proj_1", &secrets, false)
            .await
            .unwrap();
        assert_eq!(results.len(), 2);

        let all_secrets = provider.list_secrets("proj_1").await.unwrap();
        assert_eq!(all_secrets.len(), 2);
    }

    #[tokio::test]
    async fn test_mock_provider_sync_secrets_overwrite() {
        let provider = MockProvider::new();
        let project = create_test_project();
        provider.add_project(project);

        // Create initial secret
        provider
            .create_secret("proj_1", "KEY1", "old_value", None)
            .await
            .unwrap();

        // Sync with overwrite
        let mut secrets = HashMap::new();
        secrets.insert("KEY1".to_string(), "new_value".to_string());

        provider
            .sync_secrets("proj_1", &secrets, true)
            .await
            .unwrap();

        let map = provider.get_secrets_map("proj_1").await.unwrap();
        assert_eq!(map.get("KEY1"), Some(&"new_value".to_string()));
    }

    #[tokio::test]
    async fn test_mock_provider_sync_secrets_no_overwrite() {
        let provider = MockProvider::new();
        let project = create_test_project();
        provider.add_project(project);

        // Create initial secret
        provider
            .create_secret("proj_1", "KEY1", "old_value", None)
            .await
            .unwrap();

        // Sync without overwrite
        let mut secrets = HashMap::new();
        secrets.insert("KEY1".to_string(), "new_value".to_string());

        provider
            .sync_secrets("proj_1", &secrets, false)
            .await
            .unwrap();

        let map = provider.get_secrets_map("proj_1").await.unwrap();
        assert_eq!(map.get("KEY1"), Some(&"old_value".to_string()));
    }

    #[tokio::test]
    async fn test_mock_provider_clear() {
        let provider = MockProvider::new();
        let project = create_test_project();
        provider.add_project(project);

        provider
            .create_secret("proj_1", "KEY1", "value1", None)
            .await
            .unwrap();

        provider.clear();

        let projects = provider.list_projects().await.unwrap();
        let secrets = provider.get_all_secrets();

        assert_eq!(projects.len(), 0);
        assert_eq!(secrets.len(), 0);
    }

    #[tokio::test]
    async fn test_mock_provider_with_data() {
        let project = create_test_project();
        let secret = Secret {
            id: "sec_1".to_string(),
            key: "API_KEY".to_string(),
            value: "secret123".to_string(),
            note: None,
            project_id: "proj_1".to_string(),
        };

        let provider = MockProvider::with_data(vec![project], vec![secret]);

        let projects = provider.list_projects().await.unwrap();
        let secrets = provider.list_secrets("proj_1").await.unwrap();

        assert_eq!(projects.len(), 1);
        assert_eq!(secrets.len(), 1);
    }
}
