//! bwenv - Main entry point
//!
//! Command-line interface for Bitwarden Secrets Manager .env management.

use bwenv::cli;
use bwenv::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging (will be called from CLI run when implemented)
    // bwenv::logging::initialize()?;

    // Run CLI
    cli::run().await
}
