use crate::persistence::persistence::{Persistence, Task};
use crate::ui::screen::{Screen, ScreenAction};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;

// ── Mode state machine ────────────────────────────────────────────────────────

enum TasksMode {
    /// Normal view: navigate list, see details
    View,
    /// Inline add form: typing title then description
    Adding {
        active_field: AddField,
        title: String,
        description: String,
    },
    /// Waiting for confirmation before deleting
    ConfirmDelete,
}

#[derive(PartialEq)]
enum AddField {
    Title,
    Description,
}

// ── Screen ────────────────────────────────────────────────────────────────────

pub struct TasksScreen {
    pub title: String,
    tasks: Vec<Task>,
    state: ListState,
    mode: TasksMode,
    persistence: Persistence,
}

impl TasksScreen {
    pub fn new() -> Self {
        let persistence = Persistence::new();
        persistence.sync_schema();
        let tasks = persistence.get_all::<Task>();

        let mut state = ListState::default();
        if !tasks.is_empty() {
            state.select(Some(0));
        }

        TasksScreen {
            title: "Tasks".to_string(),
            tasks,
            state,
            mode: TasksMode::View,
            persistence,
        }
    }

    fn reload(&mut self) {
        self.tasks = self.persistence.get_all::<Task>();
        // keep selection in bounds
        if self.tasks.is_empty() {
            self.state.select(None);
        } else {
            let i = self.state.selected().unwrap_or(0).min(self.tasks.len() - 1);
            self.state.select(Some(i));
        }
    }

    fn selected_task(&self) -> Option<&Task> {
        self.state.selected().and_then(|i| self.tasks.get(i))
    }

    fn list_next(&mut self) {
        if self.tasks.is_empty() { return; }
        let i = match self.state.selected() {
            Some(i) => if i >= self.tasks.len() - 1 { 0 } else { i + 1 },
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn list_prev(&mut self) {
        if self.tasks.is_empty() { return; }
        let i = match self.state.selected() {
            Some(i) => if i == 0 { self.tasks.len() - 1 } else { i - 1 },
            None => 0,
        };
        self.state.select(Some(i));
    }
}

// ── Input handling ────────────────────────────────────────────────────────────

impl Screen for TasksScreen {
    fn handle_input(&mut self, key: KeyEvent) -> Option<ScreenAction> {
        match &mut self.mode {
            // ── View mode ─────────────────────────────────────────────
            TasksMode::View => match key.code {
                KeyCode::Up => { self.list_prev(); None }
                KeyCode::Down => { self.list_next(); None }
                KeyCode::Char('a') => {
                    self.mode = TasksMode::Adding {
                        active_field: AddField::Title,
                        title: String::new(),
                        description: String::new(),
                    };
                    None
                }
                KeyCode::Char('d') | KeyCode::Delete => {
                    if self.selected_task().is_some() {
                        self.mode = TasksMode::ConfirmDelete;
                    }
                    None
                }
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('b') => {
                    let menu = crate::screens::menu::MenuScreen::new();
                    Some(ScreenAction::Switch(Box::new(menu)))
                }
                _ => None,
            },

            // ── Add mode ──────────────────────────────────────────────
            TasksMode::Adding { active_field, title, description } => match key.code {
                KeyCode::Esc => {
                    self.mode = TasksMode::View;
                    None
                }
                KeyCode::Tab => {
                    *active_field = if *active_field == AddField::Title {
                        AddField::Description
                    } else {
                        AddField::Title
                    };
                    None
                }
                KeyCode::Enter => {
                    if !title.trim().is_empty() {
                        let t = title.trim().to_string();
                        let d = description.trim().to_string();
                        let task = Task {
                            id: None,
                            title: t,
                            description: if d.is_empty() { None } else { Some(d) },
                            completed: false,
                        };
                        self.persistence.save(&task);
                        self.reload();
                        if !self.tasks.is_empty() {
                            self.state.select(Some(self.tasks.len() - 1));
                        }
                    }
                    self.mode = TasksMode::View;
                    None
                }
                KeyCode::Backspace => {
                    match active_field {
                        AddField::Title => { title.pop(); }
                        AddField::Description => { description.pop(); }
                    }
                    None
                }
                KeyCode::Char(c) => {
                    match active_field {
                        AddField::Title => title.push(c),
                        AddField::Description => description.push(c),
                    }
                    None
                }
                _ => None,
            },
            // ── Confirm delete mode ───────────────────────────────────
            TasksMode::ConfirmDelete => match key.code {
                KeyCode::Enter => {
                    if let Some(task) = self.selected_task() {
                        if let Some(id) = task.id {
                            self.persistence.delete::<Task>(id);
                        }
                    }
                    self.reload();
                    self.mode = TasksMode::View;
                    None
                }
                KeyCode::Esc | KeyCode::Char('n') => {
                    self.mode = TasksMode::View;
                    None
                }
                _ => None,
            },
        }
    }

    // ── Rendering ─────────────────────────────────────────────────────────────

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(area);

        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
            .split(vertical[0]);

        // ── Left: task list ───────────────────────────────────────────
        let items: Vec<ListItem> = if self.tasks.is_empty() {
            vec![ListItem::new(Span::styled(
                "  (no tasks — press 'a' to add one)",
                Style::default().fg(Color::DarkGray),
            ))]
        } else {
            self.tasks
                .iter()
                .map(|t| {
                    let (icon, style) = if t.completed {
                        ("✓", Style::default().fg(Color::DarkGray).add_modifier(Modifier::CROSSED_OUT))
                    } else {
                        ("○", Style::default().fg(Color::White))
                    };
                    ListItem::new(Line::from(vec![
                        Span::styled(format!("  {} ", icon), style),
                        Span::styled(t.title.clone(), style),
                    ]))
                })
                .collect()
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!(" {} ({}) ", self.title, self.tasks.len()))
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

        frame.render_stateful_widget(list, horizontal[0], &mut self.state);

        // ── Right: context panel (details / add form) ─────────────────
        match &self.mode {
            TasksMode::View | TasksMode::ConfirmDelete => {
                let detail_lines = if let Some(task) = self.selected_task() {
                    let status_str = if task.completed { "✓  Completed" } else { "○  Pending" };
                    let status_color = if task.completed { Color::Green } else { Color::Magenta };
                    let desc = task.description.as_deref().unwrap_or("No description.");
                    vec![
                        Line::from(vec![
                            Span::styled("  ID:     ", Style::default().fg(Color::DarkGray)),
                            Span::raw(task.id.unwrap_or(0).to_string()),
                        ]),
                        Line::from(""),
                        Line::from(vec![
                            Span::styled("  Title:  ", Style::default().fg(Color::DarkGray)),
                            Span::styled(task.title.clone(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                        ]),
                        Line::from(""),
                        Line::from(vec![
                            Span::styled("  Status: ", Style::default().fg(Color::DarkGray)),
                            Span::styled(status_str, Style::default().fg(status_color)),
                        ]),
                        Line::from(""),
                        Line::from(Span::styled("  Description:", Style::default().fg(Color::DarkGray))),
                        Line::from(Span::raw(format!("  {}", desc))),
                    ]
                } else {
                    vec![Line::from(Span::styled(
                        "  Select a task to see details.",
                        Style::default().fg(Color::DarkGray),
                    ))]
                };

                let detail = Paragraph::new(detail_lines)
                    .wrap(Wrap { trim: false })
                    .block(
                        Block::default()
                            .title(" Details ")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Cyan)),
                    );
                frame.render_widget(detail, horizontal[1]);
            }

            TasksMode::Adding { active_field, title, description } => {
                let title_style = if *active_field == AddField::Title {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                let desc_style = if *active_field == AddField::Description {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let form_lines = vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        "  Title",
                        Style::default().fg(Color::DarkGray),
                    )),
                    Line::from(vec![
                        Span::styled("  > ", Style::default().fg(Color::Cyan)),
                        Span::styled(format!("{}_", title), title_style),
                    ]),
                    Line::from(""),
                    Line::from(Span::styled(
                        "  Description  (optional)",
                        Style::default().fg(Color::DarkGray),
                    )),
                    Line::from(vec![
                        Span::styled("  > ", Style::default().fg(Color::Cyan)),
                        Span::styled(format!("{}_", description), desc_style),
                    ]),
                    Line::from(""),
                    Line::from(""),
                    Line::from(Span::styled(
                        "  Tab → next field   Enter → save   Esc → cancel",
                        Style::default().fg(Color::DarkGray),
                    )),
                ];

                let form = Paragraph::new(form_lines)
                    .block(
                        Block::default()
                            .title(" Add Task ")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Yellow)),
                    );
                frame.render_widget(form, horizontal[1]);
            }
        }

        // ── Bottom: status / hint bar ────────────────────────────────
        let (status_text, status_color) = match &self.mode {
            TasksMode::View => (
                "  ↑↓ navigate   a → add   d → delete   q/Esc → back".to_string(),
                Color::Green,
            ),
            TasksMode::Adding { .. } => (
                "  Adding task — Tab: switch field   Enter: save   Esc: cancel".to_string(),
                Color::Yellow,
            ),
            TasksMode::ConfirmDelete => (
                "  ⚠  Delete this task?   Enter → confirm   Esc/n → cancel".to_string(),
                Color::Red,
            ),
        };

        let status = Paragraph::new(Line::from(Span::styled(status_text, Style::default().fg(status_color))))
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));

        frame.render_widget(status, vertical[1]);
    }
}
