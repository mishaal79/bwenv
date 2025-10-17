use anyhow::{Context, Result};
use chrono::Local;
use fern::{Dispatch, InitError};
use log::{debug, error, info, trace, warn, LevelFilter};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Log verbosity levels following GNU/Linux conventions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verbosity {
    Quiet,   // No output except errors
    Normal,  // Default level - warnings and errors
    Verbose, // More information - info, warnings, and errors
    Debug,   // Debug information - debug, info, warnings, and errors
    Trace,   // Trace information - all log messages
}

impl Verbosity {
    /// Convert verbosity to log::LevelFilter
    pub fn to_level_filter(&self) -> LevelFilter {
        match self {
            Verbosity::Quiet => LevelFilter::Error,
            Verbosity::Normal => LevelFilter::Warn,
            Verbosity::Verbose => LevelFilter::Info,
            Verbosity::Debug => LevelFilter::Debug,
            Verbosity::Trace => LevelFilter::Trace,
        }
    }

    /// Create from command-line verbosity count
    pub fn from_count(count: u8) -> Self {
        match count {
            0 => Verbosity::Normal,
            1 => Verbosity::Verbose,
            2 => Verbosity::Debug,
            _ => Verbosity::Trace,
        }
    }
}

/// Returns the path to the log directory
pub fn get_log_directory() -> PathBuf {
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

    // Create standard XDG-compliant log directory
    let log_dir = if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
        // Follow XDG Base Directory Specification for Linux/macOS
        let xdg_data_home = env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home_dir.join(".local/share"));
        xdg_data_home.join("bwenv/logs")
    } else {
        // Windows or other OS
        home_dir.join(".bwenv/logs")
    };

    // Create directory if it doesn't exist
    if !log_dir.exists() {
        let _ = fs::create_dir_all(&log_dir);
    }

    log_dir
}

/// Returns the path to the current log file
pub fn get_log_file_path() -> PathBuf {
    let log_dir = get_log_directory();
    let date = Local::now().format("%Y-%m-%d");
    log_dir.join(format!("bwenv-{}.log", date))
}

/// Initialize logging with the specified verbosity level
pub fn initialize(verbosity: Verbosity, quiet: bool) -> Result<(), InitError> {
    // Override verbosity if quiet is specified
    let level_filter = if quiet {
        LevelFilter::Error
    } else {
        verbosity.to_level_filter()
    };

    // Get log file path
    let log_file_path = get_log_file_path();

    // Set up log rotation
    let log_dir = get_log_directory();
    rotate_logs(&log_dir)?;

    // Configure logging
    let mut dispatch = Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(level_filter);

    // Add file logger
    let file_config = Dispatch::new()
        .level(LevelFilter::Debug) // Always log debug level to file
        .chain(fern::log_file(log_file_path)?);

    // Add console logger with colors if not quiet
    if !quiet {
        let stderr_config = Dispatch::new().level(level_filter).chain(io::stderr());

        dispatch = dispatch.chain(stderr_config);
    }

    dispatch = dispatch.chain(file_config);

    // Apply configuration
    dispatch.apply()?;

    // Log initialization
    debug!("Logging initialized with level: {:?}", level_filter);
    trace!("Log file location: {:?}", get_log_file_path());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_verbosity_to_level_filter() {
        assert_eq!(Verbosity::Quiet.to_level_filter(), LevelFilter::Error);
        assert_eq!(Verbosity::Normal.to_level_filter(), LevelFilter::Warn);
        assert_eq!(Verbosity::Verbose.to_level_filter(), LevelFilter::Info);
        assert_eq!(Verbosity::Debug.to_level_filter(), LevelFilter::Debug);
        assert_eq!(Verbosity::Trace.to_level_filter(), LevelFilter::Trace);
    }

    #[test]
    fn test_verbosity_from_count() {
        assert_eq!(Verbosity::from_count(0), Verbosity::Normal);
        assert_eq!(Verbosity::from_count(1), Verbosity::Verbose);
        assert_eq!(Verbosity::from_count(2), Verbosity::Debug);
        assert_eq!(Verbosity::from_count(3), Verbosity::Trace);
        assert_eq!(Verbosity::from_count(10), Verbosity::Trace); // Any value > 2 should be Trace
    }

    #[test]
    fn test_verbosity_debug_clone_copy() {
        let v1 = Verbosity::Verbose;
        let v2 = v1; // Copy
        let v3 = v1.clone(); // Clone

        assert_eq!(v1, v2);
        assert_eq!(v1, v3);
        assert_eq!(v2, v3);
    }

    #[test]
    fn test_verbosity_equality() {
        assert_eq!(Verbosity::Quiet, Verbosity::Quiet);
        assert_ne!(Verbosity::Quiet, Verbosity::Normal);
        assert_ne!(Verbosity::Normal, Verbosity::Verbose);
        assert_ne!(Verbosity::Verbose, Verbosity::Debug);
        assert_ne!(Verbosity::Debug, Verbosity::Trace);
    }

    #[test]
    fn test_get_log_directory_linux_macos() {
        if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
            let log_dir = get_log_directory();
            let path_str = log_dir.to_string_lossy();

            // Should either be in XDG_DATA_HOME or ~/.local/share
            assert!(
                path_str.contains(".local/share/bwenv/logs") || path_str.contains("bwenv/logs")
            );
        }
    }

    #[test]
    fn test_get_log_directory_windows() {
        if cfg!(target_os = "windows") {
            let log_dir = get_log_directory();
            let path_str = log_dir.to_string_lossy();
            assert!(path_str.contains(".bwenv\\logs"));
        }
    }

    #[test]
    fn test_get_log_file_path() {
        let log_file = get_log_file_path();
        let filename = log_file.file_name().unwrap().to_string_lossy();

        // Should match pattern: bwenv-YYYY-MM-DD.log
        assert!(filename.starts_with("bwenv-"));
        assert!(filename.ends_with(".log"));

        // Check date format (basic validation)
        // Should be in YYYY-MM-DD format (10 characters with 2 dashes)
        let date_part = filename
            .strip_prefix("bwenv-")
            .unwrap()
            .strip_suffix(".log")
            .unwrap();

        // Should be in YYYY-MM-DD format (10 characters with 2 dashes)
        assert_eq!(date_part.len(), 10);
        assert_eq!(date_part.chars().filter(|&c| c == '-').count(), 2);
    }

    #[test]
    fn test_rotate_logs_empty_directory() {
        let temp_dir = tempdir().unwrap();
        let log_dir = temp_dir.path();

        // Test with empty directory
        let result = rotate_logs(log_dir);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rotate_logs_nonexistent_directory() {
        let temp_dir = tempdir().unwrap();
        let log_dir = temp_dir.path().join("nonexistent");

        // Should create directory and succeed
        let result = rotate_logs(&log_dir);
        assert!(result.is_ok());
        assert!(log_dir.exists());
    }

    #[test]
    fn test_rotate_logs_with_files() {
        let temp_dir = tempdir().unwrap();
        let log_dir = temp_dir.path();

        // Create some mock log files
        for i in 0..15 {
            let filename = format!("bwenv-2023-01-{:02}.log", i + 1);
            let file_path = log_dir.join(filename);
            fs::write(file_path, "test log content").unwrap();
        }

        // Create some non-log files (should be ignored)
        fs::write(log_dir.join("other.txt"), "not a log").unwrap();
        fs::write(log_dir.join("readme.md"), "documentation").unwrap();

        let result = rotate_logs(log_dir);
        assert!(result.is_ok());

        // Count remaining log files
        let log_files: Vec<_> = fs::read_dir(log_dir)
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("log") {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        // Should have at most 10 log files remaining
        assert!(log_files.len() <= 10);

        // Non-log files should still exist
        assert!(log_dir.join("other.txt").exists());
        assert!(log_dir.join("readme.md").exists());
    }

    #[test]
    fn test_rotate_logs_with_few_files() {
        let temp_dir = tempdir().unwrap();
        let log_dir = temp_dir.path();

        // Create only 3 log files (less than the limit of 10)
        for i in 0..3 {
            let filename = format!("bwenv-2023-01-{:02}.log", i + 1);
            let file_path = log_dir.join(filename);
            fs::write(file_path, "test log content").unwrap();
        }

        let result = rotate_logs(log_dir);
        assert!(result.is_ok());

        // All files should remain
        let log_files: Vec<_> = fs::read_dir(log_dir)
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("log") {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(log_files.len(), 3);
    }

    #[test]
    fn test_xdg_data_home_environment_variable() {
        let temp_dir = tempdir().unwrap();
        let custom_data_home = temp_dir.path().join("custom_data");

        // Set XDG_DATA_HOME environment variable
        env::set_var("XDG_DATA_HOME", &custom_data_home);

        if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
            let log_dir = get_log_directory();
            let expected = custom_data_home.join("bwenv/logs");
            assert_eq!(log_dir, expected);
        }

        // Clean up
        env::remove_var("XDG_DATA_HOME");
    }

    // Note: Testing initialize() function would require more complex setup
    // due to global logger state, but we test its components above
}

/// Rotate logs - keep only the latest 10 log files
fn rotate_logs(log_dir: &Path) -> Result<(), io::Error> {
    // Get all log files
    let entries = match fs::read_dir(log_dir) {
        Ok(entries) => entries,
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                // Directory doesn't exist, try to create it
                fs::create_dir_all(log_dir)?;
                return Ok(());
            }
            return Err(e);
        }
    };

    let mut log_files: Vec<(PathBuf, SystemTime)> = Vec::new();

    // Collect log files with their modification times
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("log") {
            if let Ok(metadata) = fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    log_files.push((path, modified));
                }
            }
        }
    }

    // Sort by modification time (oldest first)
    log_files.sort_by(|a, b| a.1.cmp(&b.1));

    // Keep only the 10 most recent logs
    let files_to_delete = log_files.len().saturating_sub(10);
    for i in 0..files_to_delete {
        let _ = fs::remove_file(&log_files[i].0);
    }

    Ok(())
}
