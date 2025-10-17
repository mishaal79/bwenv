//! CLI module - Command-line interface definition and routing
//!
//! This module handles argument parsing and command dispatch.

use crate::bitwarden::sdk_provider::SdkProvider;
use crate::commands;
use crate::{AppError, Result};
use clap::{Parser, Subcommand};

/// bwenv - Bitwarden Secrets Manager .env CLI
///
/// Manage your .env files using Bitwarden Secrets Manager
#[derive(Parser, Debug)]
#[command(name = "bwenv")]
#[command(about = "Manage .env files with Bitwarden Secrets Manager", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Pull secrets from Bitwarden to .env file
    Pull {
        /// Project name or ID in Bitwarden
        #[arg(short, long)]
        project: String,

        /// Output file path (default: .env)
        #[arg(short, long, default_value = ".env")]
        output: String,

        /// Overwrite existing file
        #[arg(long)]
        force: bool,
    },

    /// Push .env file secrets to Bitwarden
    Push {
        /// Project name or ID in Bitwarden
        #[arg(short, long)]
        project: String,

        /// Input .env file path (default: .env)
        #[arg(short, long, default_value = ".env")]
        input: String,

        /// Overwrite existing secrets
        #[arg(long)]
        overwrite: bool,
    },

    /// List projects and secrets
    List {
        /// List secrets in a specific project
        #[arg(short, long)]
        project: Option<String>,
    },

    /// Initialize configuration
    Init,

    /// Show status of current project
    Status {
        /// Project name or ID
        #[arg(short, long)]
        project: String,

        /// Path to .env file to compare
        #[arg(short, long)]
        env_file: Option<String>,
    },

    /// Validate .env file format
    Validate {
        /// Input .env file path (default: .env)
        #[arg(short, long, default_value = ".env")]
        input: String,
    },
}

/// Run the CLI application
pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    // Get access token from environment
    let access_token =
        std::env::var("BITWARDEN_ACCESS_TOKEN").map_err(|_| AppError::BitwardenAuthFailed)?;

    // Create SDK provider
    let provider = SdkProvider::new(access_token).await?;

    // Dispatch to command handlers
    match cli.command {
        Commands::Pull {
            project,
            output,
            force,
        } => commands::pull::execute(provider, &project, &output, force).await,
        Commands::Push {
            project,
            input,
            overwrite,
        } => commands::push::execute(provider, &project, &input, overwrite).await,
        Commands::List { project } => commands::status::list(provider, project.as_deref()).await,
        Commands::Init => commands::init::execute().await,
        Commands::Status { project, env_file } => {
            commands::status::execute(provider, &project, env_file.as_deref()).await
        }
        Commands::Validate { input } => commands::validate::execute(&input).await,
    }
}
