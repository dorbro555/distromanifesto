use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ini::Ini;
use ratatui::{
    backend::Backend,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    terminal::{Frame, Terminal},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    prelude::*,
};
use std::{io, path::{Path, PathBuf}, time::Duration};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

/// Enum to track the application's current mode.
enum AppMode {
    Navigating,
    Editing,
    ConfirmDelete, // --- NEW: Add a state for delete confirmation ---
}

/// Enum to represent a displayable line in our list.
#[derive(Clone)]
enum DisplayItem {
    Section(String),
    Property(String, String, String), // section, key, value
}

/// A struct to hold the application's state.
struct App<'a> {
    file_path: PathBuf,
    conf: Ini,
    items: Vec<DisplayItem>,
    list_items: Vec<ListItem<'a>>,
    state: ListState,
    mode: AppMode,
    input: Input,
    editing_key: Option<(String, String)>, // (section, key) of the item being edited
}

impl<'a> App<'a> {
    /// Creates a new App instance from a loaded Ini configuration.
    fn new(conf: Ini, file_path: &Path) -> App<'a> {
        let mut state = ListState::default();
        if !conf.is_empty() {
            state.select(Some(0));
        }

        let mut app = App {
            file_path: file_path.to_path_buf(),
            conf,
            items: Vec::new(),
            list_items: Vec::new(),
            state,
            mode: AppMode::Navigating,
            input: Input::default(),
            editing_key: None,
        };
        app.refresh_items();
        app
    }

    /// Re-generates the list of items from the `conf`.
    fn refresh_items(&mut self) {
        let mut items = Vec::new();
        let mut list_items = Vec::new();

        for (sec, prop) in self.conf.iter() {
            let section_name = sec.unwrap_or("Global").to_string();
            items.push(DisplayItem::Section(section_name.clone()));
            list_items.push(
                ListItem::new(format!("[{section_name}]"))
                    .style(Style::default().fg(Color::Green).bold()),
            );

            for (key, value) in prop.iter() {
                items.push(DisplayItem::Property(
                    section_name.clone(),
                    key.to_string(),
                    value.to_string(),
                ));
                list_items.push(ListItem::new(format!("  {key} = {value}")));
            }
        }
        self.items = items;
        self.list_items = list_items;
    }

    /// Moves the selection to the next item.
    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.list_items.len() - 1 {
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
                    self.list_items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Enters editing mode for the currently selected item.
    fn start_editing(&mut self) {
        if let Some(index) = self.state.selected() {
            match &self.items[index] {
                DisplayItem::Property(section, key, value) => {
                    self.mode = AppMode::Editing;
                    self.editing_key = Some((section.clone(), key.clone()));
                    self.input = Input::new(value.clone());
                }
                DisplayItem::Section(_) => {} // Can't edit sections
            }
        }
    }

    /// Cancels the current edit.
    fn cancel_editing(&mut self) {
        self.mode = AppMode::Navigating;
        self.editing_key = None;
        self.input = Input::default();
    }

    /// Submits the current edit, saving it to the `conf`.
    fn submit_editing(&mut self) {
        if let (Some((section, key)), Some(index)) = 
            (self.editing_key.clone(), self.state.selected()) {
            
            let new_value = self.input.value().to_string();
            
            let section_name = if section == "Global" { None } else { Some(section.as_str()) };
            self.conf
                .with_section(section_name)
                .set(&key, &new_value);
            
            self.items[index] = DisplayItem::Property(
                section.clone(),
                key.clone(),
                new_value.clone()
            );
            self.list_items[index] = ListItem::new(format!("  {key} = {new_value}"));

            self.cancel_editing();
            self.state.select(Some(index));
        }
    }

    /// Saves the current in-memory `conf` back to the file.
    fn save_to_file(&mut self) -> Result<()> {
        let mut new_conf = Ini::new();
        for item in &self.items {
            if let DisplayItem::Property(section, key, value) = item {
                let section_name = if section == "Global" { None } else { Some(section.as_str()) };
                new_conf.with_section(section_name).set(key, value);
            }
        }
        new_conf.write_to_file(&self.file_path)?;
        self.conf = new_conf;
        Ok(())
    }

    // --- NEW: Function to delete the selected item ---
    /// Deletes the currently selected item.
    fn delete_selected(&mut self) {
        if let Some(index) = self.state.selected() {
            match &self.items[index] {
                DisplayItem::Property(section, key, _) => {
                    // 1. Remove from conf
                    let section_name = if section == "Global" { None } else { Some(section.as_str()) };
                    self.conf.delete_from(section_name, key);

                    // 2. Remove from display lists
                    self.items.remove(index);
                    self.list_items.remove(index);

                    // 3. Fix selection (select previous item, or 0 if it was the first)
                    let new_index = if index > 0 { index - 1 } else { 0 };
                    if self.list_items.is_empty() {
                        self.state.select(None);
                    } else {
                        self.state.select(Some(new_index));
                    }
                }
                DisplayItem::Section(_) => {
                    // TODO: Implement section deletion?
                    // For now, we only delete properties.
                }
            }
        }
        self.mode = AppMode::Navigating; // Go back to navigating
    }

    // --- NEW: Function to enter the delete confirmation mode ---
    /// Enters delete confirmation mode if a property is selected.
    fn start_confirm_delete(&mut self) {
        if let Some(index) = self.state.selected() {
            if let DisplayItem::Property(..) = &self.items[index] {
                self.mode = AppMode::ConfirmDelete;
            }
        }
    }

    // --- NEW: Function to cancel the delete confirmation ---
    /// Exits delete confirmation mode.
    fn cancel_delete(&mut self) {
        self.mode = AppMode::Navigating;
    }
}

/// Launches the TUI editor for a manifest file.
pub fn launch_editor(file_path: &Path) -> Result<()> {
    let conf = Ini::load_from_file(file_path)?;
    let mut app = App::new(conf, file_path);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, &mut app);

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
fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            match app.mode {
                AppMode::Navigating => {
                    if let Event::Key(key) = event::read()? {
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('s') => {
                                app.save_to_file().unwrap_or_else(|_| {});
                            },
                            KeyCode::Down | KeyCode::Char('j') => app.next(),
                            KeyCode::Up | KeyCode::Char('k')=> app.previous(),
                            KeyCode::Enter => app.start_editing(),
                            KeyCode::Char('d') => app.start_confirm_delete(), // --- NEW ---
                            _ => {}
                        }
                    }
                }
                AppMode::Editing => {
                    if let Event::Key(key) = event::read()? {
                        match key.code {
                            KeyCode::Enter => app.submit_editing(),
                            KeyCode::Esc => app.cancel_editing(),
                            _ => {
                                app.input.handle_event(&Event::Key(key));
                            }
                        }
                    }
                }
                // --- NEW: Handle key input in the delete confirmation mode ---
                AppMode::ConfirmDelete => {
                    if let Event::Key(key) = event::read()? {
                        match key.code {
                            KeyCode::Char('y') | KeyCode::Char('Y') => app.delete_selected(),
                            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => app.cancel_delete(),
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

/// Renders the user interface.
fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(size);

    let title_block = Block::default()
        .borders(Borders::ALL)
        .title(" Distromanifesto Editor ");
    f.render_widget(title_block, chunks[0]);

    // --- NEW: Update footer text based on mode ---
    let footer_text = match app.mode {
        AppMode::Navigating => " (q) Quit | (s) Save | (↑/↓) Nav | (Enter) Edit | (d) Delete ",
        AppMode::Editing => " (Enter) Accept | (Esc) Cancel ",
        AppMode::ConfirmDelete => " Delete selected item? (y/n) ",
    };
    let footer_block = Block::default()
        .borders(Borders::ALL)
        .title(footer_text);
    f.render_widget(footer_block, chunks[2]);

    let list = List::new(app.list_items.clone())
        .block(Block::default().title("Manifest").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[1], &mut app.state);

    // Render popups on top of everything else
    if let AppMode::Editing = app.mode {
        draw_editing_popup(f, app);
    } else if let AppMode::ConfirmDelete = app.mode {
        // --- NEW: Draw the delete confirmation popup ---
        draw_delete_popup(f);
    }
}

/// Helper function to draw the editing popup
fn draw_editing_popup<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let area = centered_rect(60, 20, f.size());
    f.render_widget(Clear, area);

    let (section, key) = app.editing_key.as_ref().unwrap();
    let title = format!(" Edit Value for [{section}] -> {key} ");

    let width = area.width.max(3) - 3;
    let scroll = app.input.visual_scroll(width as usize);
    
    let input = Paragraph::new(app.input.value())
        .style(Style::default().fg(Color::Yellow))
        .scroll((0, scroll as u16))
        .block(Block::default().borders(Borders::ALL).title(title));
    
    f.render_widget(input, area);

    f.set_cursor(
        area.x + 1 + (app.input.visual_cursor().max(scroll) - scroll) as u16,
        area.y + 1,
    )
}

// --- NEW: Helper function to draw the delete confirmation popup ---
/// Helper function to draw the delete confirmation popup
fn draw_delete_popup<B: Backend>(f: &mut Frame<B>) {
    let area = centered_rect(40, 20, f.size());
    f.render_widget(Clear, area); // Clear the area

    let text = Paragraph::new("Are you sure you want to delete this item?\n\n(y/n)")
        .style(Style::default().fg(Color::Red))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Confirm Delete ")
                .title_style(Style::default().fg(Color::Red))
                .border_style(Style::default().fg(Color::Red))
        );
    
    f.render_widget(text, area);
}


/// Helper function to create a centered rectangle for the popup.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}