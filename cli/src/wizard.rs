use anyhow::{anyhow, Result};
use inquire::{Confirm, MultiSelect, Select, Text};
use std::{env, fs, process::Command};

/// Entry point for the wizard. This version finds ~/.distromanifesto itself.
pub fn launch_wizard() -> Result<()> {
    println!("\n🧙 Welcome to the Distro Manifesto Wizard!\n");

    // Step 1: basic info
    let container_name = required_text_prompt("What should the container be named?")?;
    let base_image = required_text_prompt("Enter the base image (e.g. archlinux:latest):")?;

    // Step 2: additional packages (space separated)
    let additional_packages = optional_text_prompt(
        "Enter additional packages (space separated, leave blank for none):",
    )?;

    // Step 3: home directory handling
    let home_choice = Select::new(
        "Choose how to handle the container's home directory:",
        vec![
            "Create a new home inside ~/.distromanifesto/homes",
            "Use an existing home inside ~/.distromanifesto/homes",
            "Specify a full path manually",
        ],
    )
    .prompt()?;

    let home_value = match home_choice {
        "Create a new home inside ~/.distromanifesto/homes" => {
            let base = crate::setup::ensure_hidden_dir()?;
            let homes_dir = base.join("homes");
            fs::create_dir_all(&homes_dir)?;
            let home_path = homes_dir.join(&container_name);
            fs::create_dir_all(&home_path)?;
            format!("{}", home_path.display())
        }
        "Use an existing home inside ~/.distromanifesto/homes" => {
            let base = crate::setup::ensure_hidden_dir()?;
            let homes_dir = base.join("homes");
            fs::create_dir_all(&homes_dir)?;

            let entries = fs::read_dir(&homes_dir)?
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().into_string().unwrap_or_default())
                .collect::<Vec<_>>();

            if entries.is_empty() {
                println!("No existing homes found. Creating a new one instead.");
                let new_home = homes_dir.join(&container_name);
                fs::create_dir_all(&new_home)?;
                format!("{}", new_home.display())
            } else {
                let choice = Select::new("Select an existing home:", entries).prompt()?;
                format!("{}/{}", homes_dir.display(), choice)
            }
        }
        "Specify a full path manually" => required_text_prompt("Enter full path for home directory:")?,
        _ => unreachable!(),
    };

    // Step 4: hook commands (optional)
    let pre_init_hooks = optional_text_prompt(
        "Commands to run *before* package installation (e.g. 'echo pre'), leave blank for none:",
    )?;

    let init_hooks = optional_text_prompt(
        "Commands to run *after* package installation (e.g. 'cd ~ && git clone ...'), leave blank for none:",
    )?;

    // Step 5: flags
    let enabled_flags = multiselect_prompt("Select which flags you’d like to enable:")?;

    let manifest_content = build_manifest(
        &container_name,
        &base_image,
        &additional_packages,
        &home_value,
        &pre_init_hooks,
        &init_hooks,
        &enabled_flags,
    );

    println!("\n──────────────────────────────");
    println!("Manifest preview:\n\n{}", manifest_content);
    println!("──────────────────────────────\n");

    match crate::verify::verify_manifest(&manifest_content) {
        Ok(()) => {
            println!("✅ Manifest verification passed.");
            if Confirm::new("Save this manifest?").with_default(true).prompt()? {
                let saved_path = save_manifest(&container_name, &manifest_content)?;
                println!("✅ Saved at {}.", saved_path.display());

                if Confirm::new(&format!(
                    "Would you like to run 'distrobox assemble create --file {}'?",
                    saved_path.display()
                ))
                .with_default(false)
                .prompt()? {
                    if let Err(e) = Command::new("distrobox")
                        .args(["assemble", "create", "--file", &saved_path.display().to_string()])
                        .status()
                    {
                        eprintln!("⚠️ Failed to execute distrobox: {}", e);
                    }
                }
            } else {
                println!("Aborted: manifest not saved.");
            }
        }
        Err(err) => eprintln!("⚠️ Manifest verification failed: {}\n", err),
    }

    Ok(())
}

fn build_manifest(
    container_name: &str,
    base_image: &str,
    additional_packages: &str,
    home_value: &str,
    pre_init_hooks: &str,
    init_hooks: &str,
    enabled_flags: &str,
) -> String {
    let mut content = format!("[{}]\nimage=\"{}\"\n", container_name, base_image);

    if !additional_packages.trim().is_empty() {
        content.push_str(&format!("additional_packages=\"{}\"\n", additional_packages.trim()));
    }

    content.push_str(&format!("home={}\n", home_value));

    if !pre_init_hooks.trim().is_empty() {
        content.push_str(&format!("pre_init_hooks={}\n", pre_init_hooks.trim()));
    }

    if !init_hooks.trim().is_empty() {
        content.push_str(&format!("init_hooks={}\n", init_hooks.trim()));
    }

    content.push_str(enabled_flags);
    content
}

fn required_text_prompt(prompt: &str) -> Result<String> {
    Ok(Text::new(prompt).prompt()?)
}

fn optional_text_prompt(prompt: &str) -> Result<String> {
    let input = Text::new(prompt).prompt()?;
    Ok(input.trim().to_string())
}

fn save_manifest(name: &str, content: &str) -> Result<std::path::PathBuf> {
    let base = crate::setup::ensure_hidden_dir()
        .map_err(|e| anyhow!("Failed to ensure config dir: {}", e))?;

    let manifest_dir = base.join("manifests");
    fs::create_dir_all(&manifest_dir)?;

    let filename = format!("{}.ini", name.replace(' ', "_"));
    let path = manifest_dir.join(filename);

    fs::write(&path, content)?;

    Ok(path)
}

fn multiselect_prompt(prompt: &str) -> Result<String> {
    let options = vec![
        "entry", "start_now", "init", "nvidia", "pull", "root", "unshare_ipc",
        "unshare_netns", "unshare_process", "unshare_devsys", "unshare_all",
    ];

    let flags = MultiSelect::new(prompt, options).prompt()?;
    let mut formatted_flags = String::new();
    for flag in flags {
        formatted_flags.push_str(&format!("{}=true\n", flag));
    }
    Ok(formatted_flags)
}
