use anyhow::Result;
use inquire::{Text, Confirm, MultiSelect, validator::Validation};
use std::{fs, path::PathBuf};
use crate::setup::ensure_manifest_dir;
use crate::verify::verify_manifest;

/// Entry point for the wizard
pub fn launch_wizard() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧙 Welcome to the Distro Manifesto Wizard!\n");

    let name = Text::new("Manifest name:")
    .with_validator(|input: &str| {
        if input.trim().is_empty() {
            Ok(Validation::Invalid("Name cannot be empty.".into()))
        } else {
            Ok(Validation::Valid)
        }
    })
    .prompt()?;

    let image = Text::new("Base distro image (e.g., archlinux:latest):").prompt()?;
    let packages = Text::new("Additional packages (comma separated):").prompt()?;
    let description = Text::new("Short description:").prompt()?;

    // let enabled_flags = multiselect_prompt("Select which flags you’d like to enable:")?;

    let manifest_content = format!(
        "[manifest]\nname = {}\nimage = {}\npackages = {}\ndescription = {}\n",
        name, image, packages, description
    );

    println!("\n📜 Manifest Preview:\n{}\n", manifest_content);

    verify_manifest(&manifest_content)?;

    let confirm_save = Confirm::new("Save this manifest?")
        .with_default(true)
        .prompt()?;

    if confirm_save {
        let dir = ensure_manifest_dir()?;
        let mut path = PathBuf::from(dir);
        path.push(format!("{}.ini", name.replace(' ', "_")));
        fs::write(&path, &manifest_content)?;
        println!("✅ Saved successfully at {}", path.display());
    } else {
        println!("❌ Manifest discarded.");
    }

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
    // let output_path = output_dir.join(format!("{}.ini", container_name));
    // fs::write(&output_path, assemble_content)?;
    // println!("\n✅ Manifest file created at: {}\n", output_path.display());

    Ok(())
}

// fn text_prompt(prompt: &str) -> Result<String> {
//     let validator = |input: &str| {
//         if input.trim().is_empty() {
//             Ok(Validation::Invalid("This field cannot be empty.".into()))
//         } else if input.len() > 140 {
//             Ok(Validation::Invalid("Max 140 characters.".into()))
//         } else {
//             Ok(Validation::Valid)
//         }
//     };

//     let value = Text::new(prompt)
//         .with_validator(validator)
//         .prompt()?;

//     Ok(value)
// }

// fn multiselect_prompt(prompt: &str) -> Result<String> {
//     let options = vec![
//         "entry",
//         "start_now",
//         "init",
//         "nvidia",
//         "pull",
//         "root",
//         "unshare_ipc",
//         "unshare_netns",
//         "unshare_process",
//         "unshare_devsys",
//         "unshare_all",
//     ];

//     let flags = MultiSelect::new(prompt, options).prompt()?;
//     let mut formatted_flags = String::new();

//     for flag in flags {
//         formatted_flags.push_str(&format!("{}=true\n", flag));
//     }

//     Ok(formatted_flags)
// }

#[allow(dead_code)]
fn confirm_prompt(prompt: &str) -> Result<bool> {
    let result = Confirm::new(prompt)
        .with_default(false)
        .with_help_message("Use arrow keys to choose, Enter to confirm.")
        .prompt()?;
    Ok(result)
}
