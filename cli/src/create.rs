use anyhow::{Result};
use std::{path::PathBuf};

pub fn launch_creator() -> Result<()> {
    // We must pass a *dummy* path and section name to start.
    // The TUI wizard will overwrite them.
    let dummy_path = PathBuf::from("/tmp/distromanifesto.tmp");
    let initial_items = Vec::new(); // Start with no items
    let initial_section = String::new();

    // Launch the generic TUI in Create mode, which will
    // trigger the new wizard flow.
    crate::tui::run_tui(
        &dummy_path,
        initial_items,
        initial_section,
        crate::tui::TuiMode::Create,
    )
}