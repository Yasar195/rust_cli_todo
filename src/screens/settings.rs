use crate::ui::navigation::NavigatableList;
use crate::ui::screen::{Screen, ScreenAction};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect, Alignment};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;
use std::sync::mpsc::{self, Receiver};
use std::thread;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub enum UpdateState {
    Idle,
    Checking,
    UpdateAvailable(String),
}

pub struct SettingsScreen {
    pub title: String,
    pub list: NavigatableList,
    pub status_message: Option<String>,
    pub update_state: UpdateState,
    update_rx: Option<Receiver<Option<String>>>,
}

impl SettingsScreen {
    pub fn new() -> Self {
        let mut list = NavigatableList {
            state: ratatui::widgets::ListState::default(),
            options: vec![
                "Check for Updates".to_string(),
                "Back".to_string(),
            ],
        };
        list.state.select(Some(0));

        SettingsScreen {
            title: "Settings".to_string(),
            list,
            status_message: None,
            update_state: UpdateState::Idle,
            update_rx: None,
        }
    }

    fn check_for_updates(&mut self) {
        self.update_state = UpdateState::Checking;
        self.status_message = Some(" Checking for updates...".to_string());

        let (tx, rx) = mpsc::channel();
        self.update_rx = Some(rx);

        thread::spawn(move || {
            let result = match crate::system::update::check_for_updates() {
                Ok(info) if info.update_available => Some(info.latest_version),
                _ => None,
            };
            let _ = tx.send(result);
        });
    }

    fn poll_updates(&mut self) {
        if let Some(rx) = &self.update_rx {
            if let Ok(result) = rx.try_recv() {
                self.update_rx = None; // clear the receiver
                if let Some(latest) = result {
                    self.update_state = UpdateState::UpdateAvailable(latest);
                } else {
                    self.update_state = UpdateState::Idle;
                    self.status_message = Some(format!(" ✓ Version {} is up to date.", VERSION));
                }
            }
        }
    }
}

impl Screen for SettingsScreen {
    fn handle_input(&mut self, key: KeyEvent) -> Option<ScreenAction> {
        // Intercept input if update prompt is active
        if let UpdateState::UpdateAvailable(_) = &self.update_state {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                    return Some(ScreenAction::UpdateAndExit);
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.update_state = UpdateState::Idle;
                    self.status_message = Some(" Update cancelled.".to_string());
                    return None;
                }
                _ => return None,
            }
        }

        match key.code {
            KeyCode::Down => {
                self.list.next();
                None
            }
            KeyCode::Up => {
                self.list.previous();
                None
            }
            KeyCode::Enter => {
                let selected = self.list.state.selected().unwrap_or(0);
                match self.list.options[selected].as_str() {
                    "Check for Updates" => {
                        self.check_for_updates();
                        None
                    }
                    "Back" => {
                        let menu = crate::screens::menu::MenuScreen::new();
                        Some(ScreenAction::Switch(Box::new(menu)))
                    }
                    _ => None,
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                let menu = crate::screens::menu::MenuScreen::new();
                Some(ScreenAction::Switch(Box::new(menu)))
            }
            _ => None,
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Poll for update results before rendering
        self.poll_updates();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(area);

        // --- Option list ---
        let items: Vec<ListItem> = self
            .list
            .options
            .iter()
            .map(|label| ListItem::new(format!("  {}", label)))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!(" {} ", self.title))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, chunks[0], &mut self.list.state);

        // --- Status bar ---
        let (status_text, status_color) = match &self.update_state {
            UpdateState::Checking => (" Checking for updates...".to_string(), Color::Yellow),
            UpdateState::UpdateAvailable(_) => (" Waiting for input...".to_string(), Color::Yellow),
            UpdateState::Idle => {
                let msg = self.status_message.clone().unwrap_or_else(|| {
                    " Press Enter on an option to select.  |  Esc / q → Back".to_string()
                });
                (msg, Color::Green)
            }
        };

        let status = Paragraph::new(Line::from(vec![Span::styled(
            status_text,
            Style::default().fg(status_color),
        )]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

        frame.render_widget(status, chunks[1]);
        
        // --- Overlay Popup ---
        if let UpdateState::UpdateAvailable(latest) = &self.update_state {
            let prompt_text = vec![
                Line::from(""),
                Line::from(vec![
                    Span::raw("A new version ("),
                    Span::styled(latest.clone(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw(") is available!"),
                ]),
                Line::from(""),
                Line::from(Span::styled("Would you like to install it now?", Style::default().fg(Color::DarkGray))),
                Line::from(""),
                Line::from(vec![
                    Span::styled(" [y] Yes  ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::styled(" [n] No ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                ]),
            ];
            
            let popup = Paragraph::new(prompt_text)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: false })
                .block(
                    Block::default()
                        .title(" Update Available ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Yellow)),
                );
            // Center area computation
            let popup_area = centered_rect(50, 40, area);
            frame.render_widget(Clear, popup_area); // This clears out the background
            frame.render_widget(popup, popup_area);
        }
    }
}

/// Helper function to create a centered rect using up certain percentage of the available rect `r`
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
