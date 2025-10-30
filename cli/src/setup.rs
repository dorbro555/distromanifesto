use std::fs;
use std::path::PathBuf;

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
