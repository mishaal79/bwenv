//! Init command - Initialize project with .bwenv.toml
//!
//! Creates configuration file and sets up project for bwenv management.

use crate::Result;
use std::fs;
use std::path::Path;

pub async fn execute() -> Result<()> {
    let config_path = Path::new(".bwenv.toml");

    if config_path.exists() {
        println!("⚠️  .bwenv.toml already exists");
        println!("   Use --force to overwrite (not yet implemented)");
        return Ok(());
    }

    let config_content = r#"# bwenv Configuration
# This file configures bwenv for your project

# Default Bitwarden project for this repository
# You can override this with --project flag
default_project = "MyProject"

# Default .env file location
env_file = ".env"

# Automatically sync on pull
auto_sync = false

# Show secrets in status output (WARNING: insecure)
show_secrets = false
"#;

    fs::write(config_path, config_content)?;

    println!("✓ Created .bwenv.toml configuration file");
    println!();
    println!("Next steps:");
    println!("  1. Edit .bwenv.toml and set your default project");
    println!("  2. Run 'bwenv push' to upload your .env to Bitwarden");
    println!("  3. Add .bwenv.toml to git (safe to commit)");
    println!("  4. Add .env to .gitignore (contains secrets)");

    Ok(())
}
