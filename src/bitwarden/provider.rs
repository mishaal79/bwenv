//! SecretsProvider trait abstraction
//!
//! Defines the interface for interacting with secrets providers (SDK, mock, etc.)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::Result;

/// Represents a Bitwarden project containing secrets
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub organization_id: String,
}

/// Represents a secret in Bitwarden Secrets Manager
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Secret {
    pub id: String,
    pub key: String,
    pub value: String,
    pub note: Option<String>,
    pub project_id: String,
}

/// Trait for secrets provider implementations
///
/// This trait abstracts the interaction with Bitwarden Secrets Manager,
/// enabling testing with mock implementations and production use with the SDK.
#[async_trait]
pub trait SecretsProvider: Send + Sync {
    /// List all accessible projects
    async fn list_projects(&self) -> Result<Vec<Project>>;

    /// Get a specific project by ID
    async fn get_project(&self, project_id: &str) -> Result<Option<Project>>;

    /// Get a project by name
    async fn get_project_by_name(&self, name: &str) -> Result<Option<Project>>;

    /// List all secrets in a project
    async fn list_secrets(&self, project_id: &str) -> Result<Vec<Secret>>;

    /// Get secrets as a HashMap for easy .env conversion
    async fn get_secrets_map(&self, project_id: &str) -> Result<HashMap<String, String>> {
        let secrets = self.list_secrets(project_id).await?;
        Ok(secrets.into_iter().map(|s| (s.key, s.value)).collect())
    }

    /// Get a specific secret by ID
    async fn get_secret(&self, secret_id: &str) -> Result<Option<Secret>>;

    /// Create a new secret in a project
    async fn create_secret(
        &self,
        project_id: &str,
        key: &str,
        value: &str,
        note: Option<&str>,
    ) -> Result<Secret>;

    /// Update an existing secret
    async fn update_secret(
        &self,
        secret_id: &str,
        key: &str,
        value: &str,
        note: Option<&str>,
    ) -> Result<Secret>;

    /// Delete a secret
    async fn delete_secret(&self, secret_id: &str) -> Result<()>;

    /// Bulk update or create secrets (used for push operations)
    async fn sync_secrets(
        &self,
        project_id: &str,
        secrets: &HashMap<String, String>,
        overwrite: bool,
    ) -> Result<Vec<Secret>> {
        let existing = self.list_secrets(project_id).await?;
        let mut existing_map: HashMap<String, Secret> =
            existing.into_iter().map(|s| (s.key.clone(), s)).collect();

        let mut results = Vec::new();

        for (key, value) in secrets {
            if let Some(existing_secret) = existing_map.remove(key) {
                // Update existing secret
                if overwrite {
                    let updated = self
                        .update_secret(
                            &existing_secret.id,
                            key,
                            value,
                            existing_secret.note.as_deref(),
                        )
                        .await?;
                    results.push(updated);
                } else {
                    // Skip if not overwriting
                    results.push(existing_secret);
                }
            } else {
                // Create new secret
                let created = self.create_secret(project_id, key, value, None).await?;
                results.push(created);
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_creation() {
        let project = Project {
            id: "proj123".to_string(),
            name: "Test Project".to_string(),
            organization_id: "org456".to_string(),
        };

        assert_eq!(project.id, "proj123");
        assert_eq!(project.name, "Test Project");
    }

    #[test]
    fn test_secret_creation() {
        let secret = Secret {
            id: "sec123".to_string(),
            key: "API_KEY".to_string(),
            value: "secret_value".to_string(),
            note: Some("Production API key".to_string()),
            project_id: "proj123".to_string(),
        };

        assert_eq!(secret.key, "API_KEY");
        assert_eq!(secret.value, "secret_value");
        assert_eq!(secret.note, Some("Production API key".to_string()));
    }

    #[test]
    fn test_project_serialization() {
        let project = Project {
            id: "proj123".to_string(),
            name: "Test Project".to_string(),
            organization_id: "org456".to_string(),
        };

        let json = serde_json::to_string(&project).unwrap();
        let deserialized: Project = serde_json::from_str(&json).unwrap();

        assert_eq!(project, deserialized);
    }

    #[test]
    fn test_secret_serialization() {
        let secret = Secret {
            id: "sec123".to_string(),
            key: "API_KEY".to_string(),
            value: "secret_value".to_string(),
            note: None,
            project_id: "proj123".to_string(),
        };

        let json = serde_json::to_string(&secret).unwrap();
        let deserialized: Secret = serde_json::from_str(&json).unwrap();

        assert_eq!(secret, deserialized);
    }
}
