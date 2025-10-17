//! End-to-End Testing Module
//!
//! This module provides comprehensive E2E testing against a real Bitwarden Secrets Manager instance.
//! Tests run against Bitwarden Cloud (free tier) using a real access token.
//!
//! ## Setup
//!
//! 1. Create a Bitwarden account and enable Secrets Manager
//! 2. Generate an access token
//! 3. Create `.env.test` with: `BITWARDEN_ACCESS_TOKEN=your_token_here`
//! 4. Run tests: `./scripts/run-e2e-tests.sh`

pub mod setup;
pub mod cli_tests;

// Re-export commonly used types
pub use setup::{TestContext, TestResult};
