use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use log::{debug, error, info};
use std::path::PathBuf;
use std::process::Command;

use bwenv::{bitwarden, env_file, logging::{self, Verbosity}};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Increase verbosity (can be used multiple times)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Quiet mode, suppress output except errors
    #[arg(short, long)]
    quiet: bool,

    /// Print log messages to stderr without timestamps or level info
    #[arg(short = 'l', long)]
    log_stderr: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Store secrets from a .env file to Bitwarden
    Store {
        /// Path to the .env file to store
        #[arg(short, long)]
        file: PathBuf,

        /// Bitwarden folder path to store secrets in
        #[arg(long)]
        folder: Option<String>,

        /// Name for this environment set (defaults to folder name)
        #[arg(short, long)]
        name: Option<String>,

        /// Overwrite existing secrets with the same name
        #[arg(long)]
        overwrite: bool,
    },

    /// Retrieve secrets from Bitwarden and create a .env file
    Retrieve {
        /// Bitwarden folder path to retrieve secrets from
        #[arg(long)]
        folder: Option<String>,

        /// Name of the environment set to retrieve
        #[arg(short, long)]
        name: Option<String>,

        /// Output file path (defaults to .env)
        #[arg(short, long, default_value = ".env")]
        output: PathBuf,

        /// Merge with existing .env file instead of overwriting
        #[arg(long)]
        merge: bool,
    },

    /// List all stored environment sets
    List {
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging based on verbosity level
    let verbosity = Verbosity::from_count(cli.verbose);
    logging::initialize(verbosity, cli.quiet)
        .context("Failed to initialize logging system")?;

    // Log startup information
    info!("bwenv v{}", env!("CARGO_PKG_VERSION"));
    debug!("Verbosity level: {:?}", verbosity);

    // Check if Bitwarden is installed and accessible
    verify_bitwarden_installed()
        .context("Bitwarden CLI check failed")?;
    
    // Get session token from the Bitwarden desktop app
    info!("Getting Bitwarden session token from desktop app");
    let session_token = bitwarden::get_session_token()
        .context("Failed to get Bitwarden session token")?;
    
    debug!("Successfully obtained Bitwarden session token");

    // Execute the command
    match cli.command {
        Commands::Store {
            file,
            folder,
            name,
            overwrite,
        } => {
            info!("Executing STORE command");
            debug!("File: {:?}, Folder: {:?}, Name: {:?}, Overwrite: {}", 
                  file, folder, name, overwrite);

            // Read .env file
            info!("Reading .env file: {:?}", file);
            let secrets = env_file::read_env_file(&file)
                .context(format!("Failed to read .env file: {:?}", file))?;
            
            debug!("Successfully read {} environment variables from {:?}", secrets.len(), file);

            // Determine storage name
            let storage_name = name.unwrap_or_else(|| {
                folder
                    .clone()
                    .unwrap_or_else(|| file.file_stem().unwrap().to_string_lossy().to_string())
            });

            // Store secrets in Bitwarden
            info!("Storing secrets in Bitwarden as '{}'", storage_name);
            bitwarden::store_secrets(&session_token, &secrets, &storage_name, folder.as_deref(), overwrite)
                .context("Failed to store secrets in Bitwarden")?;

            info!("Successfully stored {} secrets in Bitwarden", secrets.len());
        }
        Commands::Retrieve {
            folder,
            name,
            output,
            merge,
        } => {
            info!("Executing RETRIEVE command");
            debug!("Folder: {:?}, Name: {:?}, Output: {:?}, Merge: {}", 
                  folder, name, output, merge);

            // Determine storage name
            let storage_name = name.unwrap_or_else(|| {
                folder
                    .clone()
                    .unwrap_or_else(|| output.file_stem().unwrap().to_string_lossy().to_string())
            });

            // Retrieve secrets from Bitwarden
            info!("Retrieving secrets from Bitwarden for '{}'", storage_name);
            let secrets = bitwarden::retrieve_secrets(&session_token, &storage_name, folder.as_deref())
                .context("Failed to retrieve secrets from Bitwarden")?;
            
            debug!("Successfully retrieved {} secrets from Bitwarden", secrets.len());

            // Write to .env file
            info!("Writing secrets to file: {:?}", output);
            env_file::write_env_file(&output, &secrets, merge)
                .context(format!("Failed to write to .env file: {:?}", output))?;

            info!("Successfully wrote {} secrets to {:?}", secrets.len(), output);
        }
        Commands::List { format } => {
            info!("Executing LIST command with format: {}", format);

            // List all environment sets
            info!("Fetching environment sets from Bitwarden");
            let env_sets = bitwarden::list_env_sets(&session_token)
                .context("Failed to list environment sets")?;
            
            debug!("Found {} environment sets", env_sets.len());

            if format == "json" {
                println!("{}", serde_json::to_string_pretty(&env_sets)?);
            } else {
                if env_sets.is_empty() {
                    println!("No environment sets found in Bitwarden");
                } else {
                    println!("Environment Sets:");
                    for (idx, set) in env_sets.iter().enumerate() {
                        println!("{:>3}. {} ({}) - {} variables", 
                                idx + 1, 
                                set.name, 
                                set.folder.as_deref().unwrap_or("No Folder"), 
                                set.items_count);
                    }
                }
            }
            info!("List command completed successfully");
        }
    }

    Ok(())
}

fn verify_bitwarden_installed() -> Result<()> {
    info!("Verifying Bitwarden CLI installation");
    
    let output = Command::new("bw")
        .arg("--version")
        .output()
        .context("Failed to execute Bitwarden CLI. Is it installed and in your PATH?")?;

    if !output.status.success() {
        error!("Bitwarden CLI check failed");
        anyhow::bail!("Bitwarden CLI not found. Please install it first.");
    }

    if let Ok(version) = String::from_utf8(output.stdout) {
        debug!("Bitwarden CLI version: {}", version.trim());
    }

    info!("Bitwarden CLI installation verified");
    Ok(())
}