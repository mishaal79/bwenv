//! Error module - Error types and handling
//!
//! Re-exports preserved error types with updates for Secrets Manager.

pub mod types;

// Re-export main error type
pub use types::AppError;
