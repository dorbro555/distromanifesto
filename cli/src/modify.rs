use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::Backend,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    terminal::{Frame, Terminal},
    widgets::{Block, Borders, List, ListItem, ListState}, // 1. Import ListState
    prelude::*,
};
use ini::Ini;
use std::{io, path::Path, time::Duration};

/// A struct to hold the application's state.
struct App<'a> {
    conf: Ini,
    items: Vec<ListItem<'a>>,
    state: ListState,
}

impl<'a> App<'a> {
    /// Creates a new App instance from a loaded Ini configuration.
    fn new(conf: Ini) -> App<'a> {
        let mut state = ListState::default();
        if !conf.is_empty() {
            state.select(Some(0)); // Select the first item by default
        }
        
        let mut app = App {
            conf,
            items: Vec::new(), // Will be populated by refresh_items
            state,
        };
        app.refresh_items(); // Populate the items list
        app
    }

    /// Re-generates the list of items from the `conf`.
    fn refresh_items(&mut self) {
        let mut items = Vec::new();
        for (sec, prop) in self.conf.iter() {
            let section_name = sec.unwrap_or("Global");
            items.push(
                ListItem::new(format!("[{section_name}]"))
                    .style(Style::default().fg(Color::Green).bold())
            );
            for (key, value) in prop.iter() {
                items.push(ListItem::new(format!("  {key} = {value}")));
            }
        }
        self.items = items;
    }

    /// Moves the selection to the next item.
    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Moves the selection to the previous item.
    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}


/// Launches the TUI editor for a manifest file.
pub fn launch_editor(file_path: &Path) -> Result<()> {
    // Load the INI file
    let conf = Ini::load_from_file(file_path)?;

    // 2. Create the new App state
    let mut app = App::new(conf);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 3. Pass the mutable app state to the run loop
    let res = run_app(&mut terminal, &mut app);

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
// 4. Update run_app to take our App struct
fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?; // Pass the app to the ui function

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                // 5. Handle navigation keybinds
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.previous(),
                    // TODO: Add KeyCode::Enter to trigger editing
                    // TODO: Add KeyCode::Char('s') to save
                    _ => {}
                }
            }
        }
    }
}

/// Renders the user interface.
// 6. Update ui to take App and render the state
fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
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
        .title(" (q) Quit | (↑/↓) Navigate | (Enter) Edit | (s) Save ");
    f.render_widget(footer_block, chunks[2]);

    // 7. Create the list from the app's items
    let list = List::new(app.items.clone()) // Clone items to pass ownership
        .block(Block::default().title("Manifest").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    // 8. Render the list as a *Stateful* widget, passing our state
    f.render_stateful_widget(list, chunks[1], &mut app.state);
}