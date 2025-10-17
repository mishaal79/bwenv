//! Common testing utilities and fixtures
//!
//! This module provides shared testing infrastructure including:
//! - Test fixtures and builders
//! - Custom assertions
//! - Environment guards
//! - Logging initialization

pub mod fixtures;
pub mod helpers;

// Re-export commonly used testing utilities
pub use fixtures::{EnvFileBuilder, TestProject};
pub use helpers::{assert_env_files_equivalent, init_test_logging, EnvGuard};

// Type aliases for convenience
pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
