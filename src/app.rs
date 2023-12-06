use crossterm::event::{KeyCode, KeyEvent};
use crate::project::{Field, Item, ProjectInfo};

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

    pub input: InputApp,
}

pub struct InputApp {
    pub current_input: String,
    pub cursor_pos: usize,
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
            input: InputApp { current_input: "".to_string(), cursor_pos: 0 },
        }
    }

    pub fn right(&mut self) {
        self.column_state += 1;
        if self.column_state >= self.fields.len() {
            self.column_state = 0;
        }
    }

    pub fn left(&mut self) {
        self.column_state = match self.column_state {
            0 => self.fields.len() - 1,
            _ => self.column_state - 1,
        };
    }

    pub fn next(&mut self) {
        self.item_state += 1;
        if self.item_state >= self.items.len() {
            self.item_state = 0;
        }
    }

    pub fn previous(&mut self) {
        self.item_state = match self.item_state {
            0 => self.items.len() - 1,
            _ => self.item_state - 1,
        };
    }

    pub fn begin_editing(&mut self) {

    }

    pub fn backspace(&mut self) {

    }

    pub fn insert_char(&mut self, c: char) {

    }
}

pub fn insert_mode_keys(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Esc => { app.menu_state = InputMode::Normal; }

        KeyCode::Char(a) => { app.insert_char(a); }
        KeyCode::Backspace => { app.backspace(); }

        _ => {},
    }
}

pub fn normal_mode_keys(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Char('q') => { app.exit = true; }

        KeyCode::Char('j') => { app.next(); }
        KeyCode::Char('k') => { app.previous(); }
        KeyCode::Char('h') => { app.left(); }
        KeyCode::Char('l') => { app.right(); }

        KeyCode::Char('i') => { app.menu_state = InputMode::Input; }

        _ => {},
    }
}