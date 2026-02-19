use std::{io, task};

use ratatui::{
    backend::CrosstermBackend,
    widgets::{ Block, Borders, List, ListItem },
    Terminal
};

use crossterm::{
    event::{ self, Event, KeyCode },
    execute,
    terminal::{ disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen },
};

mod persistence;
mod ui;
use persistence::persistence::Task;

use crate::persistence::persistence::Persistable;

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    // let persistence = persistence::persistence::Persistence::new();
    let mut app= ui::home::App::new();
    app.menu_state.select(Some((0)));

    // persistence.sync_schema();


    loop {
        terminal.draw(|f| {

            let items: Vec<ListItem> = app.menu_options.iter().map(|t| ListItem::new(t.as_str())).collect();

            let list = List::new(items).block(Block::default().title(app.screen_name.as_str()).borders(Borders::ALL)).highlight_style(ratatui::style::Style::default().bg(ratatui::style::Color::Blue).fg(ratatui::style::Color::White)).highlight_symbol(">> ");

            f.render_stateful_widget(list, f.area(), &mut app.menu_state);
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Down => app.next(),
                        KeyCode::Up => app.previous(),
                        KeyCode::Enter => {
                            // Handle the selection here
                            let selected = app.menu_state.selected().unwrap();
                            println!("Selected: {}", app.menu_options[selected]);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())

}
