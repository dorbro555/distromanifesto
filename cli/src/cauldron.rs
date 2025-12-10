// In cli/src/cauldron.rs

use anyhow::{Context, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    // --- We need more style/text modules now ---
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
    // ---
    Frame,
    Terminal,
};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use crate::modify;
use crate::setup;

// --- NEW: Enum for active (focused) pane ---
#[derive(PartialEq)]
enum FocusedPane {
    Containers,
    Manifests,
    Homes,
    Content, // For the right-hand side
}

// --- Helper function to load file/dir names from a ~/.distromanifesto subdirectory ---
// Moved *before* App to be in scope
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

// --- Helper to run `distrobox list` and parse output ---
// Moved *before* App to be in scope
fn load_distrobox_list() -> Result<Vec<String>> {
    let output = Command::new("distrobox")
        .args(["list", "--no-color"])
        .output()
        .context("Failed to execute 'distrobox list'")?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("distrobox list failed: {}", error_msg));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut containers = Vec::new();

    for line in stdout.lines().skip(1) {
        if let Some(name) = line.split('|').nth(1) {
            let trimmed_name = name.trim();
            if !trimmed_name.is_empty() {
                containers.push(trimmed_name.to_string());
            }
        }
    }

    Ok(containers)
}

// --- App State ---
struct App {
    focused_pane: FocusedPane, // <-- Which pane is active

    // Data lists
    manifests: Vec<String>,
    homes: Vec<String>,
    containers: Vec<String>,

    // TUI list states
    container_state: ListState,
    manifest_state: ListState,
    home_state: ListState,

    // TODO: State for right-hand tabs
    content_tab_index: usize,

    active_content: String,

    last_refresh: Instant,
    should_quit: bool,
}

impl App {
    // Constructor that loads initial data
    fn new() -> Result<Self> {
        let manifests = load_directory_contents("manifests").context("Failed to load manifests")?;
        let homes = load_directory_contents("homes").context("Failed to load homes")?;
        let containers = load_distrobox_list().context("Failed to load distrobox list")?;

        // --- List state initializations ---
        let mut container_state = ListState::default();
        if !containers.is_empty() {
            container_state.select(Some(0));
        }

        let mut manifest_state = ListState::default();
        if !manifests.is_empty() {
            manifest_state.select(Some(0));
        }

        let mut home_state = ListState::default();
        if !homes.is_empty() {
            home_state.select(Some(0));
        }
        // ---

        // --- Create the app instance ---
        let mut app = App {
            focused_pane: FocusedPane::Containers, // Default focus
            manifests,
            homes,
            containers,
            container_state,
            manifest_state,
            home_state,
            content_tab_index: 0,
            active_content: String::new(), // <-- This is the missing field
            last_refresh: Instant::now(),
            should_quit: false,
        };

        // --- Load initial content for the right pane ---
        app.update_active_content();

        // --- Return the initialized app ---
        Ok(app)
    }

    // Refresh data method (unchanged)
    fn refresh_data(&mut self) -> Result<()> {
        self.manifests =
            load_directory_contents("manifests").context("Failed to load manifests")?;
        self.homes = load_directory_contents("homes").context("Failed to load homes")?;
        self.containers = load_distrobox_list().context("Failed to load distrobox list")?;
        self.last_refresh = Instant::now();
        Ok(())
    }

    fn update_active_content(&mut self) {
        let content = match self.focused_pane {
            FocusedPane::Containers => self.container_state.selected().map_or_else(
                || "No container selected.".to_string(),
                |i| {
                    let container_name = &self.containers[i];
                    // TODO: Run `distrobox list --no-color` again and find the full line
                    format!("Details for container: {}", container_name)
                },
            ),
            FocusedPane::Manifests => self.manifest_state.selected().map_or_else(
                || "No manifest selected.".to_string(),
                |i| {
                    let manifest_name = &self.manifests[i];
                    let path_str = format!("~/.distromanifesto/manifests/{}", manifest_name);
                    match setup::get_full_path_from_str(&path_str) {
                        Ok(path) => match fs::read_to_string(path) {
                            Ok(content) => content,
                            Err(e) => format!("Error reading manifest:\n{}", e),
                        },
                        Err(e) => format!("Error getting path:\n{}", e),
                    }
                },
            ),
            FocusedPane::Homes => self.home_state.selected().map_or_else(
                || "No home selected.".to_string(),
                |i| {
                    let home_name = &self.homes[i];
                    let path_str = format!("~/.distromanifesto/homes/{}", home_name);
                    // TODO: Run `du -sh` on this path
                    format!("Details for home: {}", path_str)
                },
            ),
            FocusedPane::Content => {
                // Content pane is focused, but content is driven by left panes.
                // We'll just update based on the *last* focused list.
                // This logic will get smarter later.
                self.active_content.clone() // For now, just keep what's there
            }
        };
        self.active_content = content;
    }

    // --- NEW: List navigation based on focused pane ---
    fn list_next(&mut self) {
        let (state, len) = match self.focused_pane {
            FocusedPane::Containers => (&mut self.container_state, self.containers.len()),
            FocusedPane::Manifests => (&mut self.manifest_state, self.manifests.len()),
            FocusedPane::Homes => (&mut self.home_state, self.homes.len()),
            FocusedPane::Content => return,
        };

        if len == 0 {
            return;
        }
        let i = match state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        state.select(Some(i));
        self.update_active_content(); // <-- ADD THIS
    }

    fn list_previous(&mut self) {
        let (state, len) = match self.focused_pane {
            FocusedPane::Containers => (&mut self.container_state, self.containers.len()),
            FocusedPane::Manifests => (&mut self.manifest_state, self.manifests.len()),
            FocusedPane::Homes => (&mut self.home_state, self.homes.len()),
            FocusedPane::Content => return,
        };

        if len == 0 {
            return;
        }
        let i = match state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        state.select(Some(i));
        self.update_active_content(); // <-- ADD THIS
    }

    // --- NEW: Focus cycling ---
    fn cycle_focus(&mut self) {
        self.focused_pane = match self.focused_pane {
            FocusedPane::Containers => FocusedPane::Manifests,
            FocusedPane::Manifests => FocusedPane::Homes,
            FocusedPane::Homes => FocusedPane::Content,
            FocusedPane::Content => FocusedPane::Containers,
        };
        self.update_active_content(); // <-- ADD THIS
    }

    /// Stops the currently selected container
    fn action_stop_container(&mut self) -> Result<()> {
        if let Some(index) = self.container_state.selected() {
            let name = &self.containers[index];
            // run: distrobox stop --yes <name>
            let output = Command::new("distrobox")
                .args(["stop", "--yes", name])
                .output()
                .context("Failed to stop container")?;

            if output.status.success() {
                self.refresh_data()?; // Refresh list on success
            } else {
                // For now, we'll just put the error in the content view so the user sees it
                let err = String::from_utf8_lossy(&output.stderr);
                self.active_content = format!("Error stopping container:\n\n{}", err);
                self.focused_pane = FocusedPane::Content; // Switch focus so they see the error
                self.content_tab_index = 0; // Show Info tab
            }
        }
        Ok(())
    }

    /// Deletes the currently selected container
    fn action_delete_container(&mut self) -> Result<()> {
        if let Some(index) = self.container_state.selected() {
            let name = &self.containers[index];
            // run: distrobox rm --force <name>
            let output = Command::new("distrobox")
                .args(["rm", "--force", name])
                .output()
                .context("Failed to delete container")?;

            if output.status.success() {
                self.refresh_data()?; // Refresh list on success
                                      // Reset selection if out of bounds
                if index >= self.containers.len() && !self.containers.is_empty() {
                    self.container_state.select(Some(self.containers.len() - 1));
                }
            } else {
                let err = String::from_utf8_lossy(&output.stderr);
                self.active_content = format!("Error deleting container:\n\n{}", err);
                self.focused_pane = FocusedPane::Content;
                self.content_tab_index = 0;
            }
        }
        Ok(())
    }

    fn action_delete_manifest(&mut self) -> Result<()> {
        if let Some(index) = self.manifest_state.selected() {
            let name = &self.manifests[index];
            let path_str = format!("~/.distromanifesto/manifests/{}", name);

            // Resolve path
            let path = setup::get_full_path_from_str(&path_str)?;

            // Delete file
            if path.exists() {
                fs::remove_file(&path).context("Failed to delete manifest file")?;
            }

            self.refresh_data()?; // Reload list

            // Fix selection if it went out of bounds
            if index >= self.manifests.len() && !self.manifests.is_empty() {
                self.manifest_state.select(Some(self.manifests.len() - 1));
            }
        }
        Ok(())
    }

    /// Launches the existing TUI editor for the selected manifest
    fn action_modify_manifest(&mut self) -> Result<()> {
        if let Some(index) = self.manifest_state.selected() {
            let name = &self.manifests[index];
            let path_str = format!("~/.distromanifesto/manifests/{}", name);
            let path = setup::get_full_path_from_str(&path_str)?;

            // Call the external editor module
            // This will take over the terminal!
            modify::launch_editor(&path)?;

            // When we return here, we need to refresh because the file might have changed
            self.refresh_data()?;
        }
        Ok(())
    }
}

// --- Main TUI Function (Restored) ---
pub fn launch_tui() -> Result<()> {
    setup::ensure_hidden_dir().context("Failed to ensure .distromanifesto directories")?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new()?;
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
fn run_app<B: Backend + std::io::Write>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    let refresh_rate = Duration::from_secs(5);

    loop {
        // 1. DRAW
        terminal.draw(|f| ui(f, &mut app))?;

        // 2. TIMEOUT
        let timeout = refresh_rate
            .checked_sub(app.last_refresh.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        // 3. POLL
        if event::poll(timeout)? {
            // 4. READ
            if let Event::Key(key) = event::read()? {
                // 5. FILTER: Only handle PRESS events
                if key.kind == event::KeyEventKind::Press {
                    match key.code {
                        // --- Global Navigation & System Keys ---
                        KeyCode::Tab => app.cycle_focus(),
                        KeyCode::Char('q') => app.should_quit = true,
                        KeyCode::Char('r') => app.refresh_data()?,

                        // --- Global Tab Switching (Now works everywhere) ---
                        KeyCode::Char('l') | KeyCode::Right => {
                            app.content_tab_index = (app.content_tab_index + 1) % 2;
                        }
                        KeyCode::Char('h') | KeyCode::Left => {
                            app.content_tab_index = if app.content_tab_index == 0 { 1 } else { 0 };
                        }

                        // --- List Navigation (Safe: ignores input if Content pane is focused) ---
                        KeyCode::Down | KeyCode::Char('j') => app.list_next(),
                        KeyCode::Up | KeyCode::Char('k') => app.list_previous(),

                        // --- Context-Specific Actions ---
                        KeyCode::Char('s') => {
                            if app.focused_pane == FocusedPane::Containers {
                                app.action_stop_container()?;
                            }
                        }
                        KeyCode::Char('d') => {
                            if app.focused_pane == FocusedPane::Containers {
                                app.action_delete_container()?;
                            } else if app.focused_pane == FocusedPane::Manifests {
                                // --- NEW: Delete Manifest ---
                                app.action_delete_manifest()?;
                            } else if app.focused_pane == FocusedPane::Homes {
                                // Placeholder for delete home
                            }
                            // Add other 'd' cases for Manifests/Homes here later
                        }
                        KeyCode::Char('m') => {
                            if app.focused_pane == FocusedPane::Manifests {
                                // --- NEW: Modify Manifest ---

                                // 1. Run the editor
                                app.action_modify_manifest()?;

                                // 2. CRITICAL: RESTORE TERMINAL STATE
                                // Since modify::launch_editor() tore it down, we must rebuild it.
                                enable_raw_mode()?;
                                execute!(
                                    terminal.backend_mut(),
                                    EnterAlternateScreen,
                                    EnableMouseCapture
                                )?;
                                terminal.hide_cursor()?;
                                terminal.clear()?; // Force a full redraw
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // 6. AUTO-REFRESH
        if app.last_refresh.elapsed() >= refresh_rate {
            app.refresh_data()?;
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

// --- UI Drawing ---
fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // --- NEW: Main layout with footer ---
    let main_chunks_with_footer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Main app area
            Constraint::Length(1), // Footer area
        ])
        .split(f.size());

    let app_area = main_chunks_with_footer[0]; // <-- All other layout happens in here
    let footer_area = main_chunks_with_footer[1];

    // --- Define styles ---
    let focused_style = Style::default().fg(Color::Yellow);
    let default_style = Style::default().fg(Color::Gray);
    let focused_list_style = Style::default()
        .fg(Color::Black)
        .bg(Color::LightGreen)
        .add_modifier(Modifier::BOLD);

    // --- Main two-pane layout (now uses app_area) ---
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(33), Constraint::Percentage(67)].as_ref())
        .split(app_area); // <-- Use app_area, not f.size()

    let left_pane = main_chunks[0];
    let right_pane = main_chunks[1];

    // --- Build Left Pane (Stacked Boxes) ---
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ]
            .as_ref(),
        )
        .split(left_pane);

    // --- Render Containers List ---
    let container_style = if app.focused_pane == FocusedPane::Containers {
        focused_style
    } else {
        default_style
    };
    let container_items: Vec<ListItem> = app
        .containers
        .iter()
        .map(|c| ListItem::new(c.as_str()))
        .collect();
    let container_list = List::new(container_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Containers")
                .border_style(container_style),
        )
        .highlight_style(if app.focused_pane == FocusedPane::Containers {
            focused_list_style
        } else {
            Style::default()
        });
    f.render_stateful_widget(container_list, left_chunks[0], &mut app.container_state);

    // --- Render Manifests List ---
    let manifest_style = if app.focused_pane == FocusedPane::Manifests {
        focused_style
    } else {
        default_style
    };
    let manifest_items: Vec<ListItem> = app
        .manifests
        .iter()
        .map(|m| ListItem::new(m.as_str()))
        .collect();
    let manifest_list = List::new(manifest_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Manifests")
                .border_style(manifest_style),
        )
        .highlight_style(if app.focused_pane == FocusedPane::Manifests {
            focused_list_style
        } else {
            Style::default()
        });
    f.render_stateful_widget(manifest_list, left_chunks[1], &mut app.manifest_state);

    // --- Render Homes List ---
    let home_style = if app.focused_pane == FocusedPane::Homes {
        focused_style
    } else {
        default_style
    };
    let home_items: Vec<ListItem> = app
        .homes
        .iter()
        .map(|h| ListItem::new(h.as_str()))
        .collect();
    let home_list = List::new(home_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Managed Homes")
                .border_style(home_style),
        )
        .highlight_style(if app.focused_pane == FocusedPane::Homes {
            focused_list_style
        } else {
            Style::default()
        });
    f.render_stateful_widget(home_list, left_chunks[2], &mut app.home_state);

    // --- Render Right Pane (with inline tabs) ---
    let content_border_style = if app.focused_pane == FocusedPane::Content {
        focused_style
    } else {
        default_style
    };
    let content_block = Block::default()
        .borders(Borders::ALL)
        .border_style(content_border_style);
    let inner_area = content_block.inner(right_pane);
    f.render_widget(content_block, right_pane);

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // For tabs
            Constraint::Min(0),    // For content
        ])
        .split(inner_area);

    let tab_area = inner_chunks[0];
    let main_content_area = inner_chunks[1];

    // --- Render "Inline" Tabs ---
    let tab_titles = vec![
        Line::from(Span::styled(" [ Info ] ", Style::default())),
        Line::from(Span::styled(" [ Actions ] ", Style::default())),
    ];
    let tabs = Tabs::new(tab_titles)
        .select(app.content_tab_index)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, tab_area);

    // --- MODIFIED: Render Content (help text removed) ---
    let content_widget = match app.content_tab_index {
        0 => {
            // --- Info Tab ---
            Paragraph::new(app.active_content.clone()).block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(default_style),
            )
        }
        1 => {
            // --- NEW: Dynamic Actions Tab ---
            let actions_text = match app.focused_pane {
                FocusedPane::Containers => vec![
                    "Available Actions for Container:",
                    "",
                    "  (s) Stop      - Stop the container (distrobox stop --yes)",
                    "  (d) Delete    - Remove the container (distrobox rm --force)",
                    // "  (e) Enter     - Enter shell (coming soon)",
                ],
                FocusedPane::Manifests => vec![
                    "Available Actions for Manifest:",
                    "",
                    "  (d) Delete    - Delete this manifest file",
                    "  (m) Modify    - Open in Editor",
                    "  (c) Create    - Create container from this manifest (coming soon)",
                ],
                FocusedPane::Homes => vec![
                    "Available Actions for Home:",
                    "",
                    "  (d) Delete    - Delete this home directory (careful!)",
                ],
                FocusedPane::Content => vec!["Select a list on the left to see actions."],
            };

            let text_joined = actions_text.join("\n");
            Paragraph::new(text_joined).block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(default_style),
            )
        }
        _ => unreachable!(),
    };

    f.render_widget(content_widget, main_content_area);

    // --- NEW: Render Fixed Footer ---
    let help_text =
        "(q) Quit | (r) Refresh | (Tab) Switch Pane | (↑/↓) Navigate | (h/l) Switch Tabs";
    let footer_widget = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);
    f.render_widget(footer_widget, footer_area);
}
