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
use std::time::Duration;

use crate::setup;

// --- App State ---
struct App {
    manifests: Vec<String>,
    homes: Vec<String>,
    should_quit: bool,
}

impl App {
    // Constructor that loads initial data
    fn new() -> Result<Self> {
        let manifests =
            load_directory_contents("manifests").context("Failed to load manifests")?;
        let homes = load_directory_contents("homes").context("Failed to load homes")?;
        Ok(App {
            manifests,
            homes,
            should_quit: false,
        })
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

// --- Main TUI Function ---
pub fn launch_tui() -> Result<()> {
    // Ensure directories exist before trying to read them
    setup::ensure_hidden_dir().context("Failed to ensure .distromanifesto directories")?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new()?; // Load data on startup
    let res = run_app(&mut terminal, app);

    // Restore terminal
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
    loop {
        terminal.draw(|f| ui(f, &app))?;

        // Event handling
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        app.should_quit = true;
                    }
                    _ => {}
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }

        // --- Update app state here (e.g., fetch lists) ---
        // We'll add a ticker here later to refresh the data
    }
}

// --- UI Drawing ---
fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    // For now, just a simple 3-panel layout
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
    let block1 = Block::default().title("Containers").borders(Borders::ALL);
    let p1 = Paragraph::new("List of active containers... (from `distrobox list`)");
    f.render_widget(p1.block(block1), chunks[0]);

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