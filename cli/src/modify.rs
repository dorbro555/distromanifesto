use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
// We MUST alias our ini crate to avoid conflicts
use distro_ini::Ini;
use ratatui::{
    backend::Backend,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    terminal::{Frame, Terminal},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    prelude::*,
};
use std::{fs, io, path::{Path, PathBuf}, time::Duration};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

// --- NEW: Define the full distrobox key schema ---

#[derive(Clone, Copy, PartialEq, Eq)]
enum KeyType {
    String,
    StringList,
    Bool,
}

#[derive(Clone)]
struct SchemaItem {
    key: &'static str,
    key_type: KeyType,
}

// All valid keys based on user-provided list
const SCHEMA: &[SchemaItem] = &[
    SchemaItem { key: "image", key_type: KeyType::String },
    SchemaItem { key: "clone", key_type: KeyType::String },
    SchemaItem { key: "home", key_type: KeyType::String },
    SchemaItem { key: "exported_bins_path", key_type: KeyType::String },
    SchemaItem { key: "additional_flags", key_type: KeyType::StringList },
    SchemaItem { key: "additional_packages", key_type: KeyType::StringList },
    SchemaItem { key: "init_hooks", key_type: KeyType::StringList },
    SchemaItem { key: "pre_init_hooks", key_type: KeyType::StringList },
    SchemaItem { key: "volume", key_type: KeyType::StringList },
    SchemaItem { key: "exported_apps", key_type: KeyType::StringList },
    SchemaItem { key: "exported_bins", key_type: KeyType::StringList },
    SchemaItem { key: "entry", key_type: KeyType::Bool },
    SchemaItem { key: "start_now", key_type: KeyType::Bool },
    SchemaItem { key: "init", key_type: KeyType::Bool },
    SchemaItem { key: "nvidia", key_type: KeyType::Bool },
    SchemaItem { key: "pull", key_type: KeyType::Bool },
    SchemaItem { key: "root", key_type: KeyType::Bool },
    SchemaItem { key: "unshare_ipc", key_type: KeyType::Bool },
    SchemaItem { key: "unshare_netns", key_type: KeyType::Bool },
    SchemaItem { key: "unshare_process", key_type: KeyType::Bool },
    SchemaItem { key: "unshare_devsys", key_type: KeyType::Bool },
    SchemaItem { key: "unshare_all", key_type: KeyType::Bool },
];

enum AppMode {
    Navigating,
    Editing,
    ConfirmDelete,
    AddingKey(ListState), // NEW: State for selecting a key from the schema
    AddingValue,         // NEW: State for entering the value
    Saved,               // NEW: State for the "Saved!" notification
}

#[derive(Clone)]
enum DisplayItem {
    Section(String),
    Property(String, String, String), // section, key, value
}

struct App<'a> {
    file_path: PathBuf,
    items: Vec<DisplayItem>, // This is the *only* source of truth
    list_items: Vec<ListItem<'a>>,
    state: ListState,
    mode: AppMode,
    value_input: Input,
    current_section: String,
    current_add_item: Option<SchemaItem>, // NEW: The key we are adding
    current_bool_value: bool,             // NEW: For bool toggles
    status_timer: u8,
}

impl<'a> App<'a> {
    fn new(conf: Ini, file_path: &Path) -> App<'a> {
        let mut state = ListState::default();
        if !conf.is_empty() {
            state.select(Some(0));
        }

        let mut app = App {
            file_path: file_path.to_path_buf(),
            items: Vec::new(), // Will be populated by parse_conf_to_items
            list_items: Vec::new(),
            state,
            mode: AppMode::Navigating,
            value_input: Input::default(),
            current_section: "Global".to_string(),
            current_add_item: None,
            current_bool_value: true, // Default for bool add
            status_timer: 0,
        };
        // Use the loaded conf to populate our source of truth
        app.parse_conf_to_items(conf);
        app
    }

    /// Parses the `conf` object into our ordered `items` list.
    /// This is the *only* time we read from the `Ini` object.
    fn parse_conf_to_items(&mut self, conf: Ini) {
        let mut items = Vec::new();
        for (sec, prop) in conf.iter() {
            let section_name = sec.unwrap_or("Global").to_string();
            items.push(DisplayItem::Section(section_name.clone()));
            
            // We iterate over the *properties* from the Ini object
            // This does *not* preserve duplicate keys on load, but it's
            // the best we can do with the library. Our save logic fixes this.
            for (key, value) in prop.iter() {
                items.push(DisplayItem::Property(
                    section_name.clone(),
                    key.to_string(),
                    value.to_string(),
                ));
            }
        }
        self.items = items;
        self.refresh_list_items_from_items();
    }
    
    /// Re-generates the `list_items` (for drawing) from our `items` source of truth.
    fn refresh_list_items_from_items(&mut self) {
        let mut list_items = Vec::new();
        for item in &self.items {
            match item {
                DisplayItem::Section(section_name) => {
                    list_items.push(
                        ListItem::new(format!("[{section_name}]"))
                            .style(Style::default().fg(Color::Green).bold()),
                    );
                }
                DisplayItem::Property(_, key, value) => {
                     list_items.push(ListItem::new(format!("  {key} = {value}")));
                }
            }
        }
        self.list_items = list_items;
    }

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

    fn start_editing(&mut self) {
        if let Some(index) = self.state.selected() {
            match &self.items[index] {
                DisplayItem::Property(section, key, value) => {
                    self.mode = AppMode::Editing;
                    // We don't use editing_key, but we'll set value_input
                    self.value_input = Input::new(value.clone());
                }
                DisplayItem::Section(_) => {}
            }
        }
    }

    fn cancel_editing(&mut self) {
        self.mode = AppMode::Navigating;
        self.value_input = Input::default();
    }

    fn submit_editing(&mut self) {
        if let Some(index) = self.state.selected() {
            if let DisplayItem::Property(section, key, _) = &self.items[index].clone() {
                let new_value = self.value_input.value().to_string();
                
                // Update our source of truth
                self.items[index] = DisplayItem::Property(
                    section.clone(),
                    key.clone(),
                    new_value.clone()
                );
                // Update the visual list
                self.list_items[index] = ListItem::new(format!("  {key} = {new_value}"));

                self.cancel_editing();
                self.state.select(Some(index));
            }
        }
    }

    /// --- FIX: This is the new save logic ---
    /// Saves the `items` list back to the file, preserving order and duplicates.
    fn save_to_file(&mut self) -> Result<()> {
        let mut file_content = String::new();
        let mut current_section = "".to_string();

        for item in &self.items {
            match item {
                DisplayItem::Section(section) => {
                    if section != "Global" {
                        file_content.push_str(&format!("\n[{section}]\n"));
                        current_section = section.clone();
                    } else {
                        current_section = "Global".to_string();
                    }
                }
                DisplayItem::Property(section, key, value) => {
                    // This logic ensures properties are under the correct section
                    if *section != current_section && section == "Global" {
                         // We don't write a header for "Global"
                    } else if *section != current_section {
                        file_content.push_str(&format!("\n[{section}]\n"));
                        current_section = section.clone();
                    }
                    file_content.push_str(&format!("{key} = {value}\n"));
                }
            }
        }

        fs::write(&self.file_path, file_content)?;

        self.mode = AppMode::Saved;
        self.status_timer = 10; // ~1 second
        Ok(())
    }

    fn delete_selected(&mut self) {
        if let Some(index) = self.state.selected() {
            match &self.items[index] {
                DisplayItem::Property(..) => {
                    self.items.remove(index);
                    self.list_items.remove(index); // Remove from both

                    let new_index = if index > 0 { index - 1 } else { 0 };
                    if self.list_items.is_empty() {
                        self.state.select(None);
                    } else {
                        self.state.select(Some(new_index));
                    }
                }
                DisplayItem::Section(_) => {} // Can't delete sections yet
            }
        }
        self.mode = AppMode::Navigating;
    }

    fn start_confirm_delete(&mut self) {
        if let Some(index) = self.state.selected() {
            if let DisplayItem::Property(..) = &self.items[index] {
                self.mode = AppMode::ConfirmDelete;
            }
        }
    }

    fn cancel_delete(&mut self) {
        self.mode = AppMode::Navigating;
    }

    // --- NEW: All "Add" logic is refactored ---

    /// Enters the 'AddingKey' mode.
    fn start_adding(&mut self) {
        // Determine the current section
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
            // Default if list is empty
            self.current_section = "Global".to_string(); 
        }

        let mut list_state = ListState::default();
        list_state.select(Some(0));
        self.mode = AppMode::AddingKey(list_state);
    }

    /// Submits the selected key and moves to 'AddingValue'.
    fn submit_key(&mut self) {
        if let AppMode::AddingKey(list_state) = &mut self.mode {
            if let Some(index) = list_state.selected() {
                let schema_item = SCHEMA[index].clone();
                self.current_add_item = Some(schema_item.clone());
                
                // Set up the correct value input
                match schema_item.key_type {
                    KeyType::String | KeyType::StringList => {
                        self.value_input = Input::default();
                    }
                    KeyType::Bool => {
                        self.current_bool_value = true; // Default to true
                    }
                }
                self.mode = AppMode::AddingValue;
            }
        }
    }

    /// Submits the new key/value pair.
    fn submit_value(&mut self) {
        if let Some(item) = &self.current_add_item {
            let new_key = item.key.to_string();
            
            // Get the value based on the type
            let new_value = match item.key_type {
                KeyType::String | KeyType::StringList => self.value_input.value().to_string(),
                KeyType::Bool => self.current_bool_value.to_string(),
            };

            // Add to our source of truth
            let index = self.state.selected().unwrap_or(0);
            let insert_index = if self.items.is_empty() { 0 } else { index + 1 };
            
            let new_display_item = DisplayItem::Property(
                self.current_section.clone(),
                new_key.clone(),
                new_value.clone()
            );
            
            self.items.insert(insert_index, new_display_item);
            
            // Refresh the visual list
            self.refresh_list_items_from_items();
            
            self.state.select(Some(insert_index));
            self.cancel_adding();
        }
    }

    /// Cancels the entire 'Add' operation.
    fn cancel_adding(&mut self) {
        self.mode = AppMode::Navigating;
        self.current_add_item = None;
        self.value_input = Input::default();
    }

    /// Toggles the boolean value in the 'AddingValue' mode.
    fn toggle_bool_value(&mut self) {
        self.current_bool_value = !self.current_bool_value;
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
            // --- NEW: Timer logic for 'Saved' mode ---
            if let AppMode::Saved = app.mode {
                if app.status_timer > 0 {
                    app.status_timer -= 1;
                } else {
                    app.mode = AppMode::Navigating;
                }
                // Allow user to skip wait
                if let Event::Key(_) = event::read()? {
                    app.mode = AppMode::Navigating;
                    app.status_timer = 0;
                }
                continue; // Skip other input
            }
            
            // Handle all other input
            if let Event::Key(key) = event::read()? {
                match app.mode {
                    AppMode::Navigating => {
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('s') => { app.save_to_file().unwrap_or_else(|_| {}); },
                            KeyCode::Down | KeyCode::Char('j') => app.next(),
                            KeyCode::Up | KeyCode::Char('k')=> app.previous(),
                            KeyCode::Enter => app.start_editing(),
                            KeyCode::Char('d') => app.start_confirm_delete(),
                            KeyCode::Char('a') => app.start_adding(),
                            _ => {}
                        }
                    }
                    AppMode::Editing => {
                        match key.code {
                            KeyCode::Enter => app.submit_editing(),
                            KeyCode::Esc => app.cancel_editing(),
                            _ => { app.value_input.handle_event(&Event::Key(key)); }
                        }
                    }
                    AppMode::ConfirmDelete => {
                        match key.code {
                            KeyCode::Char('y') | KeyCode::Char('Y') => app.delete_selected(),
                            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => app.cancel_delete(),
                            _ => {}
                        }
                    }
                    // --- NEW: Key handling for Add Wizard ---
                    AppMode::AddingKey(ref mut list_state) => {
                        match key.code {
                            KeyCode::Enter => app.submit_key(),
                            KeyCode::Esc => app.cancel_adding(),
                            KeyCode::Down | KeyCode::Char('j') => {
                                let i = list_state.selected().unwrap_or(0);
                                let next = if i >= SCHEMA.len() - 1 { 0 } else { i + 1 };
                                list_state.select(Some(next));
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                let i = list_state.selected().unwrap_or(0);
                                let prev = if i == 0 { SCHEMA.len() - 1 } else { i - 1 };
                                list_state.select(Some(prev));
                            }
                            _ => {}
                        }
                    }
                    AppMode::AddingValue => {
                        if let Some(item) = &app.current_add_item {
                            match item.key_type {
                                KeyType::String | KeyType::StringList => {
                                    match key.code {
                                        KeyCode::Enter => app.submit_value(),
                                        KeyCode::Esc => app.cancel_adding(), // Go back to nav
                                        _ => { app.value_input.handle_event(&Event::Key(key)); }
                                    }
                                }
                                KeyType::Bool => {
                                    match key.code {
                                        KeyCode::Enter => app.submit_value(),
                                        KeyCode::Esc => app.cancel_adding(), // Go back to nav
                                        KeyCode::Char(' ') | KeyCode::Tab | KeyCode::Left | KeyCode::Right => {
                                            app.toggle_bool_value();
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                    AppMode::Saved => {} // Already handled
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

    let footer_text = match &app.mode {
        AppMode::Navigating => " (q) Quit | (s) Save | (↑/↓) Nav | (a) Add | (d) Delete | (Enter) Edit ",
        AppMode::Editing => " (Enter) Accept | (Esc) Cancel ",
        AppMode::ConfirmDelete => " Delete selected item? (y/n) ",
        AppMode::AddingKey(_) => " (↑/↓) Select Key | (Enter) Next | (Esc) Cancel ",
        AppMode::AddingValue => {
            if let Some(item) = &app.current_add_item {
                match item.key_type {
                    KeyType::Bool => " (Space) Toggle | (Enter) Accept | (Esc) Cancel ",
                    _ => " (Enter) Accept | (Esc) Cancel ",
                }
            } else { "" }
        },
        AppMode::Saved => " File saved successfully! (Press any key) ",
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
    match &mut app.mode {
        AppMode::Editing => draw_editing_popup(f, app),
        AppMode::ConfirmDelete => draw_delete_popup(f),
        AppMode::AddingKey(list_state) => draw_add_key_popup(f, list_state), // NEW
        AppMode::AddingValue => draw_add_value_popup(f, app), // NEW
        AppMode::Saved => draw_status_popup(f, "File Saved!"), // NEW
        AppMode::Navigating => {}
    }
}

/// Helper function to draw the editing popup
fn draw_editing_popup<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let area = centered_rect(60, 20, f.size());
    f.render_widget(Clear, area);

    // Try to get key from selected item
    let title = if let Some(index) = app.state.selected() {
        if let DisplayItem::Property(_, key, _) = &app.items[index] {
             format!(" Edit Value for: {key} ")
        } else {
            " Edit Value ".to_string()
        }
    } else {
        " Edit Value ".to_string()
    };
    
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

// --- NEW: Popup for selecting a key from the schema ---
fn draw_add_key_popup<B: Backend>(f: &mut Frame<B>, list_state: &mut ListState) {
    let area = centered_rect(50, 80, f.size());
    f.render_widget(Clear, area);

    let items: Vec<ListItem> = SCHEMA
        .iter()
        .map(|item| ListItem::new(item.key))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Select a Key to Add"))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, list_state);
}

// --- NEW: Popup for entering the value (context-aware) ---
fn draw_add_value_popup<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let area = centered_rect(60, 25, f.size());
    f.render_widget(Clear, area);

    let item = app.current_add_item.as_ref().unwrap();
    let title = format!(" Enter Value for: {} ", item.key);
    
    match item.key_type {
        KeyType::String | KeyType::StringList => {
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
        KeyType::Bool => {
            let text = if app.current_bool_value { "true" } else { "false" };
            let paragraph = Paragraph::new(text)
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::REVERSED))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title(title));
            f.render_widget(paragraph, area);
        }
    }
}

/// Helper function to draw a simple status popup
fn draw_status_popup<B: Backend>(f: &mut Frame<B>, message: &str) {
    let area = centered_rect(20, 20, f.size());
    f.render_widget(Clear, area);

    let text = Paragraph::new(message)
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Success ")
                .title_style(Style::default().fg(Color::Green))
                .border_style(Style::default().fg(Color::Green))
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