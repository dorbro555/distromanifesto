use std::path::PathBuf;
use std::fs;

use clap::{arg, command, Parser, Subcommand};
use inquire::{Text, Confirm, MultiSelect, validator::{Validation}};

#[derive(Parser)]
#[command(
    name = "distromanifesto",
    version,
    about = "A CLI wizard for creating and managing Distrobox manifest files",
    long_about = None
)]
struct Args {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on (-v, -vv, etc.)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the interactive wizard to create a manifest file
    Wizard {
        /// Output directory or filename
        #[arg(short, long, default_value = "./assemble.ini")]
        output: String,
    },
    /// Verify syntax and structure of a manifest file
    Verify {
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
    /// Edit an existing manifest file (TUI planned)
    Modify {
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
}

fn main() {
    let args = Args::parse();

    if args.verbose > 0 {
        println!("Verbosity level: {}", args.verbose);
    }

    match &args.command {
        Some(Commands::Wizard { output }) => {
            println!("Starting the Distromanifesto wizard...");
            if let Err(e) = run_wizard(output) {
                eprintln!("❌ Wizard failed: {}", e);
            }
        }
        Some(Commands::Verify { file }) => {
            println!("Verifying manifest: {}", file.display());
            // TODO: Implement verify logic
        }
        Some(Commands::Modify { file }) => {
            println!("Opening manifest in TUI editor: {}", file.display());
            // TODO: Implement modify logic
        }
        None => {
            // If no subcommand provided, default to wizard
            println!("No subcommand given, starting wizard by default...");
            if let Err(e) = run_wizard("./assemble.ini") {
                eprintln!("❌ Wizard failed: {}", e);
            }
        }
    }
}

fn run_wizard(output_path: &str) -> Result<(), String> {
    println!("Welcome to the Distro Manifesto Wizard!");

    let container_name = text_prompt("Container name: ")?;
    let base_image = text_prompt("Base image (e.g. archlinux:latest): ")?;
    let init_cmd = text_prompt("Init command (e.g. bash): ")?;
    let home_dir = text_prompt("Container home directory (default: /home/user): ")?;

    let enabled_flags = multiselect_prompt("Select flags to enable:")?;

    // Assemble the manifest
    let mut manifest = String::new();
    manifest.push_str(&format!("[{}]\n", container_name));
    manifest.push_str(&format!("image=\"{}\"\n", base_image));
    manifest.push_str(&format!("init=\"{}\"\n", init_cmd));
    manifest.push_str(&format!("home=\"{}\"\n", home_dir));

    for flag in &enabled_flags {
        manifest.push_str(&format!("{}=true\n", flag));
    }

    println!("\n──────────────────────────────");
    println!("Preview manifest:\n{}\n", manifest);
    println!("──────────────────────────────");

    if confirm_prompt("Save this manifest?")? {
        fs::write(output_path, manifest)
            .map_err(|e| format!("Failed to write file: {}", e))?;
        println!("✅ Manifest saved to {}", output_path);
    } else {
        println!("Manifest not saved.");
    }

    Ok(())
}

// -------------------- Prompt Helpers --------------------

fn text_prompt(prompt: &str) -> Result<String, String> {
    let validator = |input: &str| {
        if input.trim().is_empty() {
            Ok(Validation::Invalid("Input cannot be empty.".into()))
        } else if input.len() > 140 {
            Ok(Validation::Invalid("Input too long (max 140 chars).".into()))
        } else {
            Ok(Validation::Valid)
        }
    };

    Text::new(prompt)
        .with_validator(validator)
        .prompt()
        .map_err(|e| format!("Prompt failed: {}", e))
}

fn confirm_prompt(prompt: &str) -> Result<bool, String> {
    Confirm::new(prompt)
        .with_default(true)
        .prompt()
        .map_err(|e| format!("Prompt failed: {}", e))
}

fn multiselect_prompt(prompt: &str) -> Result<Vec<String>, String> {
    let options = vec![
        "entry", "start_now", "init", "nvidia", "pull",
        "root", "unshare_ipc", "unshare_netns",
        "unshare_process", "unshare_devsys", "unshare_all",
    ];

    let selected = MultiSelect::new(prompt, options)
        .prompt()
        .map_err(|e| format!("Prompt failed: {}", e))?;

    Ok(selected.into_iter().map(|s| s.to_string()).collect())
}
