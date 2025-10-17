use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Bitwarden CLI not found. Please install it first and make sure it's in your PATH.")]
    BitwardenNotFound,

    #[error("Bitwarden authentication failed. Make sure the desktop app is running and you're logged in.")]
    BitwardenAuthFailed,

    #[error("Bitwarden session error: {0}")]
    BitwardenSessionError(String),

    #[error("Failed to read .env file: {0}")]
    EnvFileReadError(String),

    #[error("Failed to write .env file: {0}")]
    EnvFileWriteError(String),

    #[error("Invalid .env file format: {0}")]
    EnvFileFormatError(String),

    #[error("Environment variable error: {0}")]
    EnvVarError(String),

    #[error("Item not found in Bitwarden: {0}")]
    ItemNotFound(String),

    #[error("Folder not found in Bitwarden: {0}")]
    FolderNotFound(String),

    #[error("Command execution failed: {0}")]
    CommandExecutionError(String),

    #[error("Invalid command arguments: {0}")]
    InvalidArguments(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Unknown(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Unknown(format!("JSON error: {}", err))
    }
}

impl From<std::string::FromUtf8Error> for AppError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        AppError::Unknown(format!("UTF-8 conversion error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_error_display() {
        let errors = vec![
            AppError::BitwardenNotFound,
            AppError::BitwardenAuthFailed,
            AppError::BitwardenSessionError("session expired".to_string()),
            AppError::EnvFileReadError("file not found".to_string()),
            AppError::EnvFileWriteError("permission denied".to_string()),
            AppError::EnvFileFormatError("invalid format".to_string()),
            AppError::EnvVarError("missing variable".to_string()),
            AppError::ItemNotFound("test-item".to_string()),
            AppError::FolderNotFound("test-folder".to_string()),
            AppError::CommandExecutionError("command failed".to_string()),
            AppError::InvalidArguments("invalid args".to_string()),
            AppError::Unknown("unknown error".to_string()),
        ];

        for error in errors {
            let display = format!("{}", error);
            assert!(!display.is_empty());
        }
    }

    #[test]
    fn test_bitwarden_not_found_error() {
        let error = AppError::BitwardenNotFound;
        assert_eq!(
            error.to_string(),
            "Bitwarden CLI not found. Please install it first and make sure it's in your PATH."
        );
    }

    #[test]
    fn test_bitwarden_auth_failed_error() {
        let error = AppError::BitwardenAuthFailed;
        assert_eq!(
            error.to_string(),
            "Bitwarden authentication failed. Make sure the desktop app is running and you're logged in."
        );
    }

    #[test]
    fn test_bitwarden_session_error() {
        let message = "Session token expired";
        let error = AppError::BitwardenSessionError(message.to_string());
        assert_eq!(
            error.to_string(),
            format!("Bitwarden session error: {}", message)
        );
    }

    #[test]
    fn test_env_file_read_error() {
        let message = "File not found: /path/to/file.env";
        let error = AppError::EnvFileReadError(message.to_string());
        assert_eq!(
            error.to_string(),
            format!("Failed to read .env file: {}", message)
        );
    }

    #[test]
    fn test_env_file_write_error() {
        let message = "Permission denied";
        let error = AppError::EnvFileWriteError(message.to_string());
        assert_eq!(
            error.to_string(),
            format!("Failed to write .env file: {}", message)
        );
    }

    #[test]
    fn test_env_file_format_error() {
        let message = "Missing equals sign on line 5";
        let error = AppError::EnvFileFormatError(message.to_string());
        assert_eq!(
            error.to_string(),
            format!("Invalid .env file format: {}", message)
        );
    }

    #[test]
    fn test_env_var_error() {
        let message = "Missing required environment variable: DATABASE_URL";
        let error = AppError::EnvVarError(message.to_string());
        assert_eq!(
            error.to_string(),
            format!("Environment variable error: {}", message)
        );
    }

    #[test]
    fn test_item_not_found_error() {
        let item_name = "my-project-secrets";
        let error = AppError::ItemNotFound(item_name.to_string());
        assert_eq!(
            error.to_string(),
            format!("Item not found in Bitwarden: {}", item_name)
        );
    }

    #[test]
    fn test_folder_not_found_error() {
        let folder_name = "Development/MyProject";
        let error = AppError::FolderNotFound(folder_name.to_string());
        assert_eq!(
            error.to_string(),
            format!("Folder not found in Bitwarden: {}", folder_name)
        );
    }

    #[test]
    fn test_command_execution_error() {
        let message = "Process exited with code 1";
        let error = AppError::CommandExecutionError(message.to_string());
        assert_eq!(
            error.to_string(),
            format!("Command execution failed: {}", message)
        );
    }

    #[test]
    fn test_invalid_arguments_error() {
        let message = "Both --name and --folder cannot be empty";
        let error = AppError::InvalidArguments(message.to_string());
        assert_eq!(
            error.to_string(),
            format!("Invalid command arguments: {}", message)
        );
    }

    #[test]
    fn test_unknown_error() {
        let message = "Something unexpected happened";
        let error = AppError::Unknown(message.to_string());
        assert_eq!(error.to_string(), format!("Unknown error: {}", message));
    }

    #[test]
    fn test_from_io_error() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let app_error = AppError::from(io_error);

        match app_error {
            AppError::Unknown(msg) => assert!(msg.contains("File not found")),
            _ => panic!("Expected Unknown error variant"),
        }
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let app_error = AppError::from(json_error);

        match app_error {
            AppError::Unknown(msg) => assert!(msg.contains("JSON error")),
            _ => panic!("Expected Unknown error variant"),
        }
    }

    #[test]
    fn test_from_utf8_error() {
        let utf8_error = String::from_utf8(vec![0, 159, 146, 150]).unwrap_err();
        let app_error = AppError::from(utf8_error);

        match app_error {
            AppError::Unknown(msg) => assert!(msg.contains("UTF-8 conversion error")),
            _ => panic!("Expected Unknown error variant"),
        }
    }

    #[test]
    fn test_error_debug_trait() {
        let error = AppError::BitwardenNotFound;
        let debug_output = format!("{:?}", error);
        assert!(debug_output.contains("BitwardenNotFound"));
    }

    #[test]
    fn test_error_source() {
        use std::error::Error;

        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
        let app_error = AppError::from(io_error);

        // Test that the error can be treated as a standard Error trait object
        let _: &dyn Error = &app_error;
    }
}
