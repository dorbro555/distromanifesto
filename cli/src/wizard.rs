use anyhow::{anyhow, Result};
use inquire::{Confirm, MultiSelect, Select, Text, validator::Validation};
use std::{env, fs, process::Command};

/// Entry point for the wizard. This version finds ~/.distromanifesto itself.
pub fn launch_wizard() -> Result<()> {
    println!("\n🧙 Welcome to the Distro Manifesto Wizard!\n");

    // Initial prompts (fields-based)
    let mut container_name = text_prompt("What should the container be named?")?;
    let mut base_image = text_prompt("Enter the base image (e.g. archlinux:latest):")?;
    let mut init_hooks_input = Text::new("Init hooks (comma separated, e.g. 'setup.sh,postinstall.sh'). Leave blank for none:")
        .with_validator(|input: &str| {
            if input.len() > 1000 {
                Ok(Validation::Invalid("Too long.".into()))
            } else {
                Ok(Validation::Valid)
            }
        })
        .prompt()?;
    let mut home_value = text_prompt("Which home directory should the container use? (e.g. /home/user)")?;
    let mut enabled_flags = multiselect_prompt("Select which flags you’d like to enable:")?;

    let mut raw_edited_content: Option<String> = None;

    loop {
        let manifest_content = {
            let init_hooks_formatted = if init_hooks_input.trim().is_empty() {
                String::new()
            } else {
                format!("init_hooks=\"{}\"\n", init_hooks_input.trim())
            };

            format!(
                "[{}]\nimage=\"{}\"\n{}home=\"{}\"\n{}\n",
                container_name, base_image, init_hooks_formatted, home_value, enabled_flags
            )
        };

        let manifest_content = if let Some(ref raw) = raw_edited_content {
            raw.clone()
        } else {
            manifest_content
        };

        println!("\n──────────────────────────────");
        println!("Manifest preview:\n\n{}", manifest_content);
        println!("──────────────────────────────\n");

        match crate::verify::verify_manifest(&manifest_content) {
            Ok(()) => {
                println!("✅ Manifest verification passed.");
                if Confirm::new("Save this manifest?")
                    .with_default(true)
                    .prompt()? {
                    save_manifest(&container_name, &manifest_content)?;
                    println!("✅ Saved.");
                } else {
                    println!("Aborted: manifest not saved.");
                }
                break;
            }
            Err(err) => {
                eprintln!("⚠️ Manifest verification failed: {}\n", err);

                let choices = vec![
                    "Edit fields (re-prompt individual fields)",
                    "Edit raw (open $EDITOR)",
                    "Re-run verification",
                    "Save anyway",
                    "Cancel / Abort",
                ];

                let choice = Select::new("What would you like to do?", choices).prompt()?;

                match choice.as_ref() {
                    "Edit fields (re-prompt individual fields)" => {
                        container_name = text_prompt_with_default("Container name:", &container_name)?;
                        base_image = text_prompt_with_default("Base image (e.g. archlinux:latest):", &base_image)?;
                        init_hooks_input = Text::new("Init hooks (comma separated), leave blank for none:")
                            .with_placeholder(&init_hooks_input)
                            .prompt()?;
                        home_value = text_prompt_with_default("Home directory (e.g. /home/user):", &home_value)?;
                        enabled_flags = multiselect_prompt_with_defaults("Select flags to enable:", &enabled_flags)?;
                        raw_edited_content = None;
                        continue;
                    }
                    "Edit raw (open $EDITOR)" => {
                        let edited = open_in_editor(&manifest_content)?;
                        raw_edited_content = Some(edited);
                        continue;
                    }
                    "Re-run verification" => {
                        println!("🔁 Re-verifying manifest...\n");
                        continue;
                    }
                    "Save anyway" => {
                        save_manifest(&container_name, &manifest_content)?;
                        println!("✅ Saved despite verification errors.");
                        break;
                    }
                    "Cancel / Abort" => {
                        println!("Aborted: manifest not saved.");
                        break;
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    Ok(())
}

fn save_manifest(name: &str, content: &str) -> Result<()> {
    let base = crate::setup::ensure_hidden_dir()
        .map_err(|e| anyhow!("Failed to ensure config dir: {}", e))?;

    let manifest_dir = base.join("manifests");
    fs::create_dir_all(&manifest_dir)
        .map_err(|e| anyhow!("Failed to create manifests dir: {}", e))?;

    let filename = format!("{}.ini", name.replace(' ', "_"));
    let path = manifest_dir.join(filename);

    fs::write(&path, content)
        .map_err(|e| anyhow!("Failed to write manifest file {}: {}", path.display(), e))?;

    Ok(())
}

fn open_in_editor(initial: &str) -> Result<String> {
    let mut tmp = env::temp_dir();
    let fname = format!("distromanifesto_edit_{}.ini", std::process::id());
    tmp.push(fname);

    fs::write(&tmp, initial)
        .map_err(|e| anyhow!("Failed to create temporary file: {}", e))?;

    let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".into());

    let status = Command::new(&editor)
        .arg(&tmp)
        .status()
        .map_err(|e| anyhow!("Failed to spawn editor '{}': {}", editor, e))?;

    if !status.success() {
        return Err(anyhow!("Editor returned non-zero exit code"));
    }

    let edited = fs::read_to_string(&tmp)
        .map_err(|e| anyhow!("Failed to read edited file: {}", e))?;

    let _ = fs::remove_file(&tmp);

    Ok(edited)
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

    Ok(Text::new(prompt).with_validator(validator).prompt()?)
}

fn text_prompt_with_default(prompt: &str, default: &str) -> Result<String> {
    let validator = |input: &str| {
        if input.trim().is_empty() {
            Ok(Validation::Invalid("This field cannot be empty.".into()))
        } else if input.len() > 140 {
            Ok(Validation::Invalid("Max 140 characters.".into()))
        } else {
            Ok(Validation::Valid)
        }
    };

    Ok(Text::new(prompt).with_validator(validator).with_placeholder(default).prompt()?)
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

fn multiselect_prompt_with_defaults(prompt: &str, _defaults: &str) -> Result<String> {
    multiselect_prompt(prompt)
}
