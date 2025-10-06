use std::path::PathBuf;
use clap::{Parser, Subcommand};
// use distromanifesto::{
    // ensure_hidden_dir, 
    // launch_wizard};
use anyhow::Result;

mod setup;
mod wizard;
mod verify;

#[derive(Parser)]
#[command(name = "distromanifesto")]
#[command(about = "A TUI and CLI wizard for creating Distrobox manifest files", long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    /// Path to a config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on (use multiple times for more verbosity)
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    /// Subcommands
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Runs the interactive manifest wizard
    Wizard,
    // /// Verifies a manifest file’s syntax
    // Verify {
    //     /// Path to manifest file
    //     #[arg(value_name = "FILE")]
    //     file: PathBuf,
    // },
    // /// Placeholder test command
    // Test {
    //     /// Lists test values
    //     #[arg(short, long)]
    //     list: bool,
    // },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Handle debug verbosity
    // match cli.debug {
    //     0 => {}
    //     1 => println!("Debug mode: basic logging enabled"),
    //     2 => println!("Debug mode: verbose output"),
    //     _ => println!("Debug mode: insane verbosity (good luck)"),
    // }

    // Ensure ~/.distromanifesto structure exists
    // let hidden_dir = ensure_hidden_dir()?;

    // Match subcommands
    match cli.command {
        // Some(Commands::Wizard { filepath }) => {
        //     println!("Launching wizard...");
        //     launch_wizard(PathBuf::from(filepath))?;
        // }
        // Some(Commands::Verify { file }) => {
        //     println!("Verifying manifest file: {:?}", file);
        //     // TODO: Add syntax checking logic in manifest.rs
        // }
        // Some(Commands::Test { list }) => {
        //     if *list {
        //         println!("Test command executed: listing items...");
        //     } else {
        //         println!("Test command executed: no list flag provided.");
        //     }
        // }
        // None => {
        //     // If no subcommand, default to wizard
        //     println!("No subcommand provided — launching wizard by default.");
        //     launch_wizard(hidden_dir)?;
        // }
        Commands::Wizard => wizard::launch_wizard()?,
    }

    Ok(())
}
