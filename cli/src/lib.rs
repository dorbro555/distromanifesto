// In cli/src/lib.rs

use std::path::PathBuf;
use clap::{Parser, Subcommand};
use anyhow::Result;

// All your modules are declared here now
mod cauldron;
mod setup;
mod verify;
mod modify;
mod constants;
mod create;
mod tui;
mod assemble;

#[derive(Debug, Parser)]
#[command(
    name = "distromanifesto",
    version,
    about = "A friendly wizard for creating and managing distrobox manifest files.",
    visible_alias = "dimo",
    author = ""
)]
pub struct Cli { // <-- Made pub
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands { // <-- Made pub and added Debug
    /// Create a new manifest with an interactive wizard
    Create,
    /// Verify the syntax of a manifest file
    Verify {
        /// The path to the manifest file to verify
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
    /// Modify an existing manifest file in an interactive TUI
    Modify {
        /// The path to the manifest file to modify
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
    Cauldron,
    /// Assemble containers from a manifest file
    Assemble { // <-- NEW
        /// The name of the manifest file (e.g., 'dev' or 'dev.ini')
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
}

// All your main logic is now in this public function
pub fn run() -> Result<()> {
    let cli = Cli::parse();

    // Ensure the ~/.distromanifesto directory and its subdirectories exist.
    setup::ensure_hidden_dir()?;

    match cli.command {
        Some(Commands::Create) => {
            create::launch_creator()?
        }
        Some(Commands::Verify { file }) => {
            let content = std::fs::read_to_string(&file)?;
            match verify::verify_manifest(&content) {
                Ok(_) => println!("✅ Manifest at '{}' is valid.", file.display()),
                Err(e) => eprintln!("❌ Manifest validation failed: {}", e),
            }
        }
        Some(Commands::Modify { file }) => {
            modify::launch_editor(&file)?;
        }
        Some(Commands::Cauldron) => {
            cauldron::launch_tui()?
        }
        Some(Commands::Assemble { file }) => {
            // We can optionally support looking in the ~/.distromanifesto/manifests folder
            // if the user provides just a name like "dev" instead of a path.
            // For now, let's assume they might pass a name, so we use your helper setup logic if needed.
            // Or, purely based on file path provided:
            if file.exists() {
                assemble::create_containers_from_file(&file)?;
            } else {
                // Try to resolve it from the manifest store
                let path_str = format!("~/.distromanifesto/manifests/{}", file.display());
                let resolved_path = setup::get_full_path_from_str(&path_str)?;
                if resolved_path.exists() {
                    assemble::create_containers_from_file(&resolved_path)?;
                } else {
                    anyhow::bail!("Manifest file not found: {}", file.display());
                }
            }
        }
        None => {
            // Default to the create command if no subcommand is provided
            create::launch_creator()?
        }
    }

    Ok(())
}