use anyhow::{anyhow, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use distro_ini::Ini; // Use our aliased crate
use ratatui::{
    backend::Backend,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    terminal::{Frame, Terminal},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    prelude::*,
};
use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
    time::Duration,
};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

// --- Schema Definition ---

#[derive(Clone, Copy, PartialEq, Eq)]
enum KeyType {
    String,
    StringList,
    Bool,
    HomeDir,
}

#[derive(Clone)]
struct SchemaItem {
    key: &'static str,
    key_type: KeyType,
}

const SCHEMA: &[SchemaItem] = &[
    SchemaItem { key: "image", key_type: KeyType::String },
    SchemaItem { key: "clone", key_type: KeyType::String },
    SchemaItem { key: "home", key_type: KeyType::HomeDir },
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

// --- NEW: Constant for the custom path option ---
const CUSTOM_HOME_PATH_OPTION: &str = "[ Type Custom Path ]";

struct Schema {
    map: HashMap<&'static str, KeyType>,
}
impl Schema {
    fn new() -> Self {
        let map = SCHEMA.iter().map(|item| (item.key, item.key_type)).collect();
        Self { map }
    }
    fn get_type(&self, key: &str) -> KeyType {
        self.map.get(key).cloned().unwrap_or(KeyType::String)
    }
}

// --- App State ---

enum AppMode {
    Navigating,
    EditingString,
    EditingBool,
    EditingHome(ListState),
    EditingHomeCustom, // --- NEW: State for custom home path input ---
    ConfirmDelete,
    AddingKey(ListState),
    AddingValue,
    Saved,
}

#[derive(Clone)]
enum DisplayItem {
    Section(String),
    Property(String, String, String), // section, key, value
}

struct App<'a> {
    file_path: PathBuf,
    items: Vec<DisplayItem>,
    list_items: Vec<ListItem<'a>>,
    state: ListState,
    mode: AppMode,
    value_input: Input,
    current_section: String,
    current_add_item: Option<SchemaItem>,
    current_bool_value: bool,
    status_timer: u8,
    schema: Schema,
    home_dir_options: Vec<String>,
}

impl<'a> App<'a> {
    fn new(conf: Ini, file_path: &Path) -> App<'a> {
        let mut state = ListState::default();
        if !conf.is_empty() {
            state.select(Some(0));
        }

        let mut app = App {
            file_path: file_path.to_path_buf(),
            items: Vec::new(),
            list_items: Vec::new(),
            state,
            mode: AppMode::Navigating,
            value_input: Input::default(),
            current_section: "Global".to_string(),
            current_add_item: None,
            current_bool_value: true,
            status_timer: 0,
            schema: Schema::new(),
            home_dir_options: Vec::new(),
        };
        app.parse_conf_to_items(conf);
        app
    }

    fn parse_conf_to_items(&mut self, conf: Ini) {
        let mut items = Vec::new();
        for (sec, prop) in conf.iter() {
            let section_name = sec.unwrap_or("Global").to_string();
            items.push(DisplayItem::Section(section_name.clone()));
            
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
            if let DisplayItem::Property(_, key, value) = &self.items[index] {
                let value_clone = value.to_string();
                let key_clone = key.to_string();

                match self.schema.get_type(&key_clone) {
                    KeyType::Bool => {
                        self.current_bool_value = value_clone.parse().unwrap_or(true);
                        self.mode = AppMode::EditingBool;
                    }
                    KeyType::HomeDir => {
                        self.load_home_dir_options();
                        let mut list_state = ListState::default();
                        let current_idx = self.home_dir_options.iter().position(|h| *h == value_clone);
                        list_state.select(current_idx.or(Some(0)));
                        self.mode = AppMode::EditingHome(list_state);
                    }
                    KeyType::String | KeyType::StringList => {
                        self.value_input = Input::new(value_clone);
                        self.mode = AppMode::EditingString;
                    }
                }
            }
        }
    }

    fn cancel_editing(&mut self) {
        self.mode = AppMode::Navigating;
        self.value_input = Input::default();
        self.home_dir_options.clear();
    }

    // --- NEW: Helper function to apply the edit ---
    fn set_edited_value(&mut self, new_value: String) {
        if let Some(index) = self.state.selected() {
            if let DisplayItem::Property(section, key, _) = &self.items[index].clone() {
                
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
    }

    fn submit_string_editing(&mut self) {
        let new_value = self.value_input.value().to_string();
        self.set_edited_value(new_value);
    }

    fn submit_bool_editing(&mut self) {
        let new_value = self.current_bool_value.to_string();
        self.set_edited_value(new_value);
    }

    fn submit_home_editing(&mut self) {
        if let AppMode::EditingHome(list_state) = &self.mode {
             if let Some(selected_home_index) = list_state.selected() {
                let selected_option = self.home_dir_options[selected_home_index].clone();
                
                if selected_option == CUSTOM_HOME_PATH_OPTION {
                    // --- NEW: Transition to custom input mode ---
                    self.value_input = Input::default();
                    self.mode = AppMode::EditingHomeCustom;
                } else {
                    // --- This is now fixed to use the full path ---
                    self.set_edited_value(selected_option);
                }
            }
        }
    }

    // --- NEW: Submit logic for the custom home path input ---
    fn submit_home_custom_editing(&mut self) {
        let new_value = self.value_input.value().to_string();
        if !new_value.is_empty() {
            self.set_edited_value(new_value);
        } else {
            self.cancel_editing();
        }
    }

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
                    if *section != current_section && section == "Global" {
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
        self.status_timer = 10;
        Ok(())
    }

    fn delete_selected(&mut self) {
        if let Some(index) = self.state.selected() {
            if let DisplayItem::Property(..) = &self.items[index] {
                self.items.remove(index);
                self.list_items.remove(index);

                let new_index = if index > 0 { index - 1 } else { 0 };
                if self.list_items.is_empty() {
                    self.state.select(None);
                } else {
                    self.state.select(Some(new_index));
                }
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

    fn start_adding(&mut self) {
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
            self.current_section = "Global".to_string(); 
        }

        let mut list_state = ListState::default();
        list_state.select(Some(0));
        self.mode = AppMode::AddingKey(list_state);
    }

    fn submit_key(&mut self) {
        if let AppMode::AddingKey(list_state) = &mut self.mode {
            if let Some(index) = list_state.selected() {
                let schema_item = SCHEMA[index].clone();
                self.current_add_item = Some(schema_item.clone());
                
                match schema_item.key_type {
                    KeyType::String | KeyType::StringList => {
                        self.value_input = Input::default();
                    }
                    KeyType::HomeDir => {
                        // When *adding*, we'll just use a text box for simplicity
                        // The smart editor is for *editing*
                        self.value_input = Input::default(); 
                    }
                    KeyType::Bool => {
                        self.current_bool_value = true;
                    }
                }
                self.mode = AppMode::AddingValue;
            }
        }
    }

    fn submit_value(&mut self) {
        if let Some(item) = &self.current_add_item {
            let new_key = item.key.to_string();
            let new_value = match item.key_type {
                KeyType::String | KeyType::StringList | KeyType::HomeDir => self.value_input.value().to_string(),
                KeyType::Bool => self.current_bool_value.to_string(),
            };

            let index = self.state.selected().unwrap_or(0);
            let insert_index = if self.items.is_empty() { 0 } else { index + 1 };
            
            let new_display_item = DisplayItem::Property(
                self.current_section.clone(),
                new_key.clone(),
                new_value.clone()
            );
            
            self.items.insert(insert_index, new_display_item);
            self.refresh_list_items_from_items();
            self.state.select(Some(insert_index));
            self.cancel_adding();
        }
    }

    fn cancel_adding(&mut self) {
        self.mode = AppMode::Navigating;
        self.current_add_item = None;
        self.value_input = Input::default();
        self.home_dir_options.clear();
    }

    fn toggle_bool_value(&mut self) {
        self.current_bool_value = !self.current_bool_value;
    }

    // --- MODIFIED: This function now saves full paths and adds the custom option ---
    fn load_home_dir_options(&mut self) {
        let mut options = vec![
            "host".to_string(),
            "none".to_string(),
        ];
        
        if let Ok(homes_path) = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not find home directory"))
            .map(|mut p| { p.push(".distromanifesto/homes"); p })
        {
            if let Ok(entries) = fs::read_dir(homes_path) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        if let Some(dir_name) = entry.path().file_name() {
                            // --- FIX: Save the full tilde-expanded path ---
                            let path_str = format!("~/.distromanifesto/homes/{}", dir_name.to_string_lossy());
                            options.push(path_str);
                        }
                    }
                }
            }
        }
        
        // --- NEW: Add the custom option ---
        options.push(CUSTOM_HOME_PATH_OPTION.to_string());
        self.home_dir_options = options;
    }
}

// --- Main TUI Functions ---

pub fn launch_editor(file_path: &Path) -> Result<()> {
    let conf = Ini::load_from_file(file_path)?;
    let mut app = App::new(conf, file_path);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout); // <-- Corrected: use `stdout`
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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let AppMode::Saved = app.mode {
                if app.status_timer > 0 {
                    app.status_timer -= 1;
                } else {
                    app.mode = AppMode::Navigating;
                }
                if let Ok(true) = event::poll(Duration::from_millis(0)) {
                   if let Event::Key(_) = event::read()? {
                        app.mode = AppMode::Navigating;
                        app.status_timer = 0;
                   }
                }
                continue;
            }
            
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
                    AppMode::EditingString => {
                        match key.code {
                            KeyCode::Enter => app.submit_string_editing(),
                            KeyCode::Esc => app.cancel_editing(),
                            _ => { app.value_input.handle_event(&Event::Key(key)); }
                        }
                    }
                    AppMode::EditingBool => {
                         match key.code {
                            KeyCode::Enter => app.submit_bool_editing(),
                            KeyCode::Esc => app.cancel_editing(),
                            KeyCode::Char(' ') | KeyCode::Tab | KeyCode::Left | KeyCode::Right => {
                                app.toggle_bool_value();
                            }
                            _ => {}
                        }
                    }
                    AppMode::EditingHome(ref mut list_state) => {
                        match key.code {
                            KeyCode::Enter => app.submit_home_editing(),
                            KeyCode::Esc => app.cancel_editing(),
                            KeyCode::Down | KeyCode::Char('j') => {
                                let i = list_state.selected().unwrap_or(0);
                                let next = if i >= app.home_dir_options.len() - 1 { 0 } else { i + 1 };
                                list_state.select(Some(next));
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                let i = list_state.selected().unwrap_or(0);
                                let prev = if i == 0 { app.home_dir_options.len() - 1 } else { i - 1 };
                                list_state.select(Some(prev));
                            }
                            _ => {}
                        }
                    }
                    // --- NEW: Key handling for custom home input ---
                    AppMode::EditingHomeCustom => {
                         match key.code {
                            KeyCode::Enter => app.submit_home_custom_editing(),
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
                                KeyType::String | KeyType::StringList | KeyType::HomeDir => {
                                    match key.code {
                                        KeyCode::Enter => app.submit_value(),
                                        KeyCode::Esc => app.cancel_adding(),
                                        _ => { app.value_input.handle_event(&Event::Key(key)); }
                                    }
                                }
                                KeyType::Bool => {
                                    match key.code {
                                        KeyCode::Enter => app.submit_value(),
                                        KeyCode::Esc => app.cancel_adding(),
                                        KeyCode::Char(' ') | KeyCode::Tab | KeyCode::Left | KeyCode::Right => {
                                            app.toggle_bool_value();
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                    AppMode::Saved => {}
                }
            }
        }
    }
}

// --- UI Drawing Functions ---

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

    let (footer_text, footer_style) = match &app.mode {
        AppMode::Navigating => (" (q) Quit | (s) Save | (↑/↓) Nav | (a) Add | (d) Delete | (Enter) Edit ".to_string(), Style::default()),
        AppMode::EditingString => (" (Enter) Accept | (Esc) Cancel ".to_string(), Style::default()),
        AppMode::EditingBool => (" (Space/←/→) Toggle | (Enter) Accept | (Esc) Cancel ".to_string(), Style::default()),
        AppMode::EditingHome(_) => (" (↑/↓) Select Home | (Enter) Accept | (Esc) Cancel ".to_string(), Style::default()),
        AppMode::EditingHomeCustom => (" (Enter) Accept Custom Path | (Esc) Cancel ".to_string(), Style::default()),
        AppMode::ConfirmDelete => (" Delete selected item? (y/n) ".to_string(), Style::default().fg(Color::Red)),
        AppMode::AddingKey(_) => (" (↑/↓) Select Key | (Enter) Next | (Esc) Cancel ".to_string(), Style::default()),
        AppMode::AddingValue => {
            let text = if let Some(item) = &app.current_add_item {
                match item.key_type {
                    KeyType::Bool => " (Space/←/→) Toggle | (Enter) Accept | (Esc) Cancel ",
                    _ => " (Enter) Accept | (Esc) Cancel ",
                }
            } else { "" };
            (text.to_string(), Style::default())
        },
        AppMode::Saved => (" File saved successfully! ".to_string(), Style::default().fg(Color::Green)),
    };
    
    let footer_block = Block::default()
        .borders(Borders::ALL)
        .title(footer_text)
        .title_style(footer_style)
        .border_style(footer_style);
    f.render_widget(footer_block, chunks[2]);

    let list = List::new(app.list_items.clone())
        .block(Block::default().title("Manifest").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[1], &mut app.state);

    match &mut app.mode {
        AppMode::EditingString => draw_edit_string_popup(f, app),
        AppMode::EditingBool => draw_edit_bool_popup(f, app),
        AppMode::EditingHome(list_state) => {
            draw_edit_home_popup(f, &app.home_dir_options, list_state)
        }
        AppMode::EditingHomeCustom => draw_edit_home_custom_popup(f, app), // --- NEW ---
        AppMode::ConfirmDelete => draw_delete_popup(f),
        AppMode::AddingKey(list_state) => draw_add_key_popup(f, list_state),
        AppMode::AddingValue => draw_add_value_popup(f, app),
        AppMode::Navigating | AppMode::Saved => {}
    }
}

fn draw_edit_string_popup<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let area = centered_rect(60, 20, f.size());
    f.render_widget(Clear, area);

    let title = if let Some(index) = app.state.selected() {
        if let DisplayItem::Property(_, key, _) = &app.items[index] {
             format!(" Edit Value for: {key} ")
        } else { " Edit Value ".to_string() }
    } else { " Edit Value ".to_string() };
    
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

fn draw_bool_toggle<B: Backend>(f: &mut Frame<B>, area: Rect, title: String, value: bool) {
    let block = Block::default().borders(Borders::ALL).title(title);
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .margin(1)
        .split(area);

    let true_style = if value {
        Style::default().fg(Color::Green).add_modifier(Modifier::REVERSED)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let false_style = if !value {
        Style::default().fg(Color::Red).add_modifier(Modifier::REVERSED)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let true_text = Paragraph::new(" true ").style(true_style).alignment(Alignment::Center);
    let false_text = Paragraph::new(" false ").style(false_style).alignment(Alignment::Center);
    
    f.render_widget(true_text, layout[0]);
    f.render_widget(false_text, layout[1]);
}

fn draw_edit_bool_popup<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let area = centered_rect(40, 20, f.size());
    let title = if let Some(index) = app.state.selected() {
        if let DisplayItem::Property(_, key, _) = &app.items[index] {
             format!(" Edit Value for: {key} ")
        } else { " Edit Value ".to_string() }
    } else { " Edit Value ".to_string() };
    
    draw_bool_toggle(f, area, title, app.current_bool_value);
}

fn draw_edit_home_popup<B: Backend>(
    f: &mut Frame<B>,
    home_dir_options: &Vec<String>,
    list_state: &mut ListState,
) {
    let area = centered_rect(50, 80, f.size());
    f.render_widget(Clear, area);

    let items: Vec<ListItem> = home_dir_options
        .iter()
        .map(|opt| {
            // --- NEW: Style the custom option ---
            if opt == CUSTOM_HOME_PATH_OPTION {
                ListItem::new(opt.as_str()).style(Style::default().fg(Color::Yellow))
            } else {
                ListItem::new(opt.as_str())
            }
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Select Home Directory"))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, list_state);
}

// --- NEW: Popup for custom home path input ---
fn draw_edit_home_custom_popup<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let area = centered_rect(60, 20, f.size());
    f.render_widget(Clear, area);

    let title = " Enter Custom Home Path ";
    
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

fn draw_add_value_popup<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let area = centered_rect(60, 25, f.size());
    let item = app.current_add_item.as_ref().unwrap();
    let title = format!(" Enter Value for: {} ", item.key);
    
    match item.key_type {
        KeyType::String | KeyType::StringList | KeyType::HomeDir => {
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
            draw_bool_toggle(f, area, title, app.current_bool_value);
        }
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