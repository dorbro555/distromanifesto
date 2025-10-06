use anyhow::Result;
use inquire::{Text, Confirm, MultiSelect, validator::Validation};
use std::{fs, path::PathBuf};

/// Entry point for the wizard (minimal + verification + preview + save)
pub fn launch_wizard() -> Result<()> {
    println!("\n🧙 Welcome to the Distro Manifesto Wizard!\n");

    // Prompts (same as before)
    let container_name = text_prompt("What should the container be named?")?;
    let base_image = text_prompt("Enter the base image (e.g. archlinux:latest):")?;
    let init_hooks = text_prompt("Enter an initialization command:")?;
    let home_value = text_prompt("Which home directory should the container use?")?;
    let enabled_flags = multiselect_prompt("Select which flags you’d like to enable:")?;

    // Build manifest content
    let assemble_content = format!(
        "[{}]\nimage=\"{}\"\ninit_hooks=\"{}\"\nhome=\"{}\"\n{}\n",
        container_name, base_image, init_hooks, home_value, enabled_flags
    );

    // --- Preview ---
    println!("\n──────────────────────────────");
    println!("Manifest preview:\n\n{}", assemble_content);
    println!("──────────────────────────────\n");

    // --- Verification ---
    match crate::verify::verify_manifest(&assemble_content) {
        Ok(()) => {
            println!("✅ Manifest verification passed.");
            // Ask to save
            if Confirm::new("Save this manifest?")
                .with_default(true)
                .prompt()?
            {
                save_manifest(&container_name, &assemble_content)?;
                println!("✅ Saved.");
            } else {
                println!("Aborted: manifest not saved.");
            }
        }
        Err(err) => {
            // Verification failed: show error and ask whether to save anyway
            eprintln!("⚠️ Manifest verification failed: {}\n", err);
            let save_anyway = Confirm::new("Verification failed — save anyway?")
                .with_default(false)
                .prompt()?;

            if save_anyway {
                save_manifest(&container_name, &assemble_content)?;
                println!("✅ Saved despite verification warnings.");
            } else {
                println!("Aborted: manifest not saved due to verification failure.");
            }
        }
    }

    Ok(())
}

/// Helper: save to ~/.distromanifesto/manifests/<container>.ini
fn save_manifest(name: &str, content: &str) -> Result<()> {
    // ensure base hidden dir exists (this also creates manifests/ and templates/)
    let base = crate::setup::ensure_hidden_dir()
        .map_err(|e| anyhow::anyhow!("Failed to ensure config dir: {}", e))?;

    let manifest_dir = base.join("manifests");
    // attempt to create manifests dir as a precaution
    std::fs::create_dir_all(&manifest_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create manifests dir: {}", e))?;

    let filename = format!("{}.ini", name.replace(' ', "_"));
    let path = manifest_dir.join(filename);

    fs::write(&path, content)
        .map_err(|e| anyhow::anyhow!("Failed to write manifest file {}: {}", path.display(), e))?;

    Ok(())
}

// ---------- small helpers re-used from previous wizard implementation ----------

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
