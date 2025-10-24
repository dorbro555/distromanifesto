use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::Backend,
    backend::CrosstermBackend, // <-- FIX 1: 'backend' is singular
    layout::{Constraint, Direction, Layout}, // <-- FIX 2: Removed unused 'Rect'
    terminal::{Frame, Terminal},
    widgets::{Block, Borders, List, ListItem},
    prelude::*,
};
use ini::Ini; // This import is correct
use std::{io, path::Path, time::Duration};

/// Launches the TUI editor for a manifest file.
pub fn launch_editor(file_path: &Path) -> Result<()> {
    // Load the INI file using rust-ini
    let mut conf = Ini::load_from_file(file_path)?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let res = run_app(&mut terminal, &mut conf);

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

/// Main application loop.
fn run_app<B: Backend>(terminal: &mut Terminal<B>, conf: &mut Ini) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, conf))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if KeyCode::Char('q') == key.code {
                    // TODO: Add 's' to save
                    // if key.code == KeyCode::Char('s') {
                    //     conf.write_to_file("path/to/save.ini").unwrap();
                    // }
                    return Ok(());
                }
            }
        }
    }
}

/// Renders the user interface.
fn ui<B: Backend>(f: &mut Frame<B>, conf: &Ini) {
    let size = f.size();
    
    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Footer
        ].as_ref())
        .split(size);

    let title_block = Block::default()
        .borders(Borders::ALL)
        .title(" Distromanifesto Editor ");
    f.render_widget(title_block, chunks[0]);

    let footer_block = Block::default()
        .borders(Borders::ALL)
        .title(" (q) Quit | (s) Save | (Enter) Edit ");
    f.render_widget(footer_block, chunks[2]);

    // Convert INI to list items
    let mut items = Vec::new();
    for (sec, prop) in conf.iter() {
        // --- FIX 3: Handle the Option<&str> ---
        let section_name = sec.unwrap_or("Global"); // Use "Global" if section is None
        items.push(
            ListItem::new(format!("[{section_name}]")) // Use the new variable
                .style(Style::default().fg(Color::Green).bold())
        );
        // --- End of Fix ---

        for (key, value) in prop.iter() {
            items.push(ListItem::new(format!("  {key} = {value}")));
        }
    }

    let list = List::new(items)
        .block(Block::default().title("Manifest").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    // Render the list in the content area
    f.render_widget(list, chunks[1]);
}