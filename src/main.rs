use std::io::stdout;
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

struct App {
    items: Vec<(String, u32)>, // (name, app_id)
    state: ListState,
}

impl App {
    fn new(items: Vec<(String, u32)>) -> Self {
        Self {
            items,
            state: ListState::default(),
        }
    }

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

    fn open_selected(&self, steam_path: &std::path::Path) {
        if let Some(i) = self.state.selected() {
            let app_id = self.items[i].1;
            let pfx_path = steam_path
                .join("steamapps")
                .join("compatdata")
                .join(format!("{}", app_id))
                .join("pfx");
            if pfx_path.exists() {
                let _ = Command::new("xdg-open").arg(&pfx_path).spawn();
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let steam_dir = SteamDir::locate()?;
    let compat_tools = steam_dir.compat_tool_mapping()?;
    let mut items = Vec::new();

    for shortcut in steam_dir.shortcuts()? {
        let shortcut = shortcut?;
        if compat_tools.contains_key(&shortcut.app_id) {
            items.push((shortcut.app_name, shortcut.app_id));
        }
    }

    if items.is_empty() {
        println!("No non-Steam games with Wine prefixes found.");
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
                .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
                .split(size);

            let list_items: Vec<ListItem> = app
                .items
                .iter()
                .map(|(name, app_id)| {
                    ListItem::new(Span::styled(
                        format!("{} (App ID: {})", name, app_id),
                        Style::default().fg(Color::White),
                    ))
                })
                .collect();

            let list = List::new(list_items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Non-Steam Games with Wine Prefixes (↑/↓ to navigate, Enter to open, q to quit)"),
                )
                .highlight_style(Style::default().bg(Color::Blue))
                .highlight_symbol(">> ");

            let footer = Paragraph::new("Press 'q' to exit the application.")
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::Gray));

            f.render_stateful_widget(list, chunks[0], &mut app.state);
            f.render_widget(footer, chunks[1]);
        })?;

        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                match key.code {
                    crossterm::event::KeyCode::Char('q') => break,
                    crossterm::event::KeyCode::Down => app.next(),
                    crossterm::event::KeyCode::Up => app.previous(),
                    crossterm::event::KeyCode::Enter => app.open_selected(steam_dir.path()),
                    _ => {}
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
