//! Push command - Upload .env secrets to Bitwarden
//!
//! Reads local .env file and uploads secrets to Bitwarden Secrets Manager.

use crate::bitwarden::provider::SecretsProvider;
use crate::env::parser;
use crate::{AppError, Result};
use std::path::Path;

pub async fn execute<P: SecretsProvider>(
    provider: P,
    project: &str,
    input: &str,
    overwrite: bool,
) -> Result<()> {
    // Check if input file exists
    if !Path::new(input).exists() {
        return Err(AppError::EnvFileReadError(format!(
            "File {} not found",
            input
        )));
    }

    // Get project by name or ID
    let proj = if let Ok(Some(p)) = provider.get_project(project).await {
        p
    } else if let Ok(Some(p)) = provider.get_project_by_name(project).await {
        p
    } else {
        return Err(AppError::ItemNotFound(format!("Project: {}", project)));
    };

    println!("Pushing secrets to project: {}", proj.name);

    // Parse .env file
    let env_vars = parser::read_env_file(input)
        .map_err(|e| AppError::EnvFileReadError(format!("Failed to read {}: {}", input, e)))?;

    if env_vars.is_empty() {
        println!("No secrets found in {}", input);
        return Ok(());
    }

    // Sync secrets to Bitwarden
    let results = provider
        .sync_secrets(&proj.id, &env_vars, overwrite)
        .await?;

    println!("Successfully pushed {} secrets to Bitwarden", results.len());
    Ok(())
}
