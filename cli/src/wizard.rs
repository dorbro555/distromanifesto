use anyhow::Result;
use inquire::{Text, Confirm, MultiSelect, validator::Validation};
use std::fs;
use std::path::PathBuf;

/// Entry point for the wizard
pub fn launch_wizard(output_dir: PathBuf) -> Result<()> {
    println!("\n🧙 Welcome to the Distro Manifesto Wizard!\n");

    let container_name = text_prompt("What should the container be named?")?;
    let base_image = text_prompt("Enter the base image (e.g. archlinux:latest):")?;
    let init_cmd = text_prompt("Enter an initialization command:")?;
    let home_value = text_prompt("Which home directory should the container use?")?;

    let enabled_flags = multiselect_prompt("Select which flags you’d like to enable:")?;

    let assemble_content = format!(
        "[{}]\nimage=\"{}\"\ninit=\"{}\"\nhome=\"{}\"\n{}\n",
        container_name, base_image, init_cmd, home_value, enabled_flags
    );

    // println!("\n──────────────────────────────");
    // println!("Preview manifest:\n{}\n", manifest);
    // println!("──────────────────────────────");

    // if confirm_prompt("Save this manifest?")? {
    //     fs::write(output_path, manifest)
    //         .map_err(|e| format!("Failed to write file: {}", e))?;
    //     println!("✅ Manifest saved to {}", output_path);
    // } else {
    //     println!("Manifest not saved.");
    // }

    // Write to file
    let output_path = output_dir.join(format!("{}.ini", container_name));
    fs::write(&output_path, assemble_content)?;
    println!("\n✅ Manifest file created at: {}\n", output_path.display());

    Ok(())
}

fn text_prompt(prompt: &str) -> Result<String> {
    let validator = |input: &str| {
        if input.trim().is_empty() {
            Ok(Validation::Invalid("This field cannot be empty.".into()))
        } else if input.len() > 140 {
            Ok(Validation::Invalid("Max 140 characters.".into()))
        } else {
            Ok(Validation::Valid)
        }
    };

    let value = Text::new(prompt)
        .with_validator(validator)
        .prompt()?;

    Ok(value)
}

fn multiselect_prompt(prompt: &str) -> Result<String> {
    let options = vec![
        "entry",
        "start_now",
        "init",
        "nvidia",
        "pull",
        "root",
        "unshare_ipc",
        "unshare_netns",
        "unshare_process",
        "unshare_devsys",
        "unshare_all",
    ];

    let flags = MultiSelect::new(prompt, options).prompt()?;
    let mut formatted_flags = String::new();

    for flag in flags {
        formatted_flags.push_str(&format!("{}=true\n", flag));
    }

    Ok(formatted_flags)
}

#[allow(dead_code)]
fn confirm_prompt(prompt: &str) -> Result<bool> {
    let result = Confirm::new(prompt)
        .with_default(false)
        .with_help_message("Use arrow keys to choose, Enter to confirm.")
        .prompt()?;
    Ok(result)
}
