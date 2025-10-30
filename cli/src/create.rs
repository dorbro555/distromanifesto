// In cli/src/create.rs

use anyhow::{anyhow, Result};
use dirs::home_dir;
use inquire::{
    ui::{RenderConfig, Styled},
    Text,
};
use std::{fs, path::PathBuf};
use crate::tui;


// Helper function to get the save path
fn get_save_path(section_name: &str) -> Result<PathBuf> {
    let mut file_path = home_dir()
        .ok_or_else(|| anyhow!("Could not find user home directory"))?;
    file_path.push(".distromanifesto");
    file_path.push("homes");
    
    fs::create_dir_all(&file_path)?;
    
    file_path.push(format!("{}.ini", section_name));
    Ok(file_path)
}

pub fn launch_creator() -> Result<()> {
    // 1. Set up the custom render config for inquire
    let render_config = RenderConfig::default()
        .with_prompt_prefix(Styled::new("🧙 "));

    // 2. Prompt for the section name
    let section_name = Text::new("Enter the name for the first section:")
        .with_default("default")
        .with_render_config(render_config)
        .prompt()?;

    // 3. Get the final file path
    let file_path = get_save_path(&section_name)?;
    
    // 4. Check if file exists
    if file_path.exists() {
        return Err(anyhow!(
            "File already exists at: {}. Use 'modify' to edit.",
            file_path.display()
        ));
    }

    // 5. Create the initial "empty" TUI items
    let initial_items = vec![tui::DisplayItem::Section(section_name.clone())];

    // 6. Launch the generic TUI in Create mode
    crate::tui::run_tui(
        &file_path,
        initial_items,
        section_name,
        crate::tui::TuiMode::Create,
    )
}