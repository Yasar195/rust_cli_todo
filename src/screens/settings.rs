use crate::ui::navigation::NavigatableList;
use crate::ui::screen::{Screen, ScreenAction};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct SettingsScreen {
    pub title: String,
    pub list: NavigatableList,
    /// Status message shown at the bottom of the screen
    pub status_message: Option<String>,
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
        }
    }
}

impl Screen for SettingsScreen {
    fn handle_input(&mut self, key: KeyEvent) -> Option<ScreenAction> {
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
                        self.status_message = Some(format!(
                            " ✓  Version {} is up to date.",
                            VERSION
                        ));
                        None
                    }
                    "Back" => {
                        // Switch back to the main menu
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
        // Split into list area and status bar at the bottom
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
        let status_text = self
            .status_message
            .clone()
            .unwrap_or_else(|| " Press Enter on an option to select.  |  Esc / q → Back".to_string());

        let status = Paragraph::new(Line::from(vec![Span::styled(
            status_text,
            Style::default().fg(Color::Green),
        )]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

        frame.render_widget(status, chunks[1]);
    }
}
