//! SDK Provider - Real Bitwarden SDK integration
//!
//! Production implementation using the official Bitwarden Rust SDK

use async_trait::async_trait;
use uuid::Uuid;

use bitwarden::{
    auth::login::AccessTokenLoginRequest,
    secrets_manager::{
        projects::{ProjectGetRequest, ProjectsListRequest},
        secrets::{
            SecretCreateRequest, SecretGetRequest, SecretIdentifiersByProjectRequest,
            SecretPutRequest, SecretsDeleteRequest,
        },
        ClientProjectsExt, ClientSecretsExt,
    },
    Client, ClientSettings, DeviceType,
};

use super::provider::{Project, Secret, SecretsProvider};
use crate::{AppError, Result};

/// SDK-based implementation using real Bitwarden SDK
#[derive(Debug)]
pub struct SdkProvider {
    client: Client,
    /// Organization ID extracted from access token
    organization_id: Uuid,
}

impl SdkProvider {
    /// Create a new SDK provider with the given access token
    ///
    /// This will initialize the Bitwarden client and authenticate with the access token.
    pub async fn new(access_token: String) -> Result<Self> {
        // Parse the access token to extract organization ID
        let organization_id = Self::parse_organization_id(&access_token)?;

        // Create client with default settings
        let settings = ClientSettings {
            identity_url: "https://identity.bitwarden.com".to_string(),
            api_url: "https://api.bitwarden.com".to_string(),
            user_agent: "bwenv".to_string(),
            device_type: DeviceType::SDK,
        };
        let client = Client::new(Some(settings));

        // Authenticate with access token
        let token_request = AccessTokenLoginRequest {
            access_token,
            state_file: None,
        };

        client
            .auth()
            .login_access_token(&token_request)
            .await
            .map_err(|_| AppError::BitwardenAuthFailed)?;

        Ok(Self {
            client,
            organization_id,
        })
    }

    /// Parse organization ID from access token
    ///
    /// Bitwarden access tokens have the format: {version}.{org_id}.{data}
    fn parse_organization_id(access_token: &str) -> Result<Uuid> {
        let parts: Vec<&str> = access_token.split('.').collect();
        if parts.len() < 2 {
            return Err(AppError::BitwardenAuthFailed);
        }

        Uuid::parse_str(parts[1]).map_err(|_| AppError::BitwardenAuthFailed)
    }

    /// Convert SDK Project to our Project type
    fn convert_project(
        sdk_project: bitwarden::secrets_manager::projects::ProjectResponse,
    ) -> Project {
        Project {
            id: sdk_project.id.to_string(),
            name: sdk_project.name,
            organization_id: sdk_project.organization_id.to_string(),
        }
    }

    /// Convert SDK Secret to our Secret type
    fn convert_secret(sdk_secret: bitwarden::secrets_manager::secrets::SecretResponse) -> Secret {
        Secret {
            id: sdk_secret.id.to_string(),
            key: sdk_secret.key,
            value: sdk_secret.value,
            note: if sdk_secret.note.is_empty() {
                None
            } else {
                Some(sdk_secret.note)
            },
            project_id: sdk_secret
                .project_id
                .map(|id| id.to_string())
                .unwrap_or_default(),
        }
    }
}

#[async_trait]
impl SecretsProvider for SdkProvider {
    async fn list_projects(&self) -> Result<Vec<Project>> {
        let request = ProjectsListRequest {
            organization_id: self.organization_id,
        };

        let response = self
            .client
            .projects()
            .list(&request)
            .await
            .map_err(|e| AppError::Unknown(format!("Failed to list projects: {}", e)))?;

        Ok(response
            .data
            .into_iter()
            .map(Self::convert_project)
            .collect())
    }

    async fn get_project(&self, project_id: &str) -> Result<Option<Project>> {
        let uuid = Uuid::parse_str(project_id).map_err(|_| {
            AppError::InvalidArguments(format!("Invalid project ID: {}", project_id))
        })?;

        let request = ProjectGetRequest { id: uuid };

        match self.client.projects().get(&request).await {
            Ok(project) => Ok(Some(Self::convert_project(project))),
            Err(_) => Ok(None),
        }
    }

    async fn get_project_by_name(&self, name: &str) -> Result<Option<Project>> {
        let projects = self.list_projects().await?;
        Ok(projects.into_iter().find(|p| p.name == name))
    }

    async fn list_secrets(&self, project_id: &str) -> Result<Vec<Secret>> {
        let uuid = Uuid::parse_str(project_id).map_err(|_| {
            AppError::InvalidArguments(format!("Invalid project ID: {}", project_id))
        })?;

        let request = SecretIdentifiersByProjectRequest { project_id: uuid };

        let identifiers = self
            .client
            .secrets()
            .list_by_project(&request)
            .await
            .map_err(|e| AppError::Unknown(format!("Failed to list secrets: {}", e)))?;

        // For each identifier, fetch the full secret
        let mut secrets = Vec::new();
        for identifier in identifiers.data {
            let secret_request = SecretGetRequest { id: identifier.id };
            match self.client.secrets().get(&secret_request).await {
                Ok(secret) => secrets.push(Self::convert_secret(secret)),
                Err(e) => {
                    // Log error but continue
                    eprintln!("Warning: Failed to fetch secret {}: {}", identifier.id, e);
                }
            }
        }

        Ok(secrets)
    }

    async fn get_secret(&self, secret_id: &str) -> Result<Option<Secret>> {
        let uuid = Uuid::parse_str(secret_id)
            .map_err(|_| AppError::InvalidArguments(format!("Invalid secret ID: {}", secret_id)))?;

        let request = SecretGetRequest { id: uuid };

        match self.client.secrets().get(&request).await {
            Ok(secret) => Ok(Some(Self::convert_secret(secret))),
            Err(_) => Ok(None),
        }
    }

    async fn create_secret(
        &self,
        project_id: &str,
        key: &str,
        value: &str,
        note: Option<&str>,
    ) -> Result<Secret> {
        let project_uuid = Uuid::parse_str(project_id).map_err(|_| {
            AppError::InvalidArguments(format!("Invalid project ID: {}", project_id))
        })?;

        let request = SecretCreateRequest {
            organization_id: self.organization_id,
            key: key.to_string(),
            value: value.to_string(),
            note: note.unwrap_or("").to_string(),
            project_ids: Some(vec![project_uuid]),
        };

        let secret = self
            .client
            .secrets()
            .create(&request)
            .await
            .map_err(|e| AppError::Unknown(format!("Failed to create secret: {}", e)))?;

        Ok(Self::convert_secret(secret))
    }

    async fn update_secret(
        &self,
        secret_id: &str,
        key: &str,
        value: &str,
        note: Option<&str>,
    ) -> Result<Secret> {
        let uuid = Uuid::parse_str(secret_id)
            .map_err(|_| AppError::InvalidArguments(format!("Invalid secret ID: {}", secret_id)))?;

        // First get the current secret to preserve project_ids
        let current = self
            .get_secret(secret_id)
            .await?
            .ok_or_else(|| AppError::ItemNotFound(secret_id.to_string()))?;

        let project_ids = if !current.project_id.is_empty() {
            Some(vec![Uuid::parse_str(&current.project_id).unwrap()])
        } else {
            None
        };

        let request = SecretPutRequest {
            id: uuid,
            organization_id: self.organization_id,
            key: key.to_string(),
            value: value.to_string(),
            note: note.unwrap_or("").to_string(),
            project_ids,
        };

        let secret = self
            .client
            .secrets()
            .update(&request)
            .await
            .map_err(|e| AppError::Unknown(format!("Failed to update secret: {}", e)))?;

        Ok(Self::convert_secret(secret))
    }

    async fn delete_secret(&self, secret_id: &str) -> Result<()> {
        let uuid = Uuid::parse_str(secret_id)
            .map_err(|_| AppError::InvalidArguments(format!("Invalid secret ID: {}", secret_id)))?;

        let request = SecretsDeleteRequest { ids: vec![uuid] };

        self.client
            .secrets()
            .delete(request)
            .await
            .map_err(|e| AppError::Unknown(format!("Failed to delete secret: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_organization_id() {
        let token = "0.48b4774c-68ca-4539-a3d7-ac00018b4377.valid_data_here";
        let org_id = SdkProvider::parse_organization_id(token).unwrap();
        assert_eq!(org_id.to_string(), "48b4774c-68ca-4539-a3d7-ac00018b4377");
    }

    #[test]
    fn test_parse_organization_id_invalid() {
        let token = "invalid_token";
        let result = SdkProvider::parse_organization_id(token);
        assert!(result.is_err());
    }
}
