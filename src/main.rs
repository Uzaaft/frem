mod api;
mod auth;
mod config;
mod oauth;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;

use crate::api::client::LinearClient;
use crate::api::types::Issue;

struct App {
    should_quit: bool,
    client: LinearClient,
    issues: Vec<Issue>,
    selected_index: usize,
    loading: bool,
    error: Option<String>,
}

impl App {
    fn new(client: LinearClient) -> Self {
        Self {
            should_quit: false,
            client,
            issues: Vec::new(),
            selected_index: 0,
            loading: true,
            error: None,
        }
    }

    fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_index < self.issues.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
            }
            KeyCode::Char('r') => {
                self.load_issues();
            }
            _ => {}
        }
    }

    fn load_issues(&mut self) {
        self.loading = true;
        self.error = None;
        
        match self.client.get_issues(50) {
            Ok(issues) => {
                self.issues = issues;
                self.loading = false;
                if self.selected_index >= self.issues.len() && !self.issues.is_empty() {
                    self.selected_index = self.issues.len() - 1;
                }
            }
            Err(e) => {
                self.error = Some(e.to_string());
                self.loading = false;
            }
        }
    }
}

fn main() -> Result<()> {
    let client = auth::ensure_authenticated()?;
    
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(client);
    app.load_issues();
    
    let res = run_app(&mut terminal, app);

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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                app.on_key(key.code);
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    let header = Paragraph::new("Linear TUI")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    if let Some(error) = &app.error {
        let error_msg = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL).title("Error"));
        f.render_widget(error_msg, chunks[1]);
    } else if app.loading {
        let loading = Paragraph::new("Loading issues...")
            .block(Block::default().borders(Borders::ALL).title("Issues"));
        f.render_widget(loading, chunks[1]);
    } else {
        let items: Vec<ListItem> = app
            .issues
            .iter()
            .enumerate()
            .map(|(i, issue)| {
                let style = if i == app.selected_index {
                    Style::default().bg(Color::DarkGray).fg(Color::White)
                } else {
                    Style::default()
                };
                
                let content = format!(
                    "{} {} - {}",
                    issue.identifier,
                    issue.state.name,
                    issue.title
                );
                
                ListItem::new(content).style(style)
            })
            .collect();

        let main_list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Issues"))
            .style(Style::default().fg(Color::White));
        f.render_widget(main_list, chunks[1]);
    }

    let footer = Paragraph::new("[q]uit | [r]efresh | [↑/k] up | [↓/j] down")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}