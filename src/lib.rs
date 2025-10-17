//! bwenv - Bitwarden Secrets Manager .env CLI
//!
//! A developer-friendly CLI for managing .env files using Bitwarden Secrets Manager.
//! Built with the official Bitwarden Rust SDK for native performance and security.

pub mod bitwarden;
pub mod cli;
pub mod commands;
pub mod config;
pub mod env;
pub mod error;
pub mod logging;
pub mod sync;

// Re-export commonly used types
pub use error::types::AppError;
pub type Result<T> = std::result::Result<T, AppError>;
