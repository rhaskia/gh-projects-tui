use crossterm::event::{KeyCode, KeyEvent};
use serde_json::value::Value;
use crate::project::{Field, FieldOption, Item, ProjectInfo};
use std::string::String;

#[derive(PartialEq)]
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
    pub cursor_pos: u16,
    pub current_options: Option<Vec<FieldOption>>,
    pub current_option: usize,
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
            input: InputApp { current_input: "".to_string(), cursor_pos: 0, current_options: None, current_option: 0 },
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

    pub fn shift_option_up(&mut self) {
        self.input.current_option = match self.input.current_option {
            0 => self.input.current_options.as_ref().unwrap().len() - 1,
            _ => self.input.current_option - 1,
        };
    }

    pub fn shift_option_down(&mut self) {
        self.input.current_option += 1;
        if self.input.current_option >= self.input.current_options.as_ref().unwrap().len() {
            self.input.current_option = 0;
        }
    }

    pub fn begin_editing(&mut self) {
        self.menu_state = InputMode::Input;

        let index_field = self.fields[self.column_state].name.to_ascii_lowercase();

        self.input.current_input = String::from(
            self.items[self.item_state]
            .fields.get(&*index_field)
            .unwrap_or(&Value::String(String::new()))
            .as_str().unwrap_or(&*String::new()));

        self.input.cursor_pos = self.input.current_input.len() as u16;

        self.input.current_options = self.fields[self.column_state].options.clone();
    }

    pub fn backspace(&mut self) {
        if self.input.cursor_pos == 0 { return; }

        self.input.cursor_pos -= 1;
        self.input.current_input.remove(self.input.cursor_pos as usize);
    }

    pub fn insert_char(&mut self, c: char) {
        self.input.current_input.insert(self.input.cursor_pos as usize, c);
        self.input.cursor_pos += 1;
    }

    pub fn cursor_left(&mut self) {
        if self.input.cursor_pos != 0 { self.input.cursor_pos -= 1; }
    }

    pub fn cursor_right(&mut self) {
        if self.input.cursor_pos != self.input.current_input.len() as u16 { self.input.cursor_pos += 1; }
    }

    pub fn save_field(&mut self, s: String) {
        self.set_field_at(self.item_state, self.column_state, s);
    }

    pub fn set_string(&mut self) {
        self.save_field(self.input.current_input.clone())
    }

    pub fn set_option(&mut self) {
        if let Some(ref options) = self.input.current_options {
            self.save_field(options[self.input.current_option].name.clone());
        }
    }

    pub fn set_field_at(&mut self, item: usize, field: usize, input: String) {
        let index_field = self.fields[field].name.to_ascii_lowercase();
        self.items[item].fields
            .insert(index_field, Value::String(input));
    }

    pub fn get_field_at(&self, item: usize, field: usize) -> &str {
        let index_field = self.fields[field].name.to_ascii_lowercase();
        self.items[item].fields
            .get(&*index_field)
            .unwrap_or(&Value::Bool(false))
            .as_str()
            .unwrap_or("")
    }
}

pub fn insert_mode_keys(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Esc => { app.menu_state = InputMode::Normal; }
        _ => {},
    }

    if app.fields[app.column_state].options.is_some() {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => { app.shift_option_down(); }
            KeyCode::Char('k') | KeyCode::Up => { app.shift_option_up(); }

            KeyCode::Enter => { app.set_option() }

            _ => {}
        }
    }
    else {
        match key.code {
            KeyCode::Char(a) => { app.insert_char(a); }
            KeyCode::Backspace => { app.backspace(); }

            KeyCode::Enter => { app.set_string() }

            KeyCode::Left => { app.cursor_left(); }
            KeyCode::Right => { app.cursor_right(); }
            _ => {}
        }
    }
}

pub fn normal_mode_keys(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Char('q') => { app.exit = true; }

        KeyCode::Char('j') | KeyCode::Down => { app.next(); }
        KeyCode::Char('k') | KeyCode::Up => { app.previous(); }
        KeyCode::Char('h') | KeyCode::Left => { app.left(); }
        KeyCode::Char('l') | KeyCode::Right => { app.right(); }

        KeyCode::Char('i') => { app.begin_editing(); }

        _ => {},
    }
}