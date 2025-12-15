// cli/src/assemble.rs

use anyhow::{Context, Result};
use distro_ini::Ini;
use std::path::Path;
use std::process::{Command, Stdio};

/// Reads a manifest file and executes distrobox create for each defined container.
pub fn create_containers_from_file(path: &Path) -> Result<()> {
    // 1. Parse the INI file
    let conf = Ini::load_from_file(path).context("Failed to parse manifest")?;
    
    // Log the filename for context
    let filename = path.file_name().unwrap_or_default().to_string_lossy();
    println!("Processing manifest: {}", filename);

    // 2. Iterate over sections (containers)
    for (section, properties) in conf.iter() {
        // 'section' is Option<&str> -> unwrap_or("Global") -> to_string()
        let container_name = section.unwrap_or("Global").to_string();
        
        // Skip the "Global" bucket or any empty sections
        if container_name == "Global" { continue; }

        println!("Creating container: [{}]...", container_name);

        let mut cmd = Command::new("distrobox");
        cmd.arg("create")
           .arg("--name")
           .arg(&container_name);
        
        // Allow the command to be interactive (inherit stdio)
        cmd.stdout(Stdio::inherit())
           .stderr(Stdio::inherit())
           .stdin(Stdio::inherit());

        // 3. Map keys to flags
        for (key, value) in properties.iter() {
            match key { 
                "image" => { cmd.arg("--image").arg(value); },
                "home" => {
                    if value != "host" && value != "none" {
                        cmd.arg("--home").arg(value);
                    }
                },
                "init" => { if value == "true" { cmd.arg("--init"); } },
                "nvidia" => { if value == "true" { cmd.arg("--nvidia"); } },
                "pull" => { if value == "true" { cmd.arg("--pull"); } },
                "root" => { if value == "true" { cmd.arg("--root"); } },
                "unshare_ipc" => { if value == "true" { cmd.arg("--unshare-ipc"); } },
                "unshare_netns" => { if value == "true" { cmd.arg("--unshare-netns"); } },
                "unshare_process" => { if value == "true" { cmd.arg("--unshare-process"); } },
                "unshare_devsys" => { if value == "true" { cmd.arg("--unshare-devsys"); } },
                "unshare_all" => { if value == "true" { cmd.arg("--unshare-all"); } },
                "additional_flags" => { cmd.arg("--additional-flags").arg(value); },
                "additional_packages" => { cmd.arg("--additional-packages").arg(value); },
                "init_hooks" => { cmd.arg("--init-hooks").arg(value); },
                "pre_init_hooks" => { cmd.arg("--pre-init-hooks").arg(value); },
                "volume" => { cmd.arg("--volume").arg(value); },
                _ => {} // Ignore unknown keys
            }
        }

        // 4. Run the command
        let status = cmd.spawn()?.wait()?;

        if !status.success() {
            println!("\n❌ Error creating container '{}'", container_name);
        } else {
            println!("\n✅ Successfully created container '{}'", container_name);
        }
    }

    Ok(())
}