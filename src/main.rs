use std::io;

use ratatui::{backend::CrosstermBackend, Terminal};

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

mod persistence;
mod screens;
mod ui;

use screens::menu::MenuScreen;
use ui::screen::{Screen, ScreenAction};

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut current_screen: Box<dyn Screen> = Box::new(MenuScreen::new());

    loop {
        terminal.draw(|f| {
            current_screen.render(f, f.area());
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    match current_screen.handle_input(key) {
                        Some(ScreenAction::Exit) => break,
                        Some(ScreenAction::Switch(next_screen)) => {
                            current_screen = next_screen;
                        }
                        None => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
