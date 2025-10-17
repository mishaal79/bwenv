//! Pull command - Download secrets from Bitwarden to .env
//!
//! Fetches secrets from Bitwarden Secrets Manager and writes to local .env file.

use crate::bitwarden::provider::SecretsProvider;
use crate::{AppError, Result};
use std::fs;
use std::path::Path;

pub async fn execute<P: SecretsProvider>(
    provider: P,
    project: &str,
    output: &str,
    force: bool,
) -> Result<()> {
    // Check if output file exists
    if Path::new(output).exists() && !force {
        return Err(AppError::EnvFileWriteError(format!(
            "File {} already exists. Use --force to overwrite",
            output
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

    println!("Pulling secrets from project: {}", proj.name);

    // Get secrets
    let secrets_map = provider.get_secrets_map(&proj.id).await?;

    if secrets_map.is_empty() {
        println!("No secrets found in project");
        return Ok(());
    }

    // Build .env content
    let mut content = String::new();
    content.push_str(&format!(
        "# Secrets from Bitwarden project: {}\n",
        proj.name
    ));
    content.push_str(&format!("# Project ID: {}\n\n", proj.id));

    for (key, value) in secrets_map.iter() {
        content.push_str(&format!("{}={}\n", key, value));
    }

    // Write to file
    fs::write(output, content)
        .map_err(|e| AppError::EnvFileWriteError(format!("Failed to write {}: {}", output, e)))?;

    println!(
        "Successfully pulled {} secrets to {}",
        secrets_map.len(),
        output
    );
    Ok(())
}
