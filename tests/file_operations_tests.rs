use bwenv::env::parser::{read_env_file, validate_env_file, write_env_file};
use std::collections::HashMap;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_roundtrip_env_file_operations() {
    let temp_dir = tempdir().unwrap();
    let original_file = temp_dir.path().join("original.env");
    let roundtrip_file = temp_dir.path().join("roundtrip.env");

    // Create original file
    let original_content = r#"
# Database configuration
DB_HOST=localhost
DB_PORT=5432
DB_NAME=myapp_db
DB_USER=dbuser
DB_PASSWORD=supersecret

# API Configuration
API_URL=https://api.example.com/v1
API_KEY=sk_test_abcdefghijklmnop
TIMEOUT=30

# Feature flags
ENABLE_LOGGING=true
DEBUG_MODE=false
"#;

    fs::write(&original_file, original_content).unwrap();

    // Read the file
    let env_vars = read_env_file(&original_file).unwrap();

    // Verify expected variables are present
    assert_eq!(env_vars.get("DB_HOST"), Some(&"localhost".to_string()));
    assert_eq!(env_vars.get("DB_PORT"), Some(&"5432".to_string()));
    assert_eq!(
        env_vars.get("API_KEY"),
        Some(&"sk_test_abcdefghijklmnop".to_string())
    );
    assert_eq!(env_vars.get("ENABLE_LOGGING"), Some(&"true".to_string()));
    assert_eq!(env_vars.get("DEBUG_MODE"), Some(&"false".to_string()));

    // Write to new file
    write_env_file(&roundtrip_file, &env_vars, false).unwrap();

    // Read back and verify
    let roundtrip_vars = read_env_file(&roundtrip_file).unwrap();

    // Should have same variables (comments are not preserved)
    assert_eq!(env_vars.len(), roundtrip_vars.len());
    for (key, value) in &env_vars {
        assert_eq!(roundtrip_vars.get(key), Some(value));
    }
}

#[test]
fn test_merge_env_files() {
    let temp_dir = tempdir().unwrap();
    let base_file = temp_dir.path().join("base.env");
    let output_file = temp_dir.path().join("merged.env");

    // Create base file
    let base_content = r#"
BASE_KEY=base_value
SHARED_KEY=original_value
ANOTHER_BASE=base_data
"#;
    fs::write(&base_file, base_content).unwrap();

    // Copy base to output
    fs::copy(&base_file, &output_file).unwrap();

    // Create new variables to merge
    let mut new_vars = HashMap::new();
    new_vars.insert("NEW_KEY".to_string(), "new_value".to_string());
    new_vars.insert("SHARED_KEY".to_string(), "updated_value".to_string());
    new_vars.insert("ANOTHER_NEW".to_string(), "more_data".to_string());

    // Merge with existing file
    write_env_file(&output_file, &new_vars, true).unwrap();

    // Read merged result
    let merged_vars = read_env_file(&output_file).unwrap();

    // Should have all variables
    assert_eq!(merged_vars.get("BASE_KEY"), Some(&"base_value".to_string()));
    assert_eq!(
        merged_vars.get("ANOTHER_BASE"),
        Some(&"base_data".to_string())
    );
    assert_eq!(merged_vars.get("NEW_KEY"), Some(&"new_value".to_string()));
    assert_eq!(
        merged_vars.get("ANOTHER_NEW"),
        Some(&"more_data".to_string())
    );

    // Shared key should be updated
    assert_eq!(
        merged_vars.get("SHARED_KEY"),
        Some(&"updated_value".to_string())
    );

    // Total count should be correct
    assert_eq!(merged_vars.len(), 5);
}

#[test]
fn test_overwrite_vs_merge_behavior() {
    let temp_dir = tempdir().unwrap();
    let target_file = temp_dir.path().join("target.env");

    // Create initial file
    let initial_content = "INITIAL_KEY=initial_value\nSHARED_KEY=old_value";
    fs::write(&target_file, initial_content).unwrap();

    let mut new_vars = HashMap::new();
    new_vars.insert("NEW_KEY".to_string(), "new_value".to_string());
    new_vars.insert("SHARED_KEY".to_string(), "new_value".to_string());

    // Test overwrite (merge = false)
    write_env_file(&target_file, &new_vars, false).unwrap();
    let overwrite_result = read_env_file(&target_file).unwrap();

    assert_eq!(overwrite_result.len(), 2);
    assert_eq!(
        overwrite_result.get("NEW_KEY"),
        Some(&"new_value".to_string())
    );
    assert_eq!(
        overwrite_result.get("SHARED_KEY"),
        Some(&"new_value".to_string())
    );
    assert_eq!(overwrite_result.get("INITIAL_KEY"), None); // Should be gone

    // Reset and test merge
    fs::write(&target_file, initial_content).unwrap();
    write_env_file(&target_file, &new_vars, true).unwrap();
    let merge_result = read_env_file(&target_file).unwrap();

    assert_eq!(merge_result.len(), 3);
    assert_eq!(
        merge_result.get("INITIAL_KEY"),
        Some(&"initial_value".to_string())
    );
    assert_eq!(merge_result.get("NEW_KEY"), Some(&"new_value".to_string()));
    assert_eq!(
        merge_result.get("SHARED_KEY"),
        Some(&"new_value".to_string())
    );
}

#[test]
fn test_env_file_validation_comprehensive() {
    let temp_dir = tempdir().unwrap();

    // Test valid file
    let valid_file = temp_dir.path().join("valid.env");
    let valid_content = r#"
# Valid .env file
KEY1=value1
KEY2=value2
KEY3=
KEY4=value with spaces
KEY_WITH_UNDERSCORE=value
KEY123=numeric_key
"#;
    fs::write(&valid_file, valid_content).unwrap();
    assert!(validate_env_file(&valid_file).is_ok());

    // Test file with missing equals
    let invalid_file1 = temp_dir.path().join("invalid1.env");
    let invalid_content1 = r#"
KEY1=value1
INVALID_LINE_NO_EQUALS
KEY2=value2
"#;
    fs::write(&invalid_file1, invalid_content1).unwrap();
    let result1 = validate_env_file(&invalid_file1);
    assert!(result1.is_err());
    assert!(result1
        .unwrap_err()
        .to_string()
        .contains("missing '=' character"));

    // Test file with empty key
    let invalid_file2 = temp_dir.path().join("invalid2.env");
    let invalid_content2 = r#"
KEY1=value1
=value_with_empty_key
KEY2=value2
"#;
    fs::write(&invalid_file2, invalid_content2).unwrap();
    let result2 = validate_env_file(&invalid_file2);
    assert!(result2.is_err());
    assert!(result2.unwrap_err().to_string().contains("empty key name"));
}

#[test]
fn test_env_file_with_special_characters() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("special.env");

    let content = r#"
URL_WITH_PROTOCOL=https://example.com:8080/path?query=value&other=data
JSON_CONFIG={"key":"value","number":123,"boolean":true}
PATH_WITH_SPACES=/path/with spaces/file.txt
SPECIAL_CHARS=!@#$%^&*()_+-={}[]|;:,.<>?
UNICODE_VALUE=café_résumé_naïve
EMAIL=user@example.com
BASE64_VALUE=SGVsbG8gV29ybGQ=
"#;

    fs::write(&file_path, content).unwrap();

    let env_vars = read_env_file(&file_path).unwrap();

    assert_eq!(
        env_vars.get("URL_WITH_PROTOCOL"),
        Some(&"https://example.com:8080/path?query=value&other=data".to_string())
    );
    assert_eq!(
        env_vars.get("JSON_CONFIG"),
        Some(&r#"{"key":"value","number":123,"boolean":true}"#.to_string())
    );
    assert_eq!(
        env_vars.get("PATH_WITH_SPACES"),
        Some(&"/path/with spaces/file.txt".to_string())
    );
    assert_eq!(
        env_vars.get("SPECIAL_CHARS"),
        Some(&"!@#$%^&*()_+-={}[]|;:,.<>?".to_string())
    );
    assert_eq!(env_vars.get("EMAIL"), Some(&"user@example.com".to_string()));
}

#[test]
fn test_large_env_file() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("large.env");

    let mut content = String::new();
    content.push_str("# Large environment file\n");

    // Generate many environment variables
    for i in 0..1000 {
        content.push_str(&format!("VAR_{:04}=value_{:04}\n", i, i));
    }

    fs::write(&file_path, &content).unwrap();

    let env_vars = read_env_file(&file_path).unwrap();

    assert_eq!(env_vars.len(), 1000);
    assert_eq!(env_vars.get("VAR_0000"), Some(&"value_0000".to_string()));
    assert_eq!(env_vars.get("VAR_0999"), Some(&"value_0999".to_string()));
    assert_eq!(env_vars.get("VAR_0500"), Some(&"value_0500".to_string()));
}

#[test]
fn test_env_file_edge_cases() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("edge_cases.env");

    let content = r#"
# Edge cases for .env parsing
KEY_WITH_MULTIPLE_EQUALS=value=with=equals=signs
KEY_WITH_LEADING_SPACES   =   value_with_spaces
   KEY_WITH_LEADING_SPACES2=value2
KEY_EMPTY_VALUE=
KEY_ONLY_SPACES=   
KEY_WITH_HASH_IN_VALUE=value#with#hash
KEY_WITH_QUOTE=value"with"quotes
KEY_WITH_NEWLINE=value\nwith\nnewlines
"#;

    fs::write(&file_path, content).unwrap();

    let env_vars = read_env_file(&file_path).unwrap();

    assert_eq!(
        env_vars.get("KEY_WITH_MULTIPLE_EQUALS"),
        Some(&"value=with=equals=signs".to_string())
    );
    assert_eq!(
        env_vars.get("KEY_WITH_LEADING_SPACES"),
        Some(&"value_with_spaces".to_string())
    );
    assert_eq!(
        env_vars.get("KEY_WITH_LEADING_SPACES2"),
        Some(&"value2".to_string())
    );
    assert_eq!(env_vars.get("KEY_EMPTY_VALUE"), Some(&"".to_string()));
    assert_eq!(env_vars.get("KEY_ONLY_SPACES"), Some(&"".to_string()));
    assert_eq!(
        env_vars.get("KEY_WITH_HASH_IN_VALUE"),
        Some(&"value#with#hash".to_string())
    );
    assert_eq!(
        env_vars.get("KEY_WITH_QUOTE"),
        Some(&"value\"with\"quotes".to_string())
    );
    assert_eq!(
        env_vars.get("KEY_WITH_NEWLINE"),
        Some(&"value\\nwith\\nnewlines".to_string())
    );
}

#[test]
fn test_empty_env_file() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("empty.env");

    fs::write(&file_path, "").unwrap();

    let env_vars = read_env_file(&file_path).unwrap();
    assert_eq!(env_vars.len(), 0);

    assert!(validate_env_file(&file_path).is_ok());
}

#[test]
fn test_comments_only_env_file() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("comments_only.env");

    let content = r#"
# This file only contains comments
# No actual environment variables here
# Just documentation

# More comments
# And empty lines

"#;

    fs::write(&file_path, content).unwrap();

    let env_vars = read_env_file(&file_path).unwrap();
    assert_eq!(env_vars.len(), 0);

    assert!(validate_env_file(&file_path).is_ok());
}

#[test]
fn test_write_env_file_creates_readable_format() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("formatted.env");

    let mut env_vars = HashMap::new();
    env_vars.insert("ZEBRA_VAR".to_string(), "last".to_string());
    env_vars.insert("ALPHA_VAR".to_string(), "first".to_string());
    env_vars.insert("BETA_VAR".to_string(), "second".to_string());

    write_env_file(&file_path, &env_vars, false).unwrap();

    let content = fs::read_to_string(&file_path).unwrap();

    // Check header is present
    assert!(content.contains("# Environment variables"));
    assert!(content.contains("# Generated by bwenv"));

    // Check variables are sorted
    let lines: Vec<&str> = content.lines().collect();
    let var_lines: Vec<&str> = lines
        .iter()
        .filter(|line| line.contains('=') && !line.starts_with('#'))
        .copied()
        .collect();

    assert_eq!(var_lines[0], "ALPHA_VAR=first");
    assert_eq!(var_lines[1], "BETA_VAR=second");
    assert_eq!(var_lines[2], "ZEBRA_VAR=last");
}

#[test]
fn test_file_permissions_and_access() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.env");

    // Test writing to a new file
    let mut env_vars = HashMap::new();
    env_vars.insert("TEST_KEY".to_string(), "test_value".to_string());

    write_env_file(&file_path, &env_vars, false).unwrap();
    assert!(file_path.exists());

    // Test reading the file we just wrote
    let read_vars = read_env_file(&file_path).unwrap();
    assert_eq!(read_vars.get("TEST_KEY"), Some(&"test_value".to_string()));

    // Test overwriting existing file
    env_vars.insert("NEW_KEY".to_string(), "new_value".to_string());
    write_env_file(&file_path, &env_vars, false).unwrap();

    let overwritten_vars = read_env_file(&file_path).unwrap();
    assert_eq!(overwritten_vars.len(), 2);
    assert_eq!(
        overwritten_vars.get("TEST_KEY"),
        Some(&"test_value".to_string())
    );
    assert_eq!(
        overwritten_vars.get("NEW_KEY"),
        Some(&"new_value".to_string())
    );
}
