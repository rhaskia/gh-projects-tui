use ratatui::widgets::ListState;
use crate::project::{Field, Item, ProjectInfo};

pub enum Menu {
    Items,
    Fields,
}

pub(crate) struct App {
    pub item_state: ListState,
    pub field_state: ListState,
    pub column_state: usize,
    pub menu_state: Menu,
    pub items: Vec<Item>,
    pub fields: Vec<Field>,
    pub projects: Vec<ProjectInfo>
}

impl App {
    pub fn new(items: Vec<Item>, fields: Vec<Field>, projects: Vec<ProjectInfo>) -> Self {
        App {
            item_state: ListState::default().with_selected(Some(0)),
            field_state: ListState::default().with_selected(Some(0)),
            menu_state: Menu::Items,
            column_state: 0,
            items,
            fields,
            projects,
        }
    }

    pub fn right(&mut self) {
        self.column_state += 1;
        if self.column_state > self.fields.len()
        { self.column_state = 0; }
    }

    pub fn left(&mut self) {
        self.column_state = match self.column_state {
            0 => self.fields.len(),
            _ => self.column_state - 1,
        };
    }

    pub fn next(&mut self) {
        let i = match self.item_state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.item_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.item_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.item_state.select(Some(i));
    }
}