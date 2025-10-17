//! Bitwarden module - Secrets Manager SDK integration
//!
//! Provides high-level API for interacting with Bitwarden Secrets Manager.

pub mod provider;
pub mod sdk_provider;

#[cfg(test)]
pub mod mock_provider;

// Re-export commonly used types
pub use provider::{Project, Secret, SecretsProvider};
pub use sdk_provider::SdkProvider;

#[cfg(test)]
pub use mock_provider::MockProvider;
