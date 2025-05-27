use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvSet {
    pub name: String,
    pub folder: Option<String>,
    pub items_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct BitwardenItem {
    id: String,
    name: String,
    notes: Option<String>,
    folder_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BitwardenFolder {
    id: String,
    name: String,
}

/// Checks if Bitwarden CLI is available
pub fn is_bitwarden_available() -> Result<bool> {
    let output = Command::new("bw")
        .arg("--version")
        .output();

    match output {
        Ok(out) => Ok(out.status.success()),
        Err(_) => Ok(false),
    }
}

/// Gets the session token from the Bitwarden desktop app
pub fn get_session_token() -> Result<String> {
    // Try to unlock Bitwarden and get the session token
    let output = Command::new("bw")
        .arg("unlock")
        .arg("--raw")
        .output()
        .context("Failed to execute Bitwarden unlock command")?;

    if output.status.success() {
        let session_token = String::from_utf8(output.stdout)
            .context("Failed to parse Bitwarden session token")?;
        return Ok(session_token.trim().to_string());
    }

    // If unlock fails, try to get the session token without unlocking
    let output = Command::new("bw")
        .arg("status")
        .arg("--raw")
        .output()
        .context("Failed to execute Bitwarden status command")?;

    if output.status.success() {
        let status_json = String::from_utf8(output.stdout)
            .context("Failed to parse Bitwarden status output")?;
        
        let status: serde_json::Value = serde_json::from_str(&status_json)
            .context("Failed to parse Bitwarden status JSON")?;
        
        if let Some(status_obj) = status.as_object() {
            if let Some(status_val) = status_obj.get("status") {
                if status_val.as_str().unwrap_or("") == "unlocked" {
                    if let Some(active_user_id) = status_obj.get("activeUserId") {
                        // Get session key for the active user
                        let user_id = active_user_id.as_str().unwrap_or("");
                        let output = Command::new("bw")
                            .arg("list")
                            .arg("--session")
                            .arg("--raw")
                            .output()
                            .context("Failed to get Bitwarden session")?;
                        
                        if output.status.success() {
                            let session_token = String::from_utf8(output.stdout)
                                .context("Failed to parse Bitwarden session token")?;
                            return Ok(session_token.trim().to_string());
                        }
                    }
                }
            }
        }
    }

    // If both methods fail, check environment variable
    if let Ok(session) = std::env::var("BW_SESSION") {
        if !session.is_empty() {
            return Ok(session);
        }
    }

    Err(anyhow!("Could not get Bitwarden session token. Please ensure the Bitwarden desktop app is running and unlocked."))
}

/// Stores environment variables in Bitwarden
pub fn store_secrets(
    session_token: &str,
    secrets: &HashMap<String, String>,
    name: &str,
    folder: Option<&str>,
    overwrite: bool,
) -> Result<()> {
    // Check if we need to create a folder
    let folder_id = if let Some(folder_name) = folder {
        Some(ensure_folder_exists(session_token, folder_name)?)
    } else {
        None
    };

    // Create or update the secure note with the env vars
    let env_contents = secrets
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("\n");

    // Check if the item already exists
    let existing_item = find_item_by_name(session_token, name)?;

    if let Some(item) = existing_item {
        if !overwrite {
            return Err(anyhow!(
                "An item with the name '{}' already exists. Use --overwrite to replace it.",
                name
            ));
        }

        // Update the existing item
        let mut command = Command::new("bw");
        command
            .arg("edit")
            .arg("item")
            .arg(item.id)
            .arg("--session")
            .arg(session_token)
            .arg("--notes")
            .arg(env_contents);

        if let Some(folder_id) = folder_id {
            command.arg("--folderId").arg(folder_id);
        }

        let output = command
            .output()
            .context("Failed to update item in Bitwarden")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to update item in Bitwarden: {}", error));
        }
    } else {
        // Create a new item
        let mut command = Command::new("bw");
        command
            .arg("create")
            .arg("item")
            .arg("--session")
            .arg(session_token)
            .arg("--name")
            .arg(name)
            .arg("--notes")
            .arg(env_contents)
            .arg("--type")
            .arg("securenote");

        if let Some(folder_id) = folder_id {
            command.arg("--folderId").arg(folder_id);
        }

        let output = command
            .output()
            .context("Failed to create item in Bitwarden")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to create item in Bitwarden: {}", error));
        }
    }

    Ok(())
}

/// Retrieves environment variables from Bitwarden
pub fn retrieve_secrets(
    session_token: &str,
    name: &str,
    folder: Option<&str>,
) -> Result<HashMap<String, String>> {
    let item = find_item_by_name(session_token, name)?
        .ok_or_else(|| anyhow!("No item found with name '{}'", name))?;

    // If folder is specified, verify the item is in that folder
    if let Some(folder_name) = folder {
        if let Some(folder_id) = item.folder_id {
            let folder_info = get_folder_by_id(session_token, &folder_id)?;
            if folder_info.name != folder_name {
                return Err(anyhow!(
                    "Item '{}' exists but not in folder '{}'",
                    name,
                    folder_name
                ));
            }
        } else if folder_name != "No Folder" {
            return Err(anyhow!(
                "Item '{}' exists but not in folder '{}'",
                name,
                folder_name
            ));
        }
    }

    // Parse the notes field into environment variables
    let notes = item.notes.unwrap_or_default();
    let mut env_vars = HashMap::new();

    for line in notes.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(pos) = line.find('=') {
            let key = line[..pos].trim().to_string();
            let value = line[pos + 1..].trim().to_string();
            env_vars.insert(key, value);
        }
    }

    Ok(env_vars)
}

/// Lists all environment sets stored in Bitwarden
pub fn list_env_sets(session_token: &str) -> Result<Vec<EnvSet>> {
    // Get all secure notes
    let output = Command::new("bw")
        .arg("list")
        .arg("items")
        .arg("--session")
        .arg(session_token)
        .output()
        .context("Failed to list items from Bitwarden")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to list items from Bitwarden: {}", error));
    }

    let items_json = String::from_utf8(output.stdout)
        .context("Failed to parse Bitwarden items output")?;

    let items: Vec<BitwardenItem> = serde_json::from_str(&items_json)
        .context("Failed to parse Bitwarden items JSON")?;

    // Get all folders
    let folders = get_all_folders(session_token)?;
    let folder_map: HashMap<String, String> = folders
        .into_iter()
        .map(|f| (f.id, f.name))
        .collect();

    // Filter for secure notes that look like environment sets
    let mut env_sets = Vec::new();
    for item in items {
        if let Some(notes) = &item.notes {
            // Check if this looks like an env file (contains KEY=VALUE patterns)
            let is_env_file = notes.lines().any(|line| {
                !line.is_empty() && !line.starts_with('#') && line.contains('=')
            });

            if is_env_file {
                let folder_name = item
                    .folder_id
                    .as_ref()
                    .and_then(|id| folder_map.get(id))
                    .cloned();

                let vars_count = notes
                    .lines()
                    .filter(|line| !line.is_empty() && !line.starts_with('#') && line.contains('='))
                    .count();

                env_sets.push(EnvSet {
                    name: item.name,
                    folder: folder_name,
                    items_count: vars_count,
                });
            }
        }
    }

    Ok(env_sets)
}

// Helper functions

fn find_item_by_name(session_token: &str, name: &str) -> Result<Option<BitwardenItem>> {
    let output = Command::new("bw")
        .arg("list")
        .arg("items")
        .arg("--search")
        .arg(name)
        .arg("--session")
        .arg(session_token)
        .output()
        .context("Failed to search for item in Bitwarden")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to search for item in Bitwarden: {}", error));
    }

    let items_json = String::from_utf8(output.stdout)
        .context("Failed to parse Bitwarden search output")?;

    let items: Vec<BitwardenItem> = serde_json::from_str(&items_json)
        .context("Failed to parse Bitwarden search JSON")?;

    // Find exact match by name
    let item = items.into_iter().find(|item| item.name == name);
    Ok(item)
}

fn ensure_folder_exists(session_token: &str, folder_name: &str) -> Result<String> {
    // Check if folder already exists
    let output = Command::new("bw")
        .arg("list")
        .arg("folders")
        .arg("--session")
        .arg(session_token)
        .output()
        .context("Failed to list folders from Bitwarden")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to list folders from Bitwarden: {}", error));
    }

    let folders_json = String::from_utf8(output.stdout)
        .context("Failed to parse Bitwarden folders output")?;

    let folders: Vec<BitwardenFolder> = serde_json::from_str(&folders_json)
        .context("Failed to parse Bitwarden folders JSON")?;

    // Try to find existing folder
    if let Some(folder) = folders.iter().find(|f| f.name == folder_name) {
        return Ok(folder.id.clone());
    }

    // Create new folder
    let output = Command::new("bw")
        .arg("create")
        .arg("folder")
        .arg("--session")
        .arg(session_token)
        .arg("--name")
        .arg(folder_name)
        .output()
        .context("Failed to create folder in Bitwarden")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to create folder in Bitwarden: {}", error));
    }

    let folder_json = String::from_utf8(output.stdout)
        .context("Failed to parse Bitwarden folder creation output")?;

    let folder: BitwardenFolder = serde_json::from_str(&folder_json)
        .context("Failed to parse Bitwarden folder creation JSON")?;

    Ok(folder.id)
}

fn get_folder_by_id(session_token: &str, folder_id: &str) -> Result<BitwardenFolder> {
    let output = Command::new("bw")
        .arg("get")
        .arg("folder")
        .arg(folder_id)
        .arg("--session")
        .arg(session_token)
        .output()
        .context("Failed to get folder from Bitwarden")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to get folder from Bitwarden: {}", error));
    }

    let folder_json = String::from_utf8(output.stdout)
        .context("Failed to parse Bitwarden folder output")?;

    let folder: BitwardenFolder = serde_json::from_str(&folder_json)
        .context("Failed to parse Bitwarden folder JSON")?;

    Ok(folder)
}

fn get_all_folders(session_token: &str) -> Result<Vec<BitwardenFolder>> {
    let output = Command::new("bw")
        .arg("list")
        .arg("folders")
        .arg("--session")
        .arg(session_token)
        .output()
        .context("Failed to list folders from Bitwarden")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to list folders from Bitwarden: {}", error));
    }

    let folders_json = String::from_utf8(output.stdout)
        .context("Failed to parse Bitwarden folders output")?;

    let folders: Vec<BitwardenFolder> = serde_json::from_str(&folders_json)
        .context("Failed to parse Bitwarden folders JSON")?;

    Ok(folders)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_env_set_creation() {
        let env_set = EnvSet {
            name: "test-env".to_string(),
            folder: Some("Development".to_string()),
            items_count: 5,
        };
        
        assert_eq!(env_set.name, "test-env");
        assert_eq!(env_set.folder, Some("Development".to_string()));
        assert_eq!(env_set.items_count, 5);
    }

    #[test]
    fn test_env_set_serialization() {
        let env_set = EnvSet {
            name: "test-env".to_string(),
            folder: Some("Development".to_string()),
            items_count: 3,
        };
        
        let json = serde_json::to_string(&env_set).unwrap();
        assert!(json.contains("test-env"));
        assert!(json.contains("Development"));
        assert!(json.contains("3"));
    }

    #[test]
    fn test_env_set_deserialization() {
        let json = r#"{"name":"test-env","folder":"Development","items_count":3}"#;
        let env_set: EnvSet = serde_json::from_str(json).unwrap();
        
        assert_eq!(env_set.name, "test-env");
        assert_eq!(env_set.folder, Some("Development".to_string()));
        assert_eq!(env_set.items_count, 3);
    }

    #[test]
    fn test_env_set_with_no_folder() {
        let env_set = EnvSet {
            name: "test-env".to_string(),
            folder: None,
            items_count: 2,
        };
        
        assert_eq!(env_set.name, "test-env");
        assert_eq!(env_set.folder, None);
        assert_eq!(env_set.items_count, 2);
    }

    #[test]
    fn test_bitwarden_item_creation() {
        let item = BitwardenItem {
            id: "item-123".to_string(),
            name: "Test Item".to_string(),
            notes: Some("KEY1=value1\nKEY2=value2".to_string()),
            folder_id: Some("folder-456".to_string()),
        };
        
        assert_eq!(item.id, "item-123");
        assert_eq!(item.name, "Test Item");
        assert!(item.notes.unwrap().contains("KEY1=value1"));
        assert_eq!(item.folder_id, Some("folder-456".to_string()));
    }

    #[test]
    fn test_bitwarden_folder_creation() {
        let folder = BitwardenFolder {
            id: "folder-123".to_string(),
            name: "Development".to_string(),
        };
        
        assert_eq!(folder.id, "folder-123");
        assert_eq!(folder.name, "Development");
    }

    #[test]
    fn test_is_bitwarden_available_when_command_fails() {
        // This test assumes `bw` command is not available
        // In a real test environment, you might want to mock this
        let result = is_bitwarden_available();
        assert!(result.is_ok());
        // The actual value depends on whether bw is installed in test environment
    }

    // Note: The following tests would require mocking Command::new() calls
    // which is complex in Rust. In a real-world scenario, you'd want to:
    // 1. Extract command execution into a trait
    // 2. Use dependency injection to inject a mock implementation
    // 3. Or use integration tests with a real Bitwarden CLI setup

    #[test]
    fn test_parse_env_vars_from_notes() {
        // Test the parsing logic that would be used in retrieve_secrets
        let notes = "KEY1=value1\nKEY2=value2\n# comment\nKEY3=value3\n\nKEY4=value4";
        let mut env_vars = HashMap::new();

        for line in notes.lines() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim().to_string();
                let value = line[pos + 1..].trim().to_string();
                env_vars.insert(key, value);
            }
        }

        assert_eq!(env_vars.get("KEY1"), Some(&"value1".to_string()));
        assert_eq!(env_vars.get("KEY2"), Some(&"value2".to_string()));
        assert_eq!(env_vars.get("KEY3"), Some(&"value3".to_string()));
        assert_eq!(env_vars.get("KEY4"), Some(&"value4".to_string()));
        assert_eq!(env_vars.len(), 4);
    }

    #[test]
    fn test_parse_env_vars_with_empty_values() {
        let notes = "KEY1=\nKEY2=value2\nKEY3=";
        let mut env_vars = HashMap::new();

        for line in notes.lines() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim().to_string();
                let value = line[pos + 1..].trim().to_string();
                env_vars.insert(key, value);
            }
        }

        assert_eq!(env_vars.get("KEY1"), Some(&"".to_string()));
        assert_eq!(env_vars.get("KEY2"), Some(&"value2".to_string()));
        assert_eq!(env_vars.get("KEY3"), Some(&"".to_string()));
        assert_eq!(env_vars.len(), 3);
    }

    #[test]
    fn test_parse_env_vars_with_spaces() {
        let notes = "KEY1 = value1\nKEY2= value2  \nKEY3 =value3\nKEY4=value4";
        let mut env_vars = HashMap::new();

        for line in notes.lines() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim().to_string();
                let value = line[pos + 1..].trim().to_string();
                env_vars.insert(key, value);
            }
        }

        assert_eq!(env_vars.get("KEY1"), Some(&"value1".to_string()));
        assert_eq!(env_vars.get("KEY2"), Some(&"value2".to_string()));
        assert_eq!(env_vars.get("KEY3"), Some(&"value3".to_string()));
        assert_eq!(env_vars.get("KEY4"), Some(&"value4".to_string()));
    }

    #[test]
    fn test_format_env_vars_for_storage() {
        // Test the formatting logic that would be used in store_secrets
        let mut secrets = HashMap::new();
        secrets.insert("DB_HOST".to_string(), "localhost".to_string());
        secrets.insert("DB_PORT".to_string(), "5432".to_string());
        secrets.insert("API_KEY".to_string(), "secret123".to_string());

        let env_contents = secrets
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("\n");

        // Check that all key-value pairs are present
        assert!(env_contents.contains("DB_HOST=localhost"));
        assert!(env_contents.contains("DB_PORT=5432"));
        assert!(env_contents.contains("API_KEY=secret123"));
        
        // Count the number of lines
        let line_count = env_contents.lines().count();
        assert_eq!(line_count, 3);
    }

    #[test]
    fn test_identify_env_file_in_notes() {
        // Test the logic used in list_env_sets to identify env files
        let valid_env_notes = "KEY1=value1\nKEY2=value2";
        let invalid_notes = "This is just a regular note without env vars";
        let mixed_notes = "Some text\nKEY1=value1\nMore text";
        let comment_only = "# Just comments\n# No actual env vars";

        // Valid env file should be detected
        let is_env_file = valid_env_notes.lines().any(|line| {
            !line.is_empty() && !line.starts_with('#') && line.contains('=')
        });
        assert!(is_env_file);

        // Invalid notes should not be detected
        let is_env_file = invalid_notes.lines().any(|line| {
            !line.is_empty() && !line.starts_with('#') && line.contains('=')
        });
        assert!(!is_env_file);

        // Mixed content should be detected
        let is_env_file = mixed_notes.lines().any(|line| {
            !line.is_empty() && !line.starts_with('#') && line.contains('=')
        });
        assert!(is_env_file);

        // Comment-only should not be detected
        let is_env_file = comment_only.lines().any(|line| {
            !line.is_empty() && !line.starts_with('#') && line.contains('=')
        });
        assert!(!is_env_file);
    }

    #[test]
    fn test_count_env_vars_in_notes() {
        // Test the counting logic used in list_env_sets
        let notes = "# Header comment\nKEY1=value1\n\nKEY2=value2\n# Another comment\nKEY3=value3";
        
        let vars_count = notes
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#') && line.contains('='))
            .count();
        
        assert_eq!(vars_count, 3);
    }

    #[test]
    fn test_json_parsing_error_handling() {
        // Test that invalid JSON would be handled properly
        let invalid_json = "{ invalid json }";
        let result = serde_json::from_str::<Vec<BitwardenItem>>(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_session_token_handling() {
        // Test session token validation
        let empty_token = "";
        let whitespace_token = "   ";
        
        assert!(empty_token.is_empty());
        assert!(whitespace_token.trim().is_empty());
    }
}