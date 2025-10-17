//! bwenv - Main entry point
//!
//! Command-line interface for Bitwarden Secrets Manager .env management.

use bwenv::cli;
use bwenv::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    bwenv::logging::init()?;

    // Run CLI
    cli::run().await
}
