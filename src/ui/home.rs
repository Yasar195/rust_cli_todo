use ratatui::widgets::ListState;

pub struct App {
    pub menu_options: Vec<String>,
    pub menu_state: ListState,
    pub screen_name: String
}

impl App {
    pub fn next(&mut self) {
        let i = match self.menu_state.selected() {
            Some(i) => {
                if i >= self.menu_options.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.menu_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.menu_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.menu_options.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0
        };
        self.menu_state.select(Some(i));
    }

    pub fn new() -> Self {
        App {
            menu_options: vec!["Home".to_string(), "Add Task".to_string(), "Settings".to_string(), "Exit".to_string()],
            menu_state: ListState::default(),
            screen_name: "Menu".to_string()
        }
    }
}