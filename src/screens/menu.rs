use crate::ui::navigation::NavigatableList;
use crate::ui::screen::{Screen, ScreenAction};
use crossterm::event::{KeyEvent, KeyCode, Event};
use ratatui::Frame;
use ratatui::layout::Rect;

pub struct MenuScreen {
    list: NavigatableList
}

impl Screen for MenuScreen {
    fn handle_input(&mut self, key: KeyEvent) -> Option<ScreenAction> {
        match key.code {
            KeyCode::Down => self.list.next(),
            KeyCode::Up => self.list.previous(),
            KeyCode::Enter => {
                let selected = self.list.state.selected().unwrap_or(0);
                println!("Selected: {}", self.list.options[selected]);
            }
            _ => {}
        }   
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ratatui::widgets::ListItem> = self.list.options.iter().map(|t| ratatui::widgets::ListItem::new(t.as_str())).collect();
        let list = ratatui::widgets::List::new(items)
            .block(ratatui::widgets::Block::default().title(self.screen_name.as_str()).borders(ratatui::widgets::Borders::ALL))
            .highlight_style(ratatui::style::Style::default().bg(ratatui::style::Color::Blue).fg(ratatui::style::Color::White))
            .highlight_symbol(">> ");
        frame.render_stateful_widget(list, area, &mut self.list.state);
    }
}