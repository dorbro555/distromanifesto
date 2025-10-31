use crate::constants::{CUSTOM_OCI_IMAGE_OPTION, DISTROBOX_IMAGES};
use anyhow::{anyhow, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::Backend,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    terminal::{Frame, Terminal},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use std::{
    collections::HashMap,
    fs,
    io::{self},
    path::{Path, PathBuf},
    time::Duration,
};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

// --- Schema Definition ---
#[derive(Clone, Copy, PartialEq, Eq)]
enum SelectorType {
    Home,
    Image,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum KeyType {
    String,
    StringList,
    Bool,
    HomeDir,
    Image,
}

#[derive(Clone)]
struct SchemaItem {
    key: &'static str,
    key_type: KeyType,
    description: &'static str,
}

const SCHEMA: &[SchemaItem] = &[
    SchemaItem {
        key: "image",
        key_type: KeyType::Image,
        description: "Container image to use (e.g., 'fedora-toolbox:40'). REQUIRED.",
    },
    SchemaItem {
        key: "clone",
        key_type: KeyType::String,
        description: "Name of an existing distrobox to clone from.",
    },
    SchemaItem {
        key: "home",
        key_type: KeyType::HomeDir,
        description: "The home directory strategy (e.g., 'host', 'none', or a custom path).",
    },
    SchemaItem {
        key: "exported_bins_path",
        key_type: KeyType::String,
        description: "Directory on the host to export binaries to (e.g., '~/.local/bin').",
    },
    SchemaItem {
        key: "additional_flags",
        key_type: KeyType::StringList,
        description: "Extra flags to pass to the container runtime (e.g., '--privileged').",
    },
    SchemaItem {
        key: "additional_packages",
        key_type: KeyType::StringList,
        description: "Packages to install on first boot (e.g., 'git', 'neovim').",
    },
    SchemaItem {
        key: "init_hooks",
        key_type: KeyType::StringList,
        description: "Commands to run inside the container on *every* start.",
    },
    SchemaItem {
        key: "pre_init_hooks",
        key_type: KeyType::StringList,
        description: "Commands to run *before* init on *every* start.",
    },
    SchemaItem {
        key: "volume",
        key_type: KeyType::StringList,
        description: "Mount a volume (e.g., '/host/path:/container/path').",
    },
    SchemaItem {
        key: "exported_apps",
        key_type: KeyType::StringList,
        description: "Apps to export to the host menu (e.g., 'firefox.desktop').",
    },
    SchemaItem {
        key: "exported_bins",
        key_type: KeyType::StringList,
        description: "Binaries to export to the host (e.g., 'nvim', 'npm').",
    },
    SchemaItem {
        key: "entry",
        key_type: KeyType::Bool,
        description: "DEPRECATED. Use 'init' or 'init_hooks' instead.",
    },
    SchemaItem {
        key: "start_now",
        key_type: KeyType::Bool,
        description: "Start the container immediately after creation (true/false).",
    },
    SchemaItem {
        key: "init",
        key_type: KeyType::Bool,
        description: "Use 'init' as the entrypoint (true/false).",
    },
    SchemaItem {
        key: "nvidia",
        key_type: KeyType::Bool,
        description: "Enable NVIDIA GPU support (true/false).",
    },
    SchemaItem {
        key: "pull",
        key_type: KeyType::Bool,
        description: "Force pull the image before creating (true/false).",
    },
    SchemaItem {
        key: "root",
        key_type: KeyType::Bool,
        description: "Run the container as root (true/false).",
    },
    SchemaItem {
        key: "unshare_ipc",
        key_type: KeyType::Bool,
        description: "Do not share IPC namespace with the host (true/false).",
    },
    SchemaItem {
        key: "unshare_netns",
        key_type: KeyType::Bool,
        description: "Do not share network namespace with the host (true/false).",
    },
    SchemaItem {
        key: "unshare_process",
        key_type: KeyType::Bool,
        description: "Do not share process namespace with the host (true/false).",
    },
    SchemaItem {
        key: "unshare_devsys",
        key_type: KeyType::Bool,
        description: "Do not share /dev/sysfs with the host (true/false).",
    },
    SchemaItem {
        key: "unshare_all",
        key_type: KeyType::Bool,
        description: "Unshare all possible namespaces (true/false).",
    },
];

// --- NEW: Constant for the custom path option ---
const CUSTOM_HOME_PATH_OPTION: &str = "[ Type Custom Path ]";

struct Schema {
    map: HashMap<&'static str, KeyType>,
}
impl Schema {
    fn new() -> Self {
        let map = SCHEMA
            .iter()
            .map(|item| (item.key, item.key_type))
            .collect();
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
    ConfirmDelete,
    AddingKey(ListState),
    AddingStringValue,
    AddingBoolValue,
    SelectingFromList(ListState),
    CustomListInput,
    Saved,
}

#[derive(Clone)]
pub(crate) enum DisplayItem {
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
    list_selector_options: Vec<(String, String)>, // (value_to_display, value_to_save)
    current_list_selector: Option<SelectorType>,
    tui_mode: TuiMode,
}

impl<'a> App<'a> {
    fn new(
        file_path: &Path,
        initial_items: Vec<DisplayItem>,
        initial_section: String,
        mode: TuiMode,
    ) -> App<'a> {
        let mut state = ListState::default();
        if !initial_items.is_empty() {
            state.select(Some(0));
        }

        let mut app = App {
            file_path: file_path.to_path_buf(),
            items: initial_items,
            list_items: Vec::new(),
            state,
            mode: AppMode::Navigating,
            value_input: Input::default(),
            current_section: initial_section,
            current_add_item: None,
            current_bool_value: true,
            status_timer: 0,
            schema: Schema::new(),
            list_selector_options: Vec::new(),
            current_list_selector: None,
            tui_mode: mode,
        };
        app.refresh_list_items_from_items();
        app
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
                        self.load_list_options(SelectorType::Home);
                        let mut list_state = ListState::default();
                        let current_idx = self
                            .list_selector_options
                            .iter()
                            .position(|(_, save_val)| *save_val == value_clone);
                        list_state.select(current_idx.or(Some(0)));
                        self.mode = AppMode::SelectingFromList(list_state);
                    }
                    KeyType::Image => {
                        self.load_list_options(SelectorType::Image);
                        let mut list_state = ListState::default();
                        let current_idx = self
                            .list_selector_options
                            .iter()
                            .position(|(_, save_val)| *save_val == value_clone);
                        list_state.select(current_idx.or(Some(0)));
                        self.mode = AppMode::SelectingFromList(list_state);
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
        self.list_selector_options.clear();
        self.current_list_selector = None;
    }

    // --- NEW: Helper function to apply the edit ---
    fn set_edited_value(&mut self, new_value: String) {
        if let Some(index) = self.state.selected() {
            if let DisplayItem::Property(section, key, _) = &self.items[index].clone() {
                self.items[index] =
                    DisplayItem::Property(section.clone(), key.clone(), new_value.clone());
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

    // In impl App<'a>

    fn submit_key(&mut self) {
        // First, get the selected index without holding the borrow on self.mode
        let selected_index: Option<usize> = if let AppMode::AddingKey(list_state) = &mut self.mode {
            list_state.selected()
        } else {
            None
        };

        // Now, use the index to modify self.mode
        if let Some(index) = selected_index {
            let schema_item = SCHEMA[index].clone();
            self.current_add_item = Some(schema_item.clone());

            match schema_item.key_type {
                KeyType::String | KeyType::StringList => {
                    self.value_input = Input::default();
                    self.mode = AppMode::AddingStringValue;
                }
                KeyType::HomeDir => {
                    self.load_list_options(SelectorType::Home);
                    let mut list_state = ListState::default();
                    list_state.select(Some(0));
                    self.mode = AppMode::SelectingFromList(list_state);
                }
                KeyType::Image => {
                    self.load_list_options(SelectorType::Image);
                    let mut list_state = ListState::default();
                    list_state.select(Some(0));
                    self.mode = AppMode::SelectingFromList(list_state);
                }
                KeyType::Bool => {
                    self.current_bool_value = true;
                    self.mode = AppMode::AddingBoolValue;
                }
            }
        }
    }

    // Rename submit_value
    fn submit_string_value(&mut self) {
        if let Some(item) = &self.current_add_item {
            let new_key = item.key.to_string();
            // Only handle String, StringList here now
            let new_value = self.value_input.value().to_string();

            // Insert logic remains the same...
            let index = self.state.selected().unwrap_or(0);
            let insert_index = if self.items.is_empty() { 0 } else { index + 1 };
            let new_display_item = DisplayItem::Property(
                self.current_section.clone(),
                new_key.clone(),
                new_value.clone(),
            );
            self.items.insert(insert_index, new_display_item);
            self.refresh_list_items_from_items();
            self.state.select(Some(insert_index));
            self.cancel_adding();
        }
    }

    // New function for submitting boolean values
    fn submit_bool_value(&mut self) {
        if let Some(item) = &self.current_add_item {
            let new_key = item.key.to_string();
            let new_value = self.current_bool_value.to_string();

            // Insert logic remains the same...
            let index = self.state.selected().unwrap_or(0);
            let insert_index = if self.items.is_empty() { 0 } else { index + 1 };
            let new_display_item = DisplayItem::Property(
                self.current_section.clone(),
                new_key.clone(),
                new_value.clone(),
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
        self.list_selector_options.clear();
        self.current_list_selector = None;
    }

    fn toggle_bool_value(&mut self) {
        self.current_bool_value = !self.current_bool_value;
    }

    fn load_list_options(&mut self, selector_type: SelectorType) {
        let mut options: Vec<(String, String)> = Vec::new();
        let custom_option_label: &str;

        match selector_type {
            SelectorType::Home => {
                custom_option_label = CUSTOM_HOME_PATH_OPTION;
                // --- Logic from load_home_dir_options ---
                options.push(("host".to_string(), "host".to_string()));
                options.push(("none".to_string(), "none".to_string()));
                if let Ok(homes_path) = dirs::home_dir()
                    .ok_or_else(|| anyhow!("Could not find home directory"))
                    .map(|mut p| {
                        p.push(".distromanifesto/homes");
                        p
                    })
                {
                    if let Ok(entries) = fs::read_dir(homes_path) {
                        for entry in entries.flatten() {
                            if entry.path().is_dir() {
                                if let Some(dir_name) = entry.path().file_name() {
                                    let path_str = format!(
                                        "~/.distromanifesto/homes/{}",
                                        dir_name.to_string_lossy()
                                    );
                                    options.push((path_str.clone(), path_str));
                                }
                            }
                        }
                    }
                }
                // --- End logic ---
            }
            SelectorType::Image => {
                custom_option_label = CUSTOM_OCI_IMAGE_OPTION;
                // --- Logic from load_image_options ---
                options = DISTROBOX_IMAGES
                    .iter()
                    .map(|(short, full)| (format!("{short}   ({full})"), short.to_string())) // Display both, save short
                    .collect();
                // --- End logic ---
            }
        }

        // Add the custom option
        options.push((
            custom_option_label.to_string(),
            custom_option_label.to_string(),
        ));

        self.list_selector_options = options;
        self.current_list_selector = Some(selector_type);
    }

    // Helper to insert the new item
    fn insert_new_property(&mut self, key: String, value: String) {
        let index = self.state.selected().unwrap_or(0);
        let insert_index = if self.items.is_empty() { 0 } else { index + 1 };

        let new_display_item =
            DisplayItem::Property(self.current_section.clone(), key.clone(), value.clone());

        self.items.insert(insert_index, new_display_item);
        self.refresh_list_items_from_items();
        self.state.select(Some(insert_index));
        self.cancel_adding();
    }

    fn submit_list_select(&mut self) {
        if let AppMode::SelectingFromList(list_state) = &self.mode {
            if let Some(selected_index) = list_state.selected() {
                let (display_val, save_val) = self.list_selector_options[selected_index].clone();

                let is_custom = match self.current_list_selector {
                    Some(SelectorType::Home) => display_val == CUSTOM_HOME_PATH_OPTION,
                    Some(SelectorType::Image) => display_val == CUSTOM_OCI_IMAGE_OPTION,
                    None => false,
                };

                if is_custom {
                    self.value_input = Input::default();
                    self.mode = AppMode::CustomListInput;
                } else {
                    // Not custom, so we save the value
                    if self.tui_mode == TuiMode::Modify {
                        self.set_edited_value(save_val);
                    } else {
                        if let Some(item) = &self.current_add_item {
                            self.insert_new_property(item.key.to_string(), save_val);
                        } else {
                            self.cancel_adding();
                        }
                    }
                }
            }
        }
    }

    fn submit_list_custom(&mut self) {
        let new_value = self.value_input.value().to_string();
        if !new_value.is_empty() {
            if self.tui_mode == TuiMode::Modify {
                self.set_edited_value(new_value);
            } else {
                if let Some(item) = &self.current_add_item {
                    self.insert_new_property(item.key.to_string(), new_value);
                } else {
                    self.cancel_adding();
                }
            }
        } else {
            if self.tui_mode == TuiMode::Modify {
                self.cancel_editing();
            } else {
                self.cancel_adding();
            }
        }
    }

    fn has_required_keys(&self) -> bool {
        self.items
            .iter()
            .any(|item| matches!(item, DisplayItem::Property(_, key, _) if key == "image"))
    }
}

// --- Main TUI Functions ---
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TuiMode {
    Create,
    Modify,
}

pub(crate) fn run_tui(
    file_path: &Path,
    initial_items: Vec<DisplayItem>,
    initial_section: String,
    mode: TuiMode,
) -> Result<()> {
    let mut app = App::new(file_path, initial_items, initial_section, mode);

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
                    AppMode::Navigating => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('s') => match app.tui_mode {
                            TuiMode::Create => {
                                if app.has_required_keys() {
                                    app.save_to_file().unwrap_or_else(|_| {});
                                }
                            }
                            TuiMode::Modify => {
                                app.save_to_file().unwrap_or_else(|_| {});
                            }
                        },
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous(),
                        KeyCode::Enter => app.start_editing(),
                        KeyCode::Char('d') => app.start_confirm_delete(),
                        KeyCode::Char('a') => app.start_adding(),
                        _ => {}
                    },
                    AppMode::EditingString => match key.code {
                        KeyCode::Enter => app.submit_string_editing(),
                        KeyCode::Esc => app.cancel_editing(),
                        _ => {
                            app.value_input.handle_event(&Event::Key(key));
                        }
                    },
                    AppMode::EditingBool => match key.code {
                        KeyCode::Enter => app.submit_bool_editing(),
                        KeyCode::Esc => app.cancel_editing(),
                        KeyCode::Char(' ') | KeyCode::Tab | KeyCode::Left | KeyCode::Right => {
                            app.toggle_bool_value();
                        }
                        _ => {}
                    },
                    AppMode::ConfirmDelete => match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => app.delete_selected(),
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            app.cancel_delete()
                        }
                        _ => {}
                    },
                    AppMode::AddingKey(ref mut list_state) => match key.code {
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
                    },
                    AppMode::AddingStringValue => {
                        if let Some(item) = &app.current_add_item {
                            // Only handle String and StringList here
                            if matches!(item.key_type, KeyType::String | KeyType::StringList) {
                                match key.code {
                                    KeyCode::Enter => app.submit_string_value(), // Use renamed function
                                    KeyCode::Esc => app.cancel_adding(),
                                    _ => {
                                        app.value_input.handle_event(&Event::Key(key));
                                    }
                                }
                            } else {
                                // Should not happen in this state, but cancel just in case
                                app.cancel_adding();
                            }
                        } else {
                            app.cancel_adding();
                        }
                    }
                    AppMode::AddingBoolValue => {
                        if let Some(item) = &app.current_add_item {
                            if item.key_type == KeyType::Bool {
                                match key.code {
                                    KeyCode::Enter => app.submit_bool_value(), // Use new bool submit function
                                    KeyCode::Esc => app.cancel_adding(),
                                    KeyCode::Char(' ')
                                    | KeyCode::Tab
                                    | KeyCode::Left
                                    | KeyCode::Right => {
                                        app.toggle_bool_value();
                                    }
                                    _ => {}
                                }
                            } else {
                                app.cancel_adding();
                            }
                        } else {
                            app.cancel_adding();
                        }
                    }
                    AppMode::SelectingFromList(ref mut list_state) => {
                        match key.code {
                            KeyCode::Enter => app.submit_list_select(), // <-- Use generic
                            KeyCode::Esc => {
                                if app.tui_mode == TuiMode::Modify {
                                    app.cancel_editing();
                                } else {
                                    app.cancel_adding();
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                let i = list_state.selected().unwrap_or(0);
                                let next = if i >= app.list_selector_options.len() - 1 {
                                    0
                                } else {
                                    i + 1
                                };
                                list_state.select(Some(next));
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                let i = list_state.selected().unwrap_or(0);
                                let prev = if i == 0 {
                                    app.list_selector_options.len() - 1
                                } else {
                                    i - 1
                                };
                                list_state.select(Some(prev));
                            }
                            _ => {}
                        }
                    }
                    AppMode::CustomListInput => {
                        match key.code {
                            KeyCode::Enter => app.submit_list_custom(), // <-- Use generic
                            KeyCode::Esc => {
                                if app.tui_mode == TuiMode::Modify {
                                    app.cancel_editing();
                                } else {
                                    app.cancel_adding();
                                }
                            }
                            _ => {
                                app.value_input.handle_event(&Event::Key(key));
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
        AppMode::Navigating => match app.tui_mode {
            TuiMode::Modify => (
                " (q) Quit | (s) Save | (↑/↓) Nav | (a) Add | (d) Delete | (Enter) Edit "
                    .to_string(),
                Style::default(),
            ),
            TuiMode::Create => {
                let has_image = app.has_required_keys();
                let save_text = if has_image { " (s) Save |" } else { "" };
                let footer_text =
                    format!(" (q) Quit |{save_text} (a) Add Key | (d) Delete Key | (Enter) Edit ");
                let footer_style = if !has_image {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                };
                let title_text = if !has_image {
                    format!("{footer_text} [WARNING: 'image' key is required]")
                } else {
                    footer_text
                };
                (title_text, footer_style)
            }
        },
        AppMode::EditingString => (
            " (Enter) Accept | (Esc) Cancel ".to_string(),
            Style::default(),
        ),
        AppMode::EditingBool => (
            " (Space/←/→) Toggle | (Enter) Accept | (Esc) Cancel ".to_string(),
            Style::default(),
        ),
        AppMode::ConfirmDelete => (
            " Delete selected item? (y/n) ".to_string(),
            Style::default().fg(Color::Red),
        ),
        AppMode::AddingKey(_) => (
            " (↑/↓) Select | (Enter) Next | (Esc) Cancel ".to_string(),
            Style::default().fg(Color::Cyan), // Keep the title cyan
        ),
        AppMode::AddingStringValue => (
            " (Enter) Accept | (Esc) Cancel ".to_string(),
            Style::default(),
        ),
        AppMode::AddingBoolValue => (
            " (Space/←/→) Toggle | (Enter) Accept | (Esc) Cancel ".to_string(),
            Style::default(),
        ),
        AppMode::SelectingFromList(_) => (
            " (↑/↓) Select | (Enter) Accept | (Esc) Cancel ".to_string(),
            Style::default(),
        ),
        AppMode::CustomListInput => (
            " (Enter) Accept Custom Value | (Esc) Cancel ".to_string(),
            Style::default(),
        ),
        AppMode::Saved => (
            " File saved successfully! ".to_string(),
            Style::default().fg(Color::Green),
        ),
    };

    let footer_block = Block::default()
        .borders(Borders::ALL)
        .title(footer_text)
        .title_style(footer_style)
        .border_style(footer_style);
    f.render_widget(footer_block, chunks[2]);
    if let AppMode::AddingKey(list_state) = &app.mode {
        // Create a temporary block *identic* to the footer_block
        // just to calculate its inner area.
        let block_for_inner = Block::default().borders(Borders::ALL);
        let inner_footer_area = block_for_inner.inner(chunks[2]);

        let description = list_state.selected().map_or("".to_string(), |index| {
            SCHEMA
                .get(index)
                .map_or("".to_string(), |item| item.description.to_string())
        });
        let text = format!("[Hint: {description}]");

        let hint_para = Paragraph::new(text)
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }); // Wrap in case description is too long

        // Render the hint inside the footer area
        f.render_widget(hint_para, inner_footer_area);
    }

    let list = List::new(app.list_items.clone())
        .block(Block::default().title("Manifest").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[1], &mut app.state);

    match &mut app.mode {
        AppMode::EditingString => draw_edit_string_popup(f, app),
        AppMode::EditingBool => draw_edit_bool_popup(f, app),
        AppMode::ConfirmDelete => draw_delete_popup(f),
        AppMode::AddingKey(list_state) => draw_add_key_popup(f, list_state),
        AppMode::AddingStringValue => draw_add_string_value_popup(f, app), // Use specific draw function
        AppMode::AddingBoolValue => draw_add_bool_value_popup(f, app), // Use specific draw function
        AppMode::SelectingFromList(list_state) => {
            draw_list_select_popup(
                f,
                app.tui_mode,
                &app.current_list_selector,
                &app.list_selector_options,
                list_state,
            );
        }
        AppMode::CustomListInput => {
            draw_list_custom_popup(
                f,
                app.tui_mode,
                &app.current_list_selector,
                &mut app.value_input,
            );
        }
        AppMode::Navigating | AppMode::Saved => {}
    }
}

fn draw_edit_string_popup<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let area = centered_rect(60, 20, f.size());
    f.render_widget(Clear, area);

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
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::REVERSED)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let false_style = if !value {
        Style::default()
            .fg(Color::Red)
            .add_modifier(Modifier::REVERSED)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let true_text = Paragraph::new(" true ")
        .style(true_style)
        .alignment(Alignment::Center);
    let false_text = Paragraph::new(" false ")
        .style(false_style)
        .alignment(Alignment::Center);

    f.render_widget(true_text, layout[0]);
    f.render_widget(false_text, layout[1]);
}

fn draw_edit_bool_popup<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let area = centered_rect(40, 20, f.size());
    let title = if let Some(index) = app.state.selected() {
        if let DisplayItem::Property(_, key, _) = &app.items[index] {
            format!(" Edit Value for: {key} ")
        } else {
            " Edit Value ".to_string()
        }
    } else {
        " Edit Value ".to_string()
    };

    draw_bool_toggle(f, area, title, app.current_bool_value);
}

fn draw_list_select_popup<B: Backend>(
    f: &mut Frame<B>,
    app_tui_mode: TuiMode,                         // <-- ADDED
    current_list_selector: &Option<SelectorType>,  // <-- ADDED
    list_selector_options: &Vec<(String, String)>, // <-- ADDED
    list_state: &mut ListState,
) {
    let (title, custom_option_label) = match *current_list_selector {
        Some(SelectorType::Home) => (" Home ", CUSTOM_HOME_PATH_OPTION),
        Some(SelectorType::Image) => (" Image ", CUSTOM_OCI_IMAGE_OPTION),
        None => (" Select ", ""), // Should not happen
    };

    let title = if app_tui_mode == TuiMode::Modify {
        format!(" Edit Value:{title}")
    } else {
        format!(" Add Value:{title}")
    };

    let area = centered_rect(80, 80, f.size());
    f.render_widget(Clear, area);

    let items: Vec<ListItem> = list_selector_options
        .iter()
        .map(|(display_val, _)| {
            if display_val == custom_option_label {
                ListItem::new(display_val.as_str()).style(Style::default().fg(Color::Yellow))
            } else {
                ListItem::new(display_val.as_str())
            }
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, list_state);
}

fn draw_list_custom_popup<B: Backend>(
    f: &mut Frame<B>,
    app_tui_mode: TuiMode,                        // <-- ADDED
    current_list_selector: &Option<SelectorType>, // <-- ADDED
    value_input: &mut Input,                      // <-- ADDED
) {
    let (title_noun, key_name) = match *current_list_selector {
        Some(SelectorType::Home) => ("Custom Path", "home"),
        Some(SelectorType::Image) => ("Custom Image", "image"),
        None => ("Custom Value", ""),
    };

    let title = if app_tui_mode == TuiMode::Modify {
        format!(" Edit {title_noun} for: {key_name} ")
    } else {
        format!(" Add {title_noun} for: {key_name} ")
    };

    let area = centered_rect(60, 20, f.size());
    f.render_widget(Clear, area);

    let width = area.width.max(3) - 3;
    let scroll = value_input.visual_scroll(width as usize);

    let input_widget = Paragraph::new(value_input.value())
        .style(Style::default().fg(Color::Yellow))
        .scroll((0, scroll as u16))
        .block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(input_widget, area);

    f.set_cursor(
        area.x + 1 + (value_input.visual_cursor().max(scroll) - scroll) as u16,
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
                .border_style(Style::default().fg(Color::Red)),
        );
    f.render_widget(text, area);
}

fn draw_add_key_popup<B: Backend>(f: &mut Frame<B>, list_state: &mut ListState) {
    let area = centered_rect(50, 80, f.size());
    f.render_widget(Clear, area);

    let items: Vec<ListItem> = SCHEMA.iter().map(|item| ListItem::new(item.key)).collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Select a Key to Add"),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, list_state);
}

fn draw_add_string_value_popup<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let area = centered_rect(60, 25, f.size());
    let item = app.current_add_item.as_ref().unwrap(); // Assume item exists
    let title = format!(" Enter Value for: {} ", item.key);

    // This part remains the same as the original AddingValue popup for strings
    let width = area.width.max(3) - 3;
    let scroll = app.value_input.visual_scroll(width as usize);

    let input = Paragraph::new(app.value_input.value())
        .style(Style::default().fg(Color::Yellow))
        .scroll((0, scroll as u16))
        .block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(Clear, area); // Clear area before drawing
    f.render_widget(input, area);

    f.set_cursor(
        area.x + 1 + (app.value_input.visual_cursor().max(scroll) - scroll) as u16,
        area.y + 1,
    )
}

// New draw function for adding boolean values
fn draw_add_bool_value_popup<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let area = centered_rect(40, 20, f.size());
    let item = app.current_add_item.as_ref().unwrap(); // Assume item exists
    let title = format!(" Set Value for: {} ", item.key);

    // Reuse the draw_bool_toggle helper function
    draw_bool_toggle(f, area, title, app.current_bool_value);
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
