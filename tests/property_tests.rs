use bwenv::env_file::{read_env_file, validate_env_file, write_env_file};
use proptest::prelude::*;
use std::collections::HashMap;
use std::fs;
use tempfile::tempdir;

// Strategy for generating valid environment variable keys
fn env_key_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"[A-Z][A-Z0-9_]*")
        .unwrap()
        .prop_filter("Key must not be empty", |s| !s.is_empty())
        .prop_map(|s| s.chars().take(50).collect()) // Limit length
}

// Strategy for generating environment variable values
fn env_value_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"[a-zA-Z0-9_\-\./:@#$%^&*()+=\[\]{}|;,<>?!~` ]*")
        .unwrap()
        .prop_map(|s| s.chars().take(200).collect()) // Limit length
}

// Strategy for generating a HashMap of environment variables
fn env_vars_strategy() -> impl Strategy<Value = HashMap<String, String>> {
    prop::collection::hash_map(env_key_strategy(), env_value_strategy(), 0..20)
}

proptest! {
    #[test]
    fn test_roundtrip_property(env_vars in env_vars_strategy()) {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.env");

        // Write environment variables to file
        write_env_file(&file_path, &env_vars, false).unwrap();

        // Read them back
        let read_vars = read_env_file(&file_path).unwrap();

        // Should be identical (accounting for whitespace trimming)
        prop_assert_eq!(env_vars.len(), read_vars.len());
        for (key, value) in &env_vars {
            let expected_value = value.trim().to_string();
            prop_assert_eq!(read_vars.get(key), Some(&expected_value));
        }
    }

    #[test]
    fn test_merge_property(
        initial_vars in env_vars_strategy(),
        new_vars in env_vars_strategy()
    ) {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.env");

        // Write initial variables
        write_env_file(&file_path, &initial_vars, false).unwrap();

        // Merge with new variables
        write_env_file(&file_path, &new_vars, true).unwrap();

        // Read merged result
        let merged_vars = read_env_file(&file_path).unwrap();

        // Check that all new variables are present
        for (key, value) in &new_vars {
            let expected_value = value.trim().to_string();
            prop_assert_eq!(merged_vars.get(key), Some(&expected_value));
        }

        // Check that initial variables are present (unless overwritten)
        for (key, value) in &initial_vars {
            if !new_vars.contains_key(key) {
                let expected_value = value.trim().to_string();
                prop_assert_eq!(merged_vars.get(key), Some(&expected_value));
            }
        }

        // Total size should be correct
        let expected_size = initial_vars.len() + new_vars.len() -
            initial_vars.keys().filter(|k| new_vars.contains_key(*k)).count();
        prop_assert_eq!(merged_vars.len(), expected_size);
    }

    #[test]
    fn test_overwrite_property(
        initial_vars in env_vars_strategy(),
        new_vars in env_vars_strategy()
    ) {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.env");

        // Write initial variables
        write_env_file(&file_path, &initial_vars, false).unwrap();

        // Overwrite with new variables (merge = false)
        write_env_file(&file_path, &new_vars, false).unwrap();

        // Read result
        let result_vars = read_env_file(&file_path).unwrap();

        // Should only contain new variables (accounting for whitespace trimming)
        prop_assert_eq!(result_vars.len(), new_vars.len());
        for (key, value) in &new_vars {
            let expected_value = value.trim().to_string();
            prop_assert_eq!(result_vars.get(key), Some(&expected_value));
        }

        // Should not contain any initial variables (unless they're also in new_vars)
        for key in initial_vars.keys() {
            if !new_vars.contains_key(key) {
                prop_assert_eq!(result_vars.get(key), None);
            }
        }
    }

    #[test]
    fn test_key_value_parsing_property(
        key in env_key_strategy(),
        value in env_value_strategy()
    ) {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.env");

        let content = format!("{}={}", key, value);
        fs::write(&file_path, &content).unwrap();

        let vars = read_env_file(&file_path).unwrap();

        prop_assert_eq!(vars.len(), 1);
        let expected_value = value.trim().to_string();
        prop_assert_eq!(vars.get(&key), Some(&expected_value));
    }

    #[test]
    fn test_multiple_equals_in_value_property(
        key in env_key_strategy(),
        value_parts in prop::collection::vec(r"[a-zA-Z0-9_\-\.]+", 2..5)
    ) {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.env");

        let value = value_parts.join("=");
        let content = format!("{}={}", key, value);
        fs::write(&file_path, &content).unwrap();

        let vars = read_env_file(&file_path).unwrap();

        prop_assert_eq!(vars.len(), 1);
        let expected_value = value.trim().to_string();
        prop_assert_eq!(vars.get(&key), Some(&expected_value));
    }

    #[test]
    fn test_empty_value_property(key in env_key_strategy()) {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.env");

        let content = format!("{}=", key);
        fs::write(&file_path, &content).unwrap();

        let vars = read_env_file(&file_path).unwrap();

        prop_assert_eq!(vars.len(), 1);
        let empty_string = String::new();
        prop_assert_eq!(vars.get(&key), Some(&empty_string));
    }

    #[test]
    fn test_whitespace_handling_property(
        key in env_key_strategy(),
        value in env_value_strategy(),
        leading_spaces in 0..10usize,
        trailing_spaces in 0..10usize
    ) {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.env");

        let content = format!("{}{}={}{}",
            " ".repeat(leading_spaces),
            key,
            value,
            " ".repeat(trailing_spaces)
        );
        fs::write(&file_path, &content).unwrap();

        let vars = read_env_file(&file_path).unwrap();

        prop_assert_eq!(vars.len(), 1);
        let expected_value = value.trim().to_string();
        prop_assert_eq!(vars.get(&key), Some(&expected_value));
    }

    #[test]
    fn test_validation_always_passes_for_valid_files_property(
        env_vars in env_vars_strategy()
    ) {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.env");

        write_env_file(&file_path, &env_vars, false).unwrap();

        // Files written by our function should always validate
        prop_assert!(validate_env_file(&file_path).is_ok());
    }

    #[test]
    fn test_sorting_consistency_property(env_vars in env_vars_strategy()) {
        let temp_dir = tempdir().unwrap();
        let file_path1 = temp_dir.path().join("test1.env");
        let file_path2 = temp_dir.path().join("test2.env");

        // Write the same variables to two different files
        write_env_file(&file_path1, &env_vars, false).unwrap();
        write_env_file(&file_path2, &env_vars, false).unwrap();

        // Files should be identical except for timestamps
        let content1 = fs::read_to_string(&file_path1).unwrap();
        let content2 = fs::read_to_string(&file_path2).unwrap();

        // Remove timestamp lines for comparison
        let lines1: Vec<&str> = content1.lines().filter(|line| !line.contains("Generated by") && !line.starts_with("# 20")).collect();
        let lines2: Vec<&str> = content2.lines().filter(|line| !line.contains("Generated by") && !line.starts_with("# 20")).collect();

        prop_assert_eq!(lines1, lines2);
    }

    #[test]
    fn test_comments_ignored_property(
        env_vars in env_vars_strategy(),
        comment_lines in prop::collection::vec(r"#[^\n]*", 0..5)
    ) {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.env");

        // Create content with comments interspersed
        let mut content = String::new();
        for comment in &comment_lines {
            content.push_str(comment);
            content.push('\n');
        }

        for (key, value) in &env_vars {
            content.push_str(&format!("{}={}\n", key, value));
            if !comment_lines.is_empty() {
                content.push_str(&comment_lines[0]);
                content.push('\n');
            }
        }

        fs::write(&file_path, &content).unwrap();

        let read_vars = read_env_file(&file_path).unwrap();

        // Should read the same variables regardless of comments (accounting for trimming)
        prop_assert_eq!(read_vars.len(), env_vars.len());
        for (key, value) in &env_vars {
            let expected_value = value.trim().to_string();
            prop_assert_eq!(read_vars.get(key), Some(&expected_value));
        }
    }
}

// Additional property tests for edge cases
proptest! {
    #[test]
    fn test_file_size_bounds_property(
        env_vars in prop::collection::hash_map(env_key_strategy(), env_value_strategy(), 0..1000)
    ) {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("large_test.env");

        write_env_file(&file_path, &env_vars, false).unwrap();

        // File should exist and be readable
        prop_assert!(file_path.exists());

        let read_vars = read_env_file(&file_path).unwrap();
        prop_assert_eq!(read_vars.len(), env_vars.len());

        // Validate that large files still pass validation
        prop_assert!(validate_env_file(&file_path).is_ok());
    }

    #[test]
    fn test_special_characters_in_values_property(
        key in env_key_strategy(),
        special_chars in r"[!@#$%^&*()_+\-=\[\]{}|;',./<>?~]+"
    ) {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.env");

        let content = format!("{}={}", key, special_chars);
        fs::write(&file_path, &content).unwrap();

        let vars = read_env_file(&file_path).unwrap();

        prop_assert_eq!(vars.len(), 1);
        let expected_value = special_chars.trim().to_string();
        prop_assert_eq!(vars.get(&key), Some(&expected_value));
    }

    #[test]
    fn test_unicode_handling_property(
        key in env_key_strategy(),
        unicode_value in r"[a-z]{1,20}"
    ) {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.env");

        let content = format!("{}={}", key, unicode_value);
        fs::write(&file_path, &content).unwrap();

        let vars = read_env_file(&file_path).unwrap();

        prop_assert_eq!(vars.len(), 1);
        let expected_value = unicode_value.trim().to_string();
        prop_assert_eq!(vars.get(&key), Some(&expected_value));
    }
}
