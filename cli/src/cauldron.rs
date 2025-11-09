// In cli/src/cauldron.rs

use anyhow::{Context, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::fs;
use std::io;
use std::path::PathBuf;
// --- Add these ---
use std::process::Command;
use std::time::{Duration, Instant};

use crate::setup;

// --- App State ---
struct App {
    manifests: Vec<String>,
    homes: Vec<String>,
    containers: Vec<String>, // <-- Add this
    last_refresh: Instant,  // <-- Add this
    should_quit: bool,
}

impl App {
    // Constructor that loads initial data
    fn new() -> Result<Self> {
        let manifests =
            load_directory_contents("manifests").context("Failed to load manifests")?;
        let homes = load_directory_contents("homes").context("Failed to load homes")?;
        let containers = load_distrobox_list().context("Failed to load distrobox list")?; // <-- Add this

        Ok(App {
            manifests,
            homes,
            containers, // <-- Add this
            last_refresh: Instant::now(), // <-- Add this
            should_quit: false,
        })
    }

    // --- NEW: Refresh data method ---
    fn refresh_data(&mut self) -> Result<()> {
        self.manifests =
            load_directory_contents("manifests").context("Failed to load manifests")?;
        self.homes = load_directory_contents("homes").context("Failed to load homes")?;
        self.containers =
            load_distrobox_list().context("Failed to load distrobox list")?;
        self.last_refresh = Instant::now();
        Ok(())
    }
}

/// Helper function to load file/dir names from a ~/.distromanifesto subdirectory
fn load_directory_contents(dir_name: &str) -> Result<Vec<String>> {
    let dir_path_str = format!("~/.distromanifesto/{}", dir_name);
    let path: PathBuf = setup::get_full_path_from_str(&dir_path_str)
        .with_context(|| format!("Failed to get full path for {}", dir_path_str))?;

    let mut entries = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if let Some(file_name) = entry.file_name().to_str() {
            entries.push(file_name.to_string());
        }
    }
    entries.sort(); // Keep the list consistent
    Ok(entries)
}

// --- NEW: Helper to run `distrobox list` and parse output ---
fn load_distrobox_list() -> Result<Vec<String>> {
    let output = Command::new("distrobox")
        .args(["list", "--no-color"])
        .output()
        .context("Failed to execute 'distrobox list'")?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "distrobox list failed: {}",
            error_msg
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut containers = Vec::new();

    // Iterate over lines, skipping the header
    for line in stdout.lines().skip(1) {
        // Split by '|' and take the second-to-last-column (Name)
        if let Some(name) = line.split('|').nth(1) {
            let trimmed_name = name.trim();
            if !trimmed_name.is_empty() {
                containers.push(trimmed_name.to_string());
            }
        }
    }

    Ok(containers)
}

// --- Main TUI Function ---
pub fn launch_tui() -> Result<()> {
    // ... (setup code is the same)
    // ...
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new()?; // Load data on startup
    let res = run_app(&mut terminal, app);

    // ... (restore terminal code is the same)
    // ...
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("{err:?}");
    }

    Ok(())
}

// --- Main App Loop ---
fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    // --- Define refresh rate ---
    let refresh_rate = Duration::from_secs(5);

    loop {
        terminal.draw(|f| ui(f, &app))?;

        // --- Event handling with a timeout ---
        let timeout = refresh_rate
            .checked_sub(app.last_refresh.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        app.should_quit = true;
                    }
                    KeyCode::Char('r') => {
                        // Manual refresh
                        app.refresh_data()?;
                    }
                    _ => {}
                }
            }
        }

        // --- Auto-refresh logic ---
        if app.last_refresh.elapsed() >= refresh_rate {
            app.refresh_data()?;
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

// --- UI Drawing ---
fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    // ... (layout code is the same)
    // ...
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ]
            .as_ref(),
        )
        .split(f.size());


    // --- Panel 1: Containers ---
    let container_items: Vec<ListItem> = app
        .containers
        .iter()
        .map(|c| ListItem::new(c.as_str()))
        .collect();
    let container_list =
        List::new(container_items).block(Block::default().title("Containers").borders(Borders::ALL));
    f.render_widget(container_list, chunks[0]); // <-- Render this

    // --- Panel 2: Manifests ---
    let manifest_items: Vec<ListItem> = app
        .manifests
        .iter()
        .map(|m| ListItem::new(m.as_str()))
        .collect();
    let manifest_list =
        List::new(manifest_items).block(Block::default().title("Manifests").borders(Borders::ALL));
    f.render_widget(manifest_list, chunks[1]);

    // --- Panel 3: Homes ---
    let home_items: Vec<ListItem> = app
        .homes
        .iter()
        .map(|h| ListItem::new(h.as_str()))
        .collect();
    let home_list =
        List::new(home_items).block(Block::default().title("Managed Homes").borders(Borders::ALL));
    f.render_widget(home_list, chunks[2]);

    // We'll also need a footer for keybindings
}