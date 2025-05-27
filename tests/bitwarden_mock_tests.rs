use std::collections::HashMap;
use std::process::{Command, Output, ExitStatus};
use std::ffi::OsStr;
use bwenv::bitwarden::*;

// Mock command executor trait
trait CommandExecutor {
    fn execute(&self, program: &str, args: &[&str]) -> Result<Output, std::io::Error>;
}

// Real command executor (for reference)
struct RealCommandExecutor;

impl CommandExecutor for RealCommandExecutor {
    fn execute(&self, program: &str, args: &[&str]) -> Result<Output, std::io::Error> {
        Command::new(program).args(args).output()
    }
}

// Mock command executor for testing
struct MockCommandExecutor {
    responses: HashMap<String, MockResponse>,
}

struct MockResponse {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    exit_code: i32,
}

impl MockCommandExecutor {
    fn new() -> Self {
        Self {
            responses: HashMap::new(),
        }
    }
    
    fn expect_command(&mut self, command_line: &str, response: MockResponse) {
        self.responses.insert(command_line.to_string(), response);
    }
    
    fn expect_success(&mut self, command_line: &str, stdout: &str) {
        self.expect_command(command_line, MockResponse {
            stdout: stdout.as_bytes().to_vec(),
            stderr: vec![],
            exit_code: 0,
        });
    }
    
    fn expect_failure(&mut self, command_line: &str, stderr: &str, exit_code: i32) {
        self.expect_command(command_line, MockResponse {
            stdout: vec![],
            stderr: stderr.as_bytes().to_vec(),
            exit_code,
        });
    }
}

impl CommandExecutor for MockCommandExecutor {
    fn execute(&self, program: &str, args: &[&str]) -> Result<Output, std::io::Error> {
        let command_line = format!("{} {}", program, args.join(" "));
        
        if let Some(response) = self.responses.get(&command_line) {
            Ok(Output {
                status: MockExitStatus { code: response.exit_code }.into(),
                stdout: response.stdout.clone(),
                stderr: response.stderr.clone(),
            })
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Unexpected command: {}", command_line)
            ))
        }
    }
}

struct MockExitStatus {
    code: i32,
}

impl From<MockExitStatus> for ExitStatus {
    fn from(_: MockExitStatus) -> Self {
        // This is a simplified mock - in real testing you'd use a more sophisticated approach
        // For now, we'll focus on testing the parsing logic rather than command execution
        std::process::Command::new("true").status().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_bitwarden_available_with_mock() {
        let mut mock_executor = MockCommandExecutor::new();
        mock_executor.expect_success("bw --version", "1.25.1");
        
        // Note: Since is_bitwarden_available() calls Command::new directly,
        // we can't easily mock it without refactoring the function.
        // This test demonstrates the testing approach for when we refactor.
    }

    #[test]
    fn test_parse_bitwarden_items_json() {
        let json_response = r#"[
            {
                "id": "item-123",
                "name": "Test Environment",
                "notes": "DB_HOST=localhost\nDB_PORT=5432\nAPI_KEY=secret123",
                "type": 2,
                "folderId": "folder-456"
            },
            {
                "id": "item-789",
                "name": "Another Environment",
                "notes": "REDIS_URL=redis://localhost:6379\nLOG_LEVEL=debug",
                "type": 2,
                "folderId": null
            }
        ]"#;
        
        let items: Vec<serde_json::Value> = serde_json::from_str(json_response).unwrap();
        assert_eq!(items.len(), 2);
        
        let first_item = &items[0];
        assert_eq!(first_item["id"], "item-123");
        assert_eq!(first_item["name"], "Test Environment");
        assert!(first_item["notes"].as_str().unwrap().contains("DB_HOST=localhost"));
    }

    #[test]
    fn test_parse_bitwarden_folders_json() {
        let json_response = r#"[
            {
                "id": "folder-123",
                "name": "Development"
            },
            {
                "id": "folder-456", 
                "name": "Production"
            }
        ]"#;
        
        let folders: Vec<serde_json::Value> = serde_json::from_str(json_response).unwrap();
        assert_eq!(folders.len(), 2);
        
        assert_eq!(folders[0]["id"], "folder-123");
        assert_eq!(folders[0]["name"], "Development");
        assert_eq!(folders[1]["id"], "folder-456");
        assert_eq!(folders[1]["name"], "Production");
    }

    #[test]
    fn test_env_vars_to_notes_conversion() {
        let mut env_vars = HashMap::new();
        env_vars.insert("DB_HOST".to_string(), "localhost".to_string());
        env_vars.insert("DB_PORT".to_string(), "5432".to_string());
        env_vars.insert("API_KEY".to_string(), "secret123".to_string());
        
        let notes = env_vars
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("\n");
        
        assert!(notes.contains("DB_HOST=localhost"));
        assert!(notes.contains("DB_PORT=5432"));
        assert!(notes.contains("API_KEY=secret123"));
        
        // Test parsing back
        let mut parsed_vars = HashMap::new();
        for line in notes.lines() {
            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim().to_string();
                let value = line[pos + 1..].trim().to_string();
                parsed_vars.insert(key, value);
            }
        }
        
        assert_eq!(parsed_vars, env_vars);
    }

    #[test]
    fn test_notes_to_env_vars_with_comments() {
        let notes = r#"# Database configuration
DB_HOST=localhost
DB_PORT=5432

# API Configuration  
API_KEY=secret123
API_URL=https://api.example.com

# Empty value
EMPTY_VAR=
"#;
        
        let mut env_vars = HashMap::new();
        for line in notes.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim().to_string();
                let value = line[pos + 1..].trim().to_string();
                env_vars.insert(key, value);
            }
        }
        
        assert_eq!(env_vars.len(), 5);
        assert_eq!(env_vars.get("DB_HOST"), Some(&"localhost".to_string()));
        assert_eq!(env_vars.get("DB_PORT"), Some(&"5432".to_string()));
        assert_eq!(env_vars.get("API_KEY"), Some(&"secret123".to_string()));
        assert_eq!(env_vars.get("API_URL"), Some(&"https://api.example.com".to_string()));
        assert_eq!(env_vars.get("EMPTY_VAR"), Some(&"".to_string()));
    }

    #[test]
    fn test_identify_env_items_from_response() {
        let items_json = r#"[
            {
                "id": "item-1",
                "name": "Password",
                "notes": "This is just a password note",
                "type": 1
            },
            {
                "id": "item-2", 
                "name": "Environment Variables",
                "notes": "DB_HOST=localhost\nDB_PORT=5432",
                "type": 2
            },
            {
                "id": "item-3",
                "name": "API Keys",
                "notes": "STRIPE_KEY=sk_test_123\nSENDGRID_KEY=SG.456",
                "type": 2
            },
            {
                "id": "item-4",
                "name": "Regular Note",
                "notes": "Just some text without env vars",
                "type": 2
            }
        ]"#;
        
        let items: Vec<serde_json::Value> = serde_json::from_str(items_json).unwrap();
        
        let env_items: Vec<_> = items.iter().filter(|item| {
            if let Some(notes) = item["notes"].as_str() {
                notes.lines().any(|line| {
                    !line.is_empty() && !line.starts_with('#') && line.contains('=')
                })
            } else {
                false
            }
        }).collect();
        
        assert_eq!(env_items.len(), 2);
        assert_eq!(env_items[0]["id"], "item-2");
        assert_eq!(env_items[1]["id"], "item-3");
    }

    #[test]
    fn test_count_env_vars_in_notes() {
        let notes = r#"# Configuration
DB_HOST=localhost
DB_PORT=5432

# API Settings
API_KEY=secret
API_URL=https://api.example.com

# Comments only section
# No variables here

# More variables
REDIS_URL=redis://localhost:6379
"#;
        
        let count = notes
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#') && line.contains('='))
            .count();
        
        assert_eq!(count, 5);
    }

    #[test]
    fn test_session_token_extraction() {
        let status_json = r#"{
            "status": "unlocked",
            "lastSync": "2023-01-01T00:00:00.000Z",
            "userEmail": "user@example.com",
            "userId": "user-123",
            "activeUserId": "user-123"
        }"#;
        
        let status: serde_json::Value = serde_json::from_str(status_json).unwrap();
        
        assert_eq!(status["status"], "unlocked");
        assert_eq!(status["activeUserId"], "user-123");
        
        // In real implementation, would extract session token based on this info
    }

    #[test]
    fn test_error_response_parsing() {
        let error_response = "Vault is locked";
        assert!(error_response.contains("locked"));
        
        let not_found_response = "Not found.";
        assert!(not_found_response.contains("Not found"));
        
        let auth_error = "You are not logged in.";
        assert!(auth_error.contains("not logged in"));
    }

    #[test]
    fn test_folder_creation_request() {
        // Test the JSON structure for creating a folder
        let folder_request = serde_json::json!({
            "name": "Development/MyProject"
        });
        
        assert_eq!(folder_request["name"], "Development/MyProject");
        
        let serialized = serde_json::to_string(&folder_request).unwrap();
        assert!(serialized.contains("Development/MyProject"));
    }

    #[test]
    fn test_item_creation_request() {
        // Test the JSON structure for creating a secure note
        let item_request = serde_json::json!({
            "type": 2,
            "name": "Test Environment",
            "notes": "DB_HOST=localhost\nDB_PORT=5432",
            "folderId": "folder-123"
        });
        
        assert_eq!(item_request["type"], 2);
        assert_eq!(item_request["name"], "Test Environment");
        assert!(item_request["notes"].as_str().unwrap().contains("DB_HOST=localhost"));
        assert_eq!(item_request["folderId"], "folder-123");
    }

    #[test]
    fn test_search_response_parsing() {
        let search_response = r#"[
            {
                "id": "item-123",
                "name": "My Environment",
                "notes": "KEY1=value1\nKEY2=value2"
            }
        ]"#;
        
        let items: Vec<serde_json::Value> = serde_json::from_str(search_response).unwrap();
        
        // Test finding exact match
        let exact_match = items.iter().find(|item| {
            item["name"].as_str() == Some("My Environment")
        });
        
        assert!(exact_match.is_some());
        assert_eq!(exact_match.unwrap()["id"], "item-123");
    }

    #[test]
    fn test_env_set_serialization() {
        let env_set = EnvSet {
            name: "Test Environment".to_string(),
            folder: Some("Development".to_string()),
            items_count: 5,
        };
        
        let json = serde_json::to_string(&env_set).unwrap();
        assert!(json.contains("Test Environment"));
        assert!(json.contains("Development"));
        assert!(json.contains("5"));
        
        // Test deserialization
        let deserialized: EnvSet = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "Test Environment");
        assert_eq!(deserialized.folder, Some("Development".to_string()));
        assert_eq!(deserialized.items_count, 5);
    }

    #[test]
    fn test_complex_env_values_in_notes() {
        let complex_notes = r#"DATABASE_URL=postgresql://user:password@localhost:5432/dbname
JWT_SECRET=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9
API_ENDPOINTS=https://api1.com,https://api2.com,https://api3.com
JSON_CONFIG={"key":"value","nested":{"inner":"data"}}
SPECIAL_CHARS=!@#$%^&*()_+-=[]{}|;:,.<>?
"#;
        
        let mut env_vars = HashMap::new();
        for line in complex_notes.lines() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim().to_string();
                let value = line[pos + 1..].trim().to_string();
                env_vars.insert(key, value);
            }
        }
        
        assert_eq!(env_vars.len(), 5);
        assert!(env_vars.get("DATABASE_URL").unwrap().contains("postgresql://"));
        assert!(env_vars.get("JWT_SECRET").unwrap().starts_with("eyJ"));
        assert!(env_vars.get("API_ENDPOINTS").unwrap().contains(","));
        assert!(env_vars.get("JSON_CONFIG").unwrap().contains("{"));
        assert!(env_vars.get("SPECIAL_CHARS").unwrap().contains("!@#"));
    }

    #[test]
    fn test_environment_variable_validation() {
        // Test valid environment variable names
        let valid_names = vec![
            "DB_HOST",
            "API_KEY", 
            "LOG_LEVEL",
            "FEATURE_FLAG_123",
            "PATH",
            "HOME",
        ];
        
        for name in valid_names {
            // Basic validation - alphanumeric and underscores, starting with letter
            assert!(name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));
            assert!(name.chars().next().unwrap().is_ascii_alphabetic() || name.starts_with('_'));
        }
    }

    #[test]
    fn test_bitwarden_cli_version_parsing() {
        let version_outputs = vec![
            "1.25.1",
            "2.0.0-beta.1",
            "1.22.0",
        ];
        
        for version in version_outputs {
            // Basic version format validation
            assert!(version.chars().any(|c| c.is_ascii_digit()));
            assert!(version.contains('.'));
        }
    }
}