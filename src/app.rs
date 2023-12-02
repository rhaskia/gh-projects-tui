use crate::project::{Field, Item, ProjectInfo};
use ratatui::widgets::ListState;

pub enum InputMode {
    Normal,
    Input,
}

pub(crate) struct App {
    pub item_state: usize,
    pub column_state: usize,
    pub menu_state: InputMode,
    pub exit: bool,

    pub items: Vec<Item>,
    pub fields: Vec<Field>,
    pub projects: Vec<ProjectInfo>,
}

impl App {
    pub fn new(items: Vec<Item>, fields: Vec<Field>, projects: Vec<ProjectInfo>) -> Self {
        App {
            item_state: 0,
            column_state: 0,

            menu_state: InputMode::Normal,
            exit: false,

            items,
            fields,
            projects,
        }
    }

    pub fn right(&mut self) {
        self.column_state += 1;
        if self.column_state > self.fields.len() {
            self.column_state = 0;
        }
    }

    pub fn left(&mut self) {
        self.column_state = match self.column_state {
            0 => self.fields.len(),
            _ => self.column_state - 1,
        };
    }

    pub fn next(&mut self) {
        self.item_state += 1;
        if self.item_state > self.items.len() {
            self.item_state = 0;
        }
    }

    pub fn previous(&mut self) {
        self.item_state = match self.item_state {
            0 => self.items.len(),
            _ => self.item_state - 1,
        };
    }
}
