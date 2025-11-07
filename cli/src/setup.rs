use std::fs;
use std::path::PathBuf;
use anyhow;
use dirs::home_dir;

/// Ensures that ~/.distromanifesto and its subdirectories exist.
/// Returns the path to the base directory.

pub(crate) fn ensure_hidden_dir() -> Result<(), anyhow::Error> {
    let mut homes_dir = home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    homes_dir.push(".distromanifesto");
    
    // Create the base .distromanifesto directory
    fs::create_dir_all(&homes_dir)?;

    // Create the homes/ subdirectory
    let mut homes_sub_dir = homes_dir.clone();
    homes_sub_dir.push("homes");
    fs::create_dir_all(&homes_sub_dir)?;

    // Create the manifests/ subdirectory
    let mut manifests_sub_dir = homes_dir.clone();
    manifests_sub_dir.push("manifests");
    fs::create_dir_all(&manifests_sub_dir)?;

    Ok(())
}

/// Parses a tilde-prefixed path and creates the directory.
pub(crate) fn create_managed_home(path_str: &str) -> Result<(), anyhow::Error> {
    if !path_str.starts_with("~/.distromanifesto/homes/") {
        return Ok(());
    }

    let full_path = get_full_path_from_str(path_str)?;
    fs::create_dir_all(&full_path)?;
    Ok(())
}

/// Parses a tilde-prefixed string into a full PathBuf
pub(crate) fn get_full_path_from_str(path_str: &str) -> Result<PathBuf, anyhow::Error> {
    let path_no_tilde = path_str.strip_prefix("~/").unwrap_or(path_str);
    let mut full_path = home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    full_path.push(path_no_tilde);
    Ok(full_path)
}