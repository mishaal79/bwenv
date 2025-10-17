//! Validate command - Check .env format and completeness
//!
//! Validates .env file format.

use crate::env::parser;
use crate::{AppError, Result};

pub async fn execute(input: &str) -> Result<()> {
    parser::validate_env_file(input)
        .map_err(|e| AppError::EnvFileFormatError(format!("Validation failed: {}", e)))?;

    println!("✓ {} is valid", input);
    Ok(())
}
