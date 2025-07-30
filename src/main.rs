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
    filtered_issues: Vec<usize>,
    selected_index: usize,
    loading: bool,
    error: Option<String>,
    search_mode: bool,
    search_query: String,
}

impl App {
    fn new(client: LinearClient) -> Self {
        Self {
            should_quit: false,
            client,
            issues: Vec::new(),
            filtered_issues: Vec::new(),
            selected_index: 0,
            loading: true,
            error: None,
            search_mode: false,
            search_query: String::new(),
        }
    }

    fn on_key(&mut self, key: KeyCode) {
        if self.search_mode {
            match key {
                KeyCode::Esc => {
                    self.search_mode = false;
                    self.search_query.clear();
                    self.filter_issues();
                }
                KeyCode::Enter => {
                    self.search_mode = false;
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.filter_issues();
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    self.filter_issues();
                }
                _ => {}
            }
        } else {
            match key {
                KeyCode::Char('q') => self.should_quit = true,
                KeyCode::Char('/') => {
                    self.search_mode = true;
                    self.search_query.clear();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let max_index = if self.filtered_issues.is_empty() {
                        self.issues.len()
                    } else {
                        self.filtered_issues.len()
                    }.saturating_sub(1);
                    
                    if self.selected_index < max_index {
                        self.selected_index += 1;
                    }
                }
                KeyCode::Char('r') => {
                    self.load_issues();
                }
                _ => {}
            }
        }
    }

    fn load_issues(&mut self) {
        self.loading = true;
        self.error = None;
        
        match self.client.get_issues(50) {
            Ok(issues) => {
                self.issues = issues;
                self.loading = false;
                self.filter_issues();
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
    
    fn filter_issues(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_issues.clear();
        } else {
            let query = self.search_query.to_lowercase();
            self.filtered_issues = self.issues
                .iter()
                .enumerate()
                .filter(|(_, issue)| {
                    issue.title.to_lowercase().contains(&query)
                        || issue.identifier.to_lowercase().contains(&query)
                        || issue.description.as_ref()
                            .map(|d| d.to_lowercase().contains(&query))
                            .unwrap_or(false)
                })
                .map(|(i, _)| i)
                .collect();
        }
        
        if self.selected_index >= self.filtered_issues.len() && !self.filtered_issues.is_empty() {
            self.selected_index = self.filtered_issues.len() - 1;
        } else if self.filtered_issues.is_empty() && !self.issues.is_empty() {
            self.selected_index = 0;
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

    let header_text = if app.search_mode {
        format!("Linear TUI - Search: {}_", app.search_query)
    } else if !app.search_query.is_empty() {
        format!("Linear TUI - Filter: {}", app.search_query)
    } else {
        "Linear TUI".to_string()
    };
    
    let header = Paragraph::new(header_text)
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
        let issues_to_display: Vec<(usize, &Issue)> = if app.filtered_issues.is_empty() {
            app.issues.iter().enumerate().collect()
        } else {
            app.filtered_issues
                .iter()
                .map(|&i| (i, &app.issues[i]))
                .collect()
        };
        
        let items: Vec<ListItem> = issues_to_display
            .iter()
            .enumerate()
            .map(|(display_idx, (_, issue))| {
                let style = if display_idx == app.selected_index {
                    Style::default().bg(Color::DarkGray).fg(Color::White)
                } else {
                    Style::default()
                };
                
                let state_color = match issue.state.name.as_str() {
                    "Todo" | "Backlog" => Color::Gray,
                    "In Progress" => Color::Yellow,
                    "Done" | "Completed" => Color::Green,
                    "Canceled" => Color::Red,
                    _ => Color::White,
                };
                
                let priority_icon = match issue.priority {
                    0 => "○",
                    1 => "◔",
                    2 => "◑",
                    3 => "◕",
                    _ => "●",
                };
                
                let assignee = issue.assignee.as_ref()
                    .map(|u| u.name.chars().take(10).collect::<String>())
                    .unwrap_or_else(|| "Unassigned".to_string());
                
                let content = format!(
                    "{} {} │ {} │ {} │ {}",
                    priority_icon,
                    issue.identifier,
                    format!("{:>12}", issue.state.name),
                    format!("{:<10}", assignee),
                    issue.title
                );
                
                ListItem::new(content).style(style)
            })
            .collect();

        let title = if !app.filtered_issues.is_empty() {
            format!("Issues ({}/{})", app.filtered_issues.len(), app.issues.len())
        } else {
            format!("Issues ({})", app.issues.len())
        };
        
        let main_list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .style(Style::default().fg(Color::White));
        f.render_widget(main_list, chunks[1]);
    }

    let footer_text = if app.search_mode {
        "[Esc] cancel | [Enter] confirm | Type to search..."
    } else {
        "[q]uit | [r]efresh | [/] search | [↑/k] up | [↓/j] down"
    };
    
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}