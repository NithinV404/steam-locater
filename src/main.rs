use std::io::stdout;
use std::path::PathBuf;
use std::process::Command;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use steamlocate::SteamDir;

#[derive(Clone)]
struct Game {
    name: String,
    app_id: u32,
    is_non_steam: bool,
    path: PathBuf,
}

struct App {
    items: Vec<Game>,
    filtered_items: Vec<Game>,
    search_query: String,
    in_search_mode: bool,
    status_message: String,
    state: ListState,
}

impl App {
    fn new(items: Vec<Game>) -> Self {
        let filtered_items = items.clone();
        Self {
            items,
            filtered_items,
            search_query: String::new(),
            in_search_mode: false,
            status_message: "Use '/' to search, 'q' to exit.".to_string(),
            state: ListState::default(),
        }
    }

    fn update_filter(&mut self) {
        self.filtered_items = self
            .items
            .iter()
            .filter(|game| {
                game.name
                    .to_lowercase()
                    .contains(&self.search_query.to_lowercase())
            })
            .cloned()
            .collect();
        // Reset selection if out of bounds
        if let Some(selected) = self.state.selected() {
            if selected >= self.filtered_items.len() {
                self.state.select(if self.filtered_items.is_empty() {
                    None
                } else {
                    Some(0)
                });
            }
        }
    }

    fn enter_search_mode(&mut self) {
        self.in_search_mode = true;
    }

    fn exit_search_mode(&mut self) {
        self.in_search_mode = false;
        self.search_query.clear();
        self.update_filter();
    }

    fn next(&mut self) {
        let len = self.filtered_items.len();
        let i = match self.state.selected() {
            Some(i) => {
                if i >= len - 1 {
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
        let len = self.filtered_items.len();
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn open_selected(&mut self) {
        if let Some(i) = self.state.selected() {
            let game = &self.filtered_items[i];
            if game.path.exists() {
                let _ = Command::new("xdg-open").arg(&game.path).spawn();
                self.status_message = if game.is_non_steam {
                    "Opened prefix folder.".to_string()
                } else {
                    "Opened game folder.".to_string()
                };
            } else {
                self.status_message = "Folder does not exist.".to_string();
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let steam_dir = SteamDir::locate()?;
    let compat_tools = steam_dir.compat_tool_mapping()?;
    let mut items = Vec::new();

    // Add Steam games
    if let Ok(libraries_iter) = steam_dir.libraries() {
        for folder in libraries_iter {
            let folder = folder?;
            for app_result in folder.apps() {
                let app = app_result?;
                if let Some(name) = app.name {
                    items.push(Game {
                        name,
                        app_id: app.app_id,
                        is_non_steam: false,
                        path: app.install_dir.into(),
                    });
                }
            }
        }
    }

    // Add non-Steam games with Wine prefixes
    for shortcut in steam_dir.shortcuts()? {
        let shortcut = shortcut?;
        if compat_tools.contains_key(&shortcut.app_id) {
            let pfx_path = steam_dir
                .path()
                .join("steamapps")
                .join("compatdata")
                .join(format!("{}", shortcut.app_id))
                .join("pfx");
            items.push(Game {
                name: shortcut.app_name,
                app_id: shortcut.app_id,
                is_non_steam: true,
                path: pfx_path,
            });
        }
    }

    if items.is_empty() {
        println!("No games found.");
        return Ok(());
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(items);
    app.state.select(Some(0));

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Percentage(94),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(size);

            let search_title = if app.in_search_mode {
                "Search (type to search, Enter to exit)"
            } else {
                "Search (press '/' to enter search mode)"
            };
            let search_block = Block::default().borders(Borders::ALL).title(search_title);
            let search_text = if app.search_query.is_empty() && !app.in_search_mode {
                "No search query"
            } else {
                &app.search_query
            };
            let search_paragraph = Paragraph::new(search_text)
                .block(search_block)
                .style(Style::default().fg(Color::White));

            let list_items: Vec<ListItem> = app
                .filtered_items
                .iter()
                .map(|game| {
                    let label = if game.is_non_steam { "Non-Steam: " } else { "" };
                    ListItem::new(Span::styled(
                        format!("{}{} (App ID: {})", label, game.name, game.app_id),
                        Style::default().fg(Color::White),
                    ))
                })
                .collect();

            let list_title = format!(
                "Games ({}/{}, ↑/↓ to navigate, Enter to open, q to quit)",
                app.filtered_items.len(),
                app.items.len()
            );
            let list = List::new(list_items)
                .block(Block::default().borders(Borders::ALL).title(list_title))
                .highlight_style(Style::default().bg(Color::Blue))
                .highlight_symbol(">> ");

            let footer = Paragraph::new(app.status_message.as_str())
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::Gray));

            f.render_widget(search_paragraph, chunks[0]);
            f.render_stateful_widget(list, chunks[1], &mut app.state);
            f.render_widget(footer, chunks[2]);
        })?;

        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                if app.in_search_mode {
                    match key.code {
                        crossterm::event::KeyCode::Enter => app.exit_search_mode(),
                        crossterm::event::KeyCode::Backspace => {
                            app.search_query.pop();
                            app.update_filter();
                        }
                        crossterm::event::KeyCode::Char(c) => {
                            app.search_query.push(c);
                            app.update_filter();
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        crossterm::event::KeyCode::Char('q') => break,
                        crossterm::event::KeyCode::Char('/') => app.enter_search_mode(),
                        crossterm::event::KeyCode::Down => app.next(),
                        crossterm::event::KeyCode::Up => app.previous(),
                        crossterm::event::KeyCode::Enter => app.open_selected(),
                        _ => {}
                    }
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
