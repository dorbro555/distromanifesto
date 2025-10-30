// In cli/src/modify.rs

use anyhow::Result;
use distro_ini::Ini;
use std::path::Path;
use crate::tui;

// --- This is the new home for parse_conf_to_items ---
fn parse_conf_to_items(conf: Ini) -> (Vec<tui::DisplayItem>, String) {
    let mut items = Vec::new();
    let mut first_section = "Global".to_string();
    let mut found_first = false;

    for (sec, prop) in conf.iter() {
        let section_name = sec.unwrap_or("Global").to_string();
        if !found_first {
            first_section = section_name.clone();
            found_first = true;
        }
        items.push(tui::DisplayItem::Section(section_name.clone()));

        for (key, value) in prop.iter() {
            items.push(tui::DisplayItem::Property(
                section_name.clone(),
                key.to_string(),
                value.to_string(),
            ));
        }
    }
    (items, first_section)
}

pub fn launch_editor(file_path: &Path) -> Result<()> {
    // 1. Load the file
    let conf = Ini::load_from_file(file_path)?;
    
    // 2. Parse it into the TUI's items
    let (initial_items, initial_section) = parse_conf_to_items(conf);

    // 3. Launch the generic TUI in Modify mode
    crate::tui::run_tui(
        file_path,
        initial_items,
        initial_section,
        crate::tui::TuiMode::Modify,
    )
}