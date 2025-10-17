//! Test helper functions and utilities
//!
//! Provides custom assertions, environment guards, and test utilities

use std::collections::HashMap;
use std::env;
use std::path::Path;

/// Initialize test logging (call once per test if needed)
pub fn init_test_logging() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Debug)
        .try_init();
}

/// Guard that restores environment variables after test
pub struct EnvGuard {
    original: HashMap<String, Option<String>>,
}

impl EnvGuard {
    /// Create a new EnvGuard for the specified keys
    pub fn new(keys: &[&str]) -> Self {
        let mut original = HashMap::new();
        for key in keys {
            original.insert(key.to_string(), env::var(key).ok());
        }
        Self { original }
    }

    /// Set an environment variable
    pub fn set(&self, key: &str, value: &str) {
        env::set_var(key, value);
    }

    /// Remove an environment variable
    pub fn remove(&self, key: &str) {
        env::remove_var(key);
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, original_value) in &self.original {
            match original_value {
                Some(value) => env::set_var(key, value),
                None => env::remove_var(key),
            }
        }
    }
}

/// Assert that two .env files contain equivalent key-value pairs
///
/// This ignores comments, empty lines, and order
pub fn assert_env_files_equivalent(path1: impl AsRef<Path>, path2: impl AsRef<Path>) {
    let map1 = parse_env_file(path1.as_ref()).expect("Failed to parse first file");
    let map2 = parse_env_file(path2.as_ref()).expect("Failed to parse second file");

    assert_eq!(
        map1, map2,
        "Environment files are not equivalent\nLeft: {:?}\nRight: {:?}",
        map1, map2
    );
}

/// Parse an .env file into a HashMap (ignoring comments and empty lines)
fn parse_env_file(path: &Path) -> Result<HashMap<String, String>, std::io::Error> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut map = HashMap::new();

    for line_result in reader.lines() {
        let line = line_result?;
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Parse KEY=VALUE
        if let Some(pos) = line.find('=') {
            let key = line[..pos].trim().to_string();
            let value = line[pos + 1..].trim().to_string();

            if !key.is_empty() {
                map.insert(key, value);
            }
        }
    }

    Ok(map)
}

/// Assert that a string does not contain any of the given secret values
pub fn assert_no_secrets_leaked(text: &str, secrets: &[&str]) {
    for secret in secrets {
        assert!(
            !text.contains(secret),
            "Secret '{}' was leaked in output:\n{}",
            secret,
            text
        );
    }
}

/// Assert that a HashMap contains the expected keys and values
pub fn assert_map_contains(map: &HashMap<String, String>, expected: &[(&str, &str)]) {
    for (key, value) in expected {
        assert_eq!(
            map.get(*key),
            Some(&value.to_string()),
            "Expected key '{}' to have value '{}', but got {:?}",
            key,
            value,
            map.get(*key)
        );
    }
}

/// Create a temporary .env file with the given content
pub fn create_temp_env_file(content: &str) -> Result<(std::path::PathBuf, tempfile::TempDir), std::io::Error> {
    let temp_dir = tempfile::tempdir()?;
    let file_path = temp_dir.path().join(".env");
    std::fs::write(&file_path, content)?;
    Ok((file_path, temp_dir))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_env_guard_restores_value() {
        let key = "TEST_ENV_VAR_GUARD";

        // Set initial value
        env::set_var(key, "initial");

        {
            let guard = EnvGuard::new(&[key]);
            guard.set(key, "modified");
            assert_eq!(env::var(key).unwrap(), "modified");
        }

        // Should be restored after guard is dropped
        assert_eq!(env::var(key).unwrap(), "initial");

        // Cleanup
        env::remove_var(key);
    }

    #[test]
    fn test_env_guard_restores_missing() {
        let key = "TEST_ENV_VAR_GUARD_MISSING";

        // Ensure key doesn't exist
        env::remove_var(key);

        {
            let guard = EnvGuard::new(&[key]);
            guard.set(key, "temporary");
            assert_eq!(env::var(key).unwrap(), "temporary");
        }

        // Should be removed after guard is dropped
        assert!(env::var(key).is_err());
    }

    #[test]
    fn test_assert_env_files_equivalent_same() {
        let temp_dir = tempdir().unwrap();
        let path1 = temp_dir.path().join("file1.env");
        let path2 = temp_dir.path().join("file2.env");

        let content1 = "KEY1=value1\nKEY2=value2\n";
        let content2 = "# Comment\nKEY2=value2\nKEY1=value1\n";

        fs::write(&path1, content1).unwrap();
        fs::write(&path2, content2).unwrap();

        assert_env_files_equivalent(&path1, &path2);
    }

    #[test]
    #[should_panic(expected = "not equivalent")]
    fn test_assert_env_files_equivalent_different() {
        let temp_dir = tempdir().unwrap();
        let path1 = temp_dir.path().join("file1.env");
        let path2 = temp_dir.path().join("file2.env");

        let content1 = "KEY1=value1\n";
        let content2 = "KEY1=different\n";

        fs::write(&path1, content1).unwrap();
        fs::write(&path2, content2).unwrap();

        assert_env_files_equivalent(&path1, &path2);
    }

    #[test]
    fn test_parse_env_file() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().join("test.env");

        let content = r#"
# Comment
KEY1=value1

KEY2=value2
# Another comment
KEY3=value3
"#;

        fs::write(&path, content).unwrap();

        let map = parse_env_file(&path).unwrap();

        assert_eq!(map.len(), 3);
        assert_eq!(map.get("KEY1"), Some(&"value1".to_string()));
        assert_eq!(map.get("KEY2"), Some(&"value2".to_string()));
        assert_eq!(map.get("KEY3"), Some(&"value3".to_string()));
    }

    #[test]
    fn test_assert_no_secrets_leaked_pass() {
        let text = "This is some log output with no secrets";
        let secrets = vec!["secret123", "password456"];

        assert_no_secrets_leaked(text, &secrets);
    }

    #[test]
    #[should_panic(expected = "leaked")]
    fn test_assert_no_secrets_leaked_fail() {
        let text = "Error: Failed with secret123";
        let secrets = vec!["secret123"];

        assert_no_secrets_leaked(text, &secrets);
    }

    #[test]
    fn test_assert_map_contains() {
        let mut map = HashMap::new();
        map.insert("KEY1".to_string(), "value1".to_string());
        map.insert("KEY2".to_string(), "value2".to_string());

        assert_map_contains(&map, &[("KEY1", "value1"), ("KEY2", "value2")]);
    }

    #[test]
    fn test_create_temp_env_file() {
        let content = "KEY1=value1\nKEY2=value2\n";
        let (path, _temp_dir) = create_temp_env_file(content).unwrap();

        assert!(path.exists());
        let read_content = fs::read_to_string(&path).unwrap();
        assert_eq!(read_content, content);
    }
}
