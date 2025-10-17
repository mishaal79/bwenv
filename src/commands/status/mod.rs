//! Status command - Show sync state between local and remote
//!
//! Compares local .env with Bitwarden Secrets Manager state.

use crate::bitwarden::provider::SecretsProvider;
use crate::env::parser;
use crate::Result;
use std::collections::HashSet;
use std::path::Path;

pub async fn execute<P: SecretsProvider>(
    provider: P,
    project: &str,
    env_file: Option<&str>,
) -> Result<()> {
    let env_path = env_file.unwrap_or(".env");

    println!("🔍 Checking sync status...");
    println!();

    // Get project
    let proj = if let Ok(Some(p)) = provider.get_project(project).await {
        p
    } else if let Ok(Some(p)) = provider.get_project_by_name(project).await {
        p
    } else {
        return Err(crate::AppError::ItemNotFound(format!(
            "Project: {}",
            project
        )));
    };

    println!("📦 Project: {} ({})", proj.name, proj.id);
    println!();

    // Get remote secrets from Bitwarden
    let remote_secrets = provider.get_secrets_map(&proj.id).await?;

    // Get local secrets from .env file
    let local_secrets = if Path::new(env_path).exists() {
        parser::read_env_file(env_path).map_err(|e| {
            crate::AppError::EnvFileReadError(format!("Failed to read {}: {}", env_path, e))
        })?
    } else {
        println!("⚠️  Local file '{}' not found", env_path);
        Default::default()
    };

    // Compare
    let remote_keys: HashSet<_> = remote_secrets.keys().collect();
    let local_keys: HashSet<_> = local_secrets.keys().collect();

    let only_remote: Vec<_> = remote_keys.difference(&local_keys).collect();
    let only_local: Vec<_> = local_keys.difference(&remote_keys).collect();
    let in_both: Vec<_> = remote_keys.intersection(&local_keys).collect();

    // Check for value differences
    let mut different_values = Vec::new();
    for key in &in_both {
        if remote_secrets.get(*key as &str) != local_secrets.get(*key as &str) {
            different_values.push(*key);
        }
    }

    // Print status
    if only_remote.is_empty() && only_local.is_empty() && different_values.is_empty() {
        println!("✅ In sync - Local and remote are identical");
        println!("   {} secrets match", in_both.len());
    } else {
        println!("⚠️  Out of sync detected:");
        println!();

        if !only_remote.is_empty() {
            println!("📥 Only in Bitwarden ({}):", only_remote.len());
            for key in only_remote {
                println!("   - {}", key);
            }
            println!("   → Run 'bwenv pull' to download these");
            println!();
        }

        if !only_local.is_empty() {
            println!("📤 Only in local .env ({}):", only_local.len());
            for key in only_local {
                println!("   - {}", key);
            }
            println!("   → Run 'bwenv push' to upload these");
            println!();
        }

        if !different_values.is_empty() {
            println!("🔄 Different values ({}):", different_values.len());
            for key in different_values {
                println!("   - {}", key);
            }
            println!("   → Run 'bwenv pull --force' to overwrite local");
            println!("   → Run 'bwenv push --overwrite' to overwrite remote");
            println!();
        }
    }

    Ok(())
}

/// List projects and optionally secrets within a project
pub async fn list<P: SecretsProvider>(provider: P, project: Option<&str>) -> Result<()> {
    if let Some(project_filter) = project {
        // List secrets in specific project
        let proj = if let Ok(Some(p)) = provider.get_project(project_filter).await {
            p
        } else if let Ok(Some(p)) = provider.get_project_by_name(project_filter).await {
            p
        } else {
            return Err(crate::AppError::ItemNotFound(format!(
                "Project: {}",
                project_filter
            )));
        };

        println!("Project: {} ({})", proj.name, proj.id);
        println!("\nSecrets:");

        let secrets = provider.list_secrets(&proj.id).await?;
        if secrets.is_empty() {
            println!("  No secrets found");
        } else {
            for secret in secrets {
                if let Some(note) = &secret.note {
                    println!("  {} = <hidden> ({})", secret.key, note);
                } else {
                    println!("  {} = <hidden>", secret.key);
                }
            }
        }
    } else {
        // List all projects
        let projects = provider.list_projects().await?;

        if projects.is_empty() {
            println!("No projects found");
        } else {
            println!("Projects:");
            for project in projects {
                println!("  {} ({})", project.name, project.id);
            }
            println!("\nUse 'bwenv list --project <name>' to see secrets in a project");
        }
    }

    Ok(())
}
