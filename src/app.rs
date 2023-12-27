// TODO: fix on release
#![allow(dead_code)]
#![allow(unused_assignments)]
#![allow(unused_attributes)]
#![allow(unused_imports)]
#![allow(unused_macros)]
#![allow(unused_mut)]
#![allow(unused_variables)]

use crate::github;
use crate::project::*;
use anyhow::anyhow;
use crossterm::event::{KeyCode, KeyEvent};
use github_device_flow::Credential;
use serde_json::value::Value;
use std::string::String;

#[derive(PartialEq, Debug)]
pub enum InputMode {
    Normal,
    Input,
}

#[derive(Debug)]
pub(crate) struct App {
    pub project_state: usize,
    pub item_state: usize,
    pub field_state: usize,
    pub menu_state: InputMode,
    pub exit: bool,

    pub user_info: Option<UserInfo>,

    pub id: Option<Credential>,
    pub input: InputApp,
}

#[derive(Debug)]
pub struct InputApp {
    pub current_input: String,
    pub cursor_pos: u16,
    pub current_options: Option<Vec<FieldOption>>,
    pub current_option: usize,
}

#[derive(Debug)]
pub struct UserInfo {
    pub user: User,
    pub items: Vec<Item>,
    pub fields: Vec<Field>,
    pub projects: Vec<Project>,
}

impl App {
    pub fn setup() -> Self {
        App {
            project_state: 0,
            item_state: 0,
            field_state: 0,

            menu_state: InputMode::Normal,
            exit: false,

            user_info: None,
            id: None,

            input: InputApp {
                current_input: "".to_string(),
                cursor_pos: 0,
                current_options: None,
                current_option: 0,
            },
        }
    }

    pub fn reload_info(&mut self) -> anyhow::Result<()> {
        if let Some(cred) = &self.id {
            let user = github::get_user(&cred.token)?;
            let projects = github::get_project_ids(&cred.token, &user.login)?;
            let items = github::fetch_project_items(&cred.token, &projects[self.project_state].id)?;
            let fields =
                github::fetch_project_fields(&cred.token, &projects[self.project_state].id)?;

            self.user_info = Some(UserInfo {
                user,
                projects,
                items,
                fields,
            })
        }

        Ok(())
    }

    pub fn info(&self) -> anyhow::Result<&UserInfo> {
        self.user_info
            .as_ref()
            .ok_or_else(|| anyhow!("No user info loaded"))
    }

    pub fn mut_info(&mut self) -> anyhow::Result<&mut UserInfo> {
        self.user_info
            .as_mut()
            .ok_or_else(|| anyhow!("No user info loaded"))
    }

    pub fn right(&mut self) {
        if let Some(info) = &self.user_info {
            self.field_state += 1;
            if self.field_state >= info.fields.len() {
                self.field_state = 0;
            }
        }
    }

    pub fn left(&mut self) {
        if let Some(info) = &self.user_info {
            self.field_state = match self.field_state {
                0 => info.fields.len() - 1,
                _ => self.field_state - 1,
            };
        }
    }

    pub fn next(&mut self) {
        if let Some(info) = &self.user_info {
            self.item_state += 1;
            if self.item_state >= info.items.len() {
                self.item_state = 0;
            }
        }
    }

    pub fn previous(&mut self) {
        if let Some(info) = &self.user_info {
            self.item_state = match self.item_state {
                0 => info.items.len() - 1,
                _ => self.item_state - 1,
            };
        }
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

        if let Some(info) = &self.user_info {
            let field_value = info.items[self.item_state]
                .field_values
                .name_from_field(&info.fields[self.field_state].get_name());

            self.input.current_input = field_value.to_string();
            self.input.cursor_pos = self.input.current_input.len() as u16;
        }
    }

    pub fn backspace(&mut self) {
        if self.input.cursor_pos == 0 {
            return;
        }

        self.input.cursor_pos -= 1;
        self.input
            .current_input
            .remove(self.input.cursor_pos as usize);
    }

    pub fn insert_char(&mut self, c: char) {
        self.input
            .current_input
            .insert(self.input.cursor_pos as usize, c);
        self.input.cursor_pos += 1;
    }

    pub fn cursor_left(&mut self) {
        if self.input.cursor_pos != 0 {
            self.input.cursor_pos -= 1;
        }
    }

    pub fn cursor_right(&mut self) {
        if self.input.cursor_pos != self.input.current_input.len() as u16 {
            self.input.cursor_pos += 1;
        }
    }

    pub fn save_field(&mut self, s: String) -> anyhow::Result<()> {
        if let Some(app_info) = &self.user_info {
            match self.get_field_at(self.item_state, self.field_state)? {
                ProjectV2ItemField::Empty(v) => {}
                ProjectV2ItemField::TextValue { text, field } => {}
                ProjectV2ItemField::DateValue { date, field } => {}
                ProjectV2ItemField::SingleSelectValue { name, field } => {}
                ProjectV2ItemField::NumberValue { number, field } => {}
                ProjectV2ItemField::IterationValue {
                    duration,
                    title,
                    field,
                } => {}
            };

            github::update_item_field(
                &self.id.clone().ok_or_else(|| anyhow!("No Credential found"))?.token,
                &app_info.projects[self.project_state].id,
                &app_info.items[self.item_state].id,
                app_info.fields[self.field_state].get_id(),
                &s,
            )?;
            self.reload_info()?;
        }

        Ok(())
    }

    pub fn set_string(&mut self) -> Result<(), anyhow::Error> {
        self.save_field(self.input.current_input.clone())
    }

    pub fn set_option(&mut self) {
        if let Some(ref options) = self.input.current_options {
            self.save_field(options[self.input.current_option].name.clone());
        }
    }

    pub fn set_field_at(&mut self, item: usize, field: usize, input: String) {
        if let Some(app_info) = &self.user_info {
            self.reload_info();
        }
    }

    pub fn get_field_at(&self, item: usize, field: usize) -> anyhow::Result<&ProjectV2ItemField> {
        if let Some(app_info) = &self.user_info {
            let index_field = app_info.fields[field].get_name();
            Ok(app_info.items[item]
                .field_values
                .get_from_field(&index_field))
        } else {
            anyhow::bail!("No app info")
        }
    }
}

pub fn insert_mode_keys(key: KeyEvent, app: &mut App) -> anyhow::Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.menu_state = InputMode::Normal;
        }
        _ => {}
    }

    if let Some(app_info) = &app.user_info {
        if let Field::ProjectV2SingleSelectField(field) 
        = &app_info.fields[app.field_state] {
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                    app.shift_option_down();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    app.shift_option_up();
                }

                KeyCode::Enter => app.set_option(),

                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Char(a) => {
                    app.insert_char(a);
                }
                KeyCode::Backspace => {
                    app.backspace();
                }

                KeyCode::Enter => app.set_string()?,

                KeyCode::Left => {
                    app.cursor_left();
                }
                KeyCode::Right => {
                    app.cursor_right();
                }
                _ => {}
            }
        }
    }

    Ok(())
}

pub fn normal_mode_keys(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Char('q') => {
            app.exit = true;
        }

        KeyCode::Char('j') | KeyCode::Down => {
            app.next();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.previous();
        }
        KeyCode::Char('h') | KeyCode::Left => {
            app.left();
        }
        KeyCode::Char('l') | KeyCode::Right => {
            app.right();
        }

        KeyCode::Char('i') => {
            app.begin_editing();
        }

        _ => {}
    }
}
