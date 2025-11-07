use std::fs;
use std::path::PathBuf;
use anyhow;
use dirs::home_dir;

/// Ensures that ~/.distromanifesto and its subdirectories exist.
/// Returns the path to the base directory.
pub fn ensure_hidden_dir() -> std::io::Result<PathBuf> {
    let mut dir = dirs::home_dir().expect("Could not find home directory");
    dir.push(".distromanifesto");

    // Create ~/.distromanifesto if missing
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
        println!("Created {:?}", dir);
    }

    // Create subdirectories
    for sub in &["manifests", "templates"] {
        let sub_path = dir.join(sub);
        if !sub_path.exists() {
            fs::create_dir_all(&sub_path)?;
            println!("Created {:?}", sub_path);
        }
    }

    Ok(dir)
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