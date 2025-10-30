use std::path::PathBuf;
use clap::{Parser, Subcommand};
use anyhow::Result;

mod setup;
mod verify;
mod modify;
mod constants;
mod create;
mod tui;

#[derive(Parser)]
#[command(name = "distromanifesto")]
#[command(author = "")]
#[command(version = "1.0")]
#[command(about = "A friendly wizard for creating and managing distrobox manifest files.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
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
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Ensure the ~/.distromanifesto directory and its subdirectories exist.
    setup::ensure_hidden_dir()?;

    match cli.command {
        // Some(Commands::Wizard) => {
        //     wizard::launch_wizard()?;
        // }
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
        None => {
            // Default to the wizard if no subcommand is provided
            create::launch_creator()?
        }
    }

    Ok(())
}