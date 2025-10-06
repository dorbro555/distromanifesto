use std::error::Error;

/// Very small manifest verifier:
/// - allows blank lines and comments (starting with #)
/// - accepts section headers [name]
/// - accepts key=value lines
/// - enforces at least one section header
pub fn verify_manifest(content: &str) -> Result<(), Box<dyn Error>> {
    let mut has_section = false;

    for (i, line) in content.lines().enumerate() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        } else if line.starts_with('[') && line.ends_with(']') {
            has_section = true;
            continue;
        } else if line.contains('=') {
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts[0].trim().is_empty() || parts[1].trim().is_empty() {
                return Err(format!("Invalid key-value pair on line {}", i + 1).into());
            }
            continue;
        } else {
            return Err(format!("Invalid line format on line {}: {}", i + 1, line).into());
        }
    }

    if !has_section {
        return Err("Manifest must contain at least one section header.".into());
    }

    Ok(())
}

