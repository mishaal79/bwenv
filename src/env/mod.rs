//! Environment module - .env file parsing and writing
//!
//! Re-exports the preserved env_file parser with updated API.

pub mod parser;

// Re-export main functions
pub use parser::{read_env_file, validate_env_file, write_env_file};
