use std::collections::HashMap;
use std::error::Error;

/// Validate a distrobox-style INI manifest string.
/// Returns Ok(()) if valid, otherwise Err with a descriptive message.
pub fn verify_manifest(content: &str) -> Result<(), Box<dyn Error>> {
    #[derive(Clone, Copy)]
    enum ValueKind {
        Bool,
        Str,
        StrList,
    }

    // Allowed keys and their expected types
    let mut allowed: HashMap<&str, ValueKind> = HashMap::new();
    // string_list
    for k in &[
        "additional_flags",
        "additional_packages",
        "init_hooks",
        "pre_init_hooks",
        "volume",
        "exported_apps",
        "exported_bins",
    ] {
        allowed.insert(k, ValueKind::StrList);
    }
    // string
    for k in &["home", "image", "clone", "exported_bins_path"] {
        allowed.insert(k, ValueKind::Str);
    }
    // bools
    for k in &[
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
    ] {
        allowed.insert(k, ValueKind::Bool);
    }

    let mut current_section: Option<String> = None;
    // For per-section validation we gather keys present
    let mut section_keys: HashMap<String, Vec<String>> = HashMap::new();
    let mut saw_any_section = false;

    for (lineno, raw) in content.lines().enumerate() {
        let line_no = lineno + 1;
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Section header
        if line.starts_with('[') && line.ends_with(']') {
            let name = &line[1..line.len() - 1].trim();
            if name.is_empty() {
                return Err(format!("Empty section name at line {}", line_no).into());
            }
            // validate characters
            if !name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
            {
                return Err(format!(
                    "Invalid characters in section name '{}' at line {}",
                    name, line_no
                )
                .into());
            }
            saw_any_section = true;
            current_section = Some(name.to_string());
            section_keys.insert(name.to_string(), Vec::new());
            continue;
        }

        // Key=value lines
        if let Some(eq_pos) = line.find('=') {
            // --- NEW CHECK: Get raw parts *before* trimming ---
            let key_raw = &line[..eq_pos];
            let value_raw = &line[eq_pos + 1..];

            // Check for spaces around the '='
            if key_raw.ends_with(' ') || value_raw.starts_with(' ') {
                return Err(format!(
                    "Invalid format on line {}: Key-value pairs must not have spaces around the '='. Found: '{}'",
                    line_no, line
                ).into());
            }

            // Now trim and proceed
            let key = key_raw.trim();
            let mut value = value_raw.trim();
            // --- END NEW CHECK ---

            if key.is_empty() {
                return Err(format!("Empty key on line {}", line_no).into());
            }

            // Remove wrapping quotes if present
            if (value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\''))
            {
                if value.len() >= 2 {
                    value = &value[1..value.len() - 1];
                } else {
                    value = "";
                }
            }

            // Is this key allowed?
            let kind = allowed
                .get(key)
                .ok_or_else(|| format!("Unknown key '{}' at line {}", key, line_no))?;

            match kind {
                ValueKind::Bool => {
                    let v = value.to_ascii_lowercase();
                    if !(v == "true" || v == "false" || v == "1" || v == "0") {
                        return Err(format!(
                            "Invalid boolean for '{}' on line {}: '{}'",
                            key, line_no, value
                        )
                        .into());
                    }
                }
                ValueKind::Str => {
                    if value.trim().is_empty() {
                        return Err(format!(
                            "Value for '{}' cannot be empty (line {})",
                            key, line_no
                        )
                        .into());
                    }
                }
                ValueKind::StrList => {
                    // Accept a comma-separated or whitespace-separated list.
                    // Empty list is allowed (interpreted as none).
                    let trimmed = value.trim();
                    if !trimmed.is_empty() {
                        // split by comma first; if only one token, split by whitespace
                        let tokens: Vec<&str> = if trimmed.contains(',') {
                            trimmed
                                .split(',')
                                .map(|s| s.trim())
                                .filter(|s| !s.is_empty())
                                .collect()
                        } else {
                            trimmed.split_whitespace().collect()
                        };
                        if tokens.is_empty() {
                            return Err(format!(
                                "Invalid list for '{}' on line {}: '{}'",
                                key, line_no, value
                            )
                            .into());
                        }
                        // optional: further validate tokens (no spaces inside tokens etc.)
                        for t in tokens {
                            if t.is_empty() {
                                return Err(format!(
                                    "Empty token in list '{}' on line {}",
                                    key, line_no
                                )
                                .into());
                            }
                        }
                    }
                }
            }

            // register that this key was present in this section
            if let Some(sec) = current_section.as_ref() {
                if let Some(vec) = section_keys.get_mut(sec) {
                    vec.push(key.to_string());
                }
            } else {
                return Err(format!(
                    "Key/value outside any section on line {}: '{}'",
                    line_no, line
                )
                .into());
            }

            continue;
        }

        // If we reach here, line is invalid
        return Err(format!("Unrecognized line {}: '{}'", line_no, line).into());
    }

    if !saw_any_section {
        return Err("Manifest must contain at least one section header like [mycontainer]".into());
    }

    // Per-section rule: require at least image or clone
    for (sec, keys) in section_keys {
        let has_image = keys.iter().any(|k| k == "image");
        let has_clone = keys.iter().any(|k| k == "clone");
        if !has_image && !has_clone {
            return Err(
                format!("Section [{}] must contain at least 'image' or 'clone'", sec).into(),
            );
        }
    }

    Ok(())
}
