// In cli/src/cauldron.rs

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Layout, Constraint, Direction},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io::{self, Stdout};
use std::time::Duration;

// --- App State ---
struct App {
    // We'll add lists for containers, manifests, etc. here
    should_quit: bool,
}

// --- Main TUI Function ---
pub fn launch_tui() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App { should_quit: false };
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
    let block2 = Block::default().title("Manifests").borders(Borders::ALL);
     let p2 = Paragraph::new("List of manifests... (from `~/.distromanifesto/homes/`)");
    f.render_widget(p2.block(block2), chunks[1]);
    
    // --- Panel 3: Homes ---
    let block3 = Block::default().title("Managed Homes").borders(Borders::ALL);
     let p3 = Paragraph::new("List of home dirs... (from `~/.distromanifesto/homes/`)");
    f.render_widget(p3.block(block3), chunks[2]);

    // We'll also need a footer for keybindings
}