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
    ConfirmDelete,
    AddingKey,   // --- NEW: State for entering the new key ---
    AddingValue, // --- NEW: State for entering the new value ---
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
    value_input: Input, // Renamed from 'input' for clarity
    key_input: Input,   // --- NEW: Input for the new key ---
    current_section: String, // --- NEW: Tracks which section to add to ---
    editing_key: Option<(String, String)>, 
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
            value_input: Input::default(),
            key_input: Input::default(), // --- NEW ---
            current_section: "Global".to_string(), // Default section
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
                    self.value_input = Input::new(value.clone()); // Load value into input
                }
                DisplayItem::Section(_) => {} // Can't edit sections
            }
        }
    }

    /// Cancels the current edit.
    fn cancel_editing(&mut self) {
        self.mode = AppMode::Navigating;
        self.editing_key = None;
        self.value_input = Input::default();
    }

    /// Submits the current edit, saving it to the `conf`.
    fn submit_editing(&mut self) {
        if let (Some((section, key)), Some(index)) = 
            (self.editing_key.clone(), self.state.selected()) {
            
            let new_value = self.value_input.value().to_string(); // Use value_input
            
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

    /// Deletes the currently selected item.
    fn delete_selected(&mut self) {
        if let Some(index) = self.state.selected() {
            match &self.items[index] {
                DisplayItem::Property(section, key, _) => {
                    let section_name = if section == "Global" { None } else { Some(section.as_str()) };
                    self.conf.delete_from(section_name, key);

                    self.items.remove(index);
                    self.list_items.remove(index);

                    let new_index = if index > 0 { index - 1 } else { 0 };
                    if self.list_items.is_empty() {
                        self.state.select(None);
                    } else {
                        self.state.select(Some(new_index));
                    }
                }
                DisplayItem::Section(_) => {}
            }
        }
        self.mode = AppMode::Navigating;
    }

    /// Enters delete confirmation mode if a property is selected.
    fn start_confirm_delete(&mut self) {
        if let Some(index) = self.state.selected() {
            if let DisplayItem::Property(..) = &self.items[index] {
                self.mode = AppMode::ConfirmDelete;
            }
        }
    }

    /// Exits delete confirmation mode.
    fn cancel_delete(&mut self) {
        self.mode = AppMode::Navigating;
    }

    // --- NEW: Functions for adding a new property ---

    /// Enters the 'AddingKey' mode.
    fn start_adding(&mut self) {
        // Determine the current section from the selected item
        if let Some(index) = self.state.selected() {
            match &self.items[index] {
                DisplayItem::Section(section) => {
                    self.current_section = section.clone();
                }
                DisplayItem::Property(section, _, _) => {
                    self.current_section = section.clone();
                }
            }
        } else {
            // If list is empty, default to "Global" or the first section if it exists
            self.current_section = self.conf.sections().next().unwrap_or(Some("Global")).unwrap_or("Global").to_string();
        }

        self.key_input = Input::default();
        self.value_input = Input::default();
        self.mode = AppMode::AddingKey;
    }

    /// Moves from 'AddingKey' to 'AddingValue'.
    fn submit_key(&mut self) {
        if !self.key_input.value().is_empty() {
            self.mode = AppMode::AddingValue;
        }
    }

    /// Submits the new key/value pair and adds it to the list.
    fn submit_value(&mut self) {
        let new_key = self.key_input.value().to_string();
        let new_value = self.value_input.value().to_string();

        if new_key.is_empty() {
            self.cancel_adding();
            return;
        }

        // 1. Add to conf
        let section_name = if self.current_section == "Global" { None } else { Some(self.current_section.as_str()) };
        self.conf.with_section(section_name).set(&new_key, &new_value);

        // 2. Add to display lists
        // We'll add it right after the currently selected item
        let index = self.state.selected().unwrap_or(0);
        
        let new_item = DisplayItem::Property(
            self.current_section.clone(),
            new_key.clone(),
            new_value.clone()
        );
        let new_list_item = ListItem::new(format!("  {new_key} = {new_value}"));

        let insert_index = if self.items.is_empty() { 0 } else { index + 1 };
        
        self.items.insert(insert_index, new_item);
        self.list_items.insert(insert_index, new_list_item);

        // 3. Select the new item
        self.state.select(Some(insert_index));
        
        // 4. Go back to navigating
        self.cancel_adding();
    }

    /// Cancels the entire 'Add' operation.
    fn cancel_adding(&mut self) {
        self.mode = AppMode::Navigating;
        self.key_input = Input::default();
        self.value_input = Input::default();
    }

    /// Goes back from 'AddingValue' to 'AddingKey'.
    fn back_to_key(&mut self) {
        self.mode = AppMode::AddingKey;
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
                            KeyCode::Char('s') => { app.save_to_file().unwrap_or_else(|_| {}); },
                            KeyCode::Down | KeyCode::Char('j') => app.next(),
                            KeyCode::Up | KeyCode::Char('k')=> app.previous(),
                            KeyCode::Enter => app.start_editing(),
                            KeyCode::Char('d') => app.start_confirm_delete(),
                            KeyCode::Char('a') => app.start_adding(), // --- NEW ---
                            _ => {}
                        }
                    }
                }
                AppMode::Editing => {
                    if let Event::Key(key) = event::read()? {
                        match key.code {
                            KeyCode::Enter => app.submit_editing(),
                            KeyCode::Esc => app.cancel_editing(),
                            _ => { app.value_input.handle_event(&Event::Key(key)); }
                        }
                    }
                }
                AppMode::ConfirmDelete => {
                    if let Event::Key(key) = event::read()? {
                        match key.code {
                            KeyCode::Char('y') | KeyCode::Char('Y') => app.delete_selected(),
                            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => app.cancel_delete(),
                            _ => {}
                        }
                    }
                }
                // --- NEW: Key handling for the Add states ---
                AppMode::AddingKey => {
                    if let Event::Key(key) = event::read()? {
                        match key.code {
                            KeyCode::Enter => app.submit_key(),
                            KeyCode::Esc => app.cancel_adding(),
                            _ => { app.key_input.handle_event(&Event::Key(key)); }
                        }
                    }
                }
                AppMode::AddingValue => {
                    if let Event::Key(key) = event::read()? {
                        match key.code {
                            KeyCode::Enter => app.submit_value(),
                            KeyCode::Esc => app.back_to_key(), // Go back to editing the key
                            _ => { app.value_input.handle_event(&Event::Key(key)); }
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

    // --- NEW: Update footer text ---
    let footer_text = match app.mode {
        AppMode::Navigating => " (q) Quit | (s) Save | (↑/↓) Nav | (a) Add | (d) Delete | (Enter) Edit ",
        AppMode::Editing => " (Enter) Accept | (Esc) Cancel ",
        AppMode::ConfirmDelete => " Delete selected item? (y/n) ",
        AppMode::AddingKey => " Enter Key: (Enter) Next | (Esc) Cancel ",
        AppMode::AddingValue => " Enter Value: (Enter) Accept | (Esc) Edit Key ",
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

    // Render popups on top
    match app.mode {
        AppMode::Editing => draw_editing_popup(f, app),
        AppMode::ConfirmDelete => draw_delete_popup(f),
        AppMode::AddingKey | AppMode::AddingValue => draw_add_popup(f, app), // --- NEW ---
        _ => {}
    }
}

/// Helper function to draw the editing popup
fn draw_editing_popup<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let area = centered_rect(60, 20, f.size());
    f.render_widget(Clear, area);

    let (section, key) = app.editing_key.as_ref().unwrap();
    let title = format!(" Edit Value for [{section}] -> {key} ");

    let width = area.width.max(3) - 3;
    let scroll = app.value_input.visual_scroll(width as usize);
    
    let input = Paragraph::new(app.value_input.value())
        .style(Style::default().fg(Color::Yellow))
        .scroll((0, scroll as u16))
        .block(Block::default().borders(Borders::ALL).title(title));
    
    f.render_widget(input, area);

    f.set_cursor(
        area.x + 1 + (app.value_input.visual_cursor().max(scroll) - scroll) as u16,
        area.y + 1,
    )
}

/// Helper function to draw the delete confirmation popup
fn draw_delete_popup<B: Backend>(f: &mut Frame<B>) {
    let area = centered_rect(40, 20, f.size());
    f.render_widget(Clear, area);

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

// --- NEW: Helper function to draw the 'Add Property' popup ---
/// Helper function to draw the add property popup
fn draw_add_popup<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let area = centered_rect(60, 25, f.size());
    f.render_widget(Clear, area);

    let title = format!(" Add Property to [{}] ", app.current_section);

    let block = Block::default().borders(Borders::ALL).title(title);
    f.render_widget(block, area);

    // Create a 2-row layout inside the popup
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Key input
            Constraint::Length(3), // Value input
        ])
        .split(area);
    
    // --- Key Input ---
    let key_width = chunks[0].width.max(3) - 3;
    let key_scroll = app.key_input.visual_scroll(key_width as usize);
    let key_input = Paragraph::new(app.key_input.value())
        .style(match app.mode {
            AppMode::AddingKey => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        })
        .scroll((0, key_scroll as u16))
        .block(Block::default().borders(Borders::ALL).title("Key"));
    
    // --- Value Input ---
    let value_width = chunks[1].width.max(3) - 3;
    let value_scroll = app.value_input.visual_scroll(value_width as usize);
    let value_input = Paragraph::new(app.value_input.value())
        .style(match app.mode {
            AppMode::AddingValue => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        })
        .scroll((0, value_scroll as u16))
        .block(Block::default().borders(Borders::ALL).title("Value"));

    f.render_widget(key_input, chunks[0]);
    f.render_widget(value_input, chunks[1]);

    // --- Set Cursor ---
    match app.mode {
        AppMode::AddingKey => {
            f.set_cursor(
                chunks[0].x + 1 + (app.key_input.visual_cursor().max(key_scroll) - key_scroll) as u16,
                chunks[0].y + 1,
            )
        }
        AppMode::AddingValue => {
            f.set_cursor(
                chunks[1].x + 1 + (app.value_input.visual_cursor().max(value_scroll) - value_scroll) as u16,
                chunks[1].y + 1,
            )
        }
        _ => {}
    }
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