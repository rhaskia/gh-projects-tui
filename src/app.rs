use crate::github;
use crate::project::*;
use ::time::{ext::NumericalDuration, Date};
use anyhow::anyhow;
use crossterm::event::{KeyCode, KeyEvent};
use github_device_flow::Credential;
use std::string::String;
use time::format_description;
use time::Duration;

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
    pub input: FieldBuffer,
}

#[derive(Debug)]
pub enum FieldBuffer {
    None,
    Text(String, u16),
    SingleSelect(Vec<FieldOption>, u16),
    Date(Date),
    Iteration(Vec<Iteration>, u16),
}

impl FieldBuffer {
    pub fn len(&self) -> usize {
        match self {
            FieldBuffer::Text(text, _) => text.len(),
            FieldBuffer::Date(_) => 6,
            _ => 0,
        }
    }
}

#[derive(Debug)]
pub struct UserInfo {
    pub user: User,
    pub items: Vec<Item>,
    pub fields: Vec<Field>,
    pub projects: Vec<Project>,
}

impl App {
    pub fn new() -> Self {
        App {
            project_state: 0,
            item_state: 0,
            field_state: 0,

            menu_state: InputMode::Normal,
            exit: false,

            user_info: None,
            id: None,

            input: FieldBuffer::None,
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
        if let FieldBuffer::SingleSelect(options, ref mut index) = &mut self.input {
            *index = match index {
                0 => options.len() as u16,
                _ => *index,
            } - 1;
        };
    }

    pub fn shift_option_down(&mut self) {
        if let FieldBuffer::SingleSelect(options, ref mut index) = &mut self.input {
            *index += 1;
            if *index >= options.len() as u16 {
                *index = 0;
            }
        };
    }

    pub fn begin_editing(&mut self) -> anyhow::Result<()> {
        if let Some(info) = &mut self.user_info {
            if !info.fields[self.field_state].is_editable() {
                return Ok(());
            }
            self.menu_state = InputMode::Input;

            let mut field_value = info.items[self.item_state]
                .field_values
                .get_from_field(&info.fields[self.field_state].get_name());

            // Empty item field
            if let ProjectV2ItemField::Empty(v) = field_value {
                info.items[self.item_state]
                    .field_values
                    .nodes
                    .push(info.fields[self.field_state].default());
                field_value = info.items[self.item_state]
                    .field_values
                    .get_from_field(&info.fields[self.field_state].get_name());
            }

            self.input = match field_value {
                ProjectV2ItemField::SingleSelectValue { name, field } => FieldBuffer::SingleSelect(
                    field.options.clone(),
                    field
                        .options
                        .iter()
                        .position(|x| x.name == field_value.value())
                        .unwrap() as u16,
                ),

                ProjectV2ItemField::IterationValue {
                    duration,
                    title,
                    field,
                } => FieldBuffer::Iteration(
                    field.configuration.iterations.clone(),
                    field
                        .configuration
                        .iterations
                        .iter()
                        .position(|x| x.title == field_value.value())
                        .unwrap() as u16,
                ),

                ProjectV2ItemField::TextValue { text, field } => {
                    FieldBuffer::Text(text.clone(), text.len() as u16)
                }

                ProjectV2ItemField::DateValue { date, field } => {
                    let format = format_description::parse("[year]-[month]-[day]")?;
                    FieldBuffer::Date(Date::parse(date, &format)?)
                }

                _ => FieldBuffer::None,
            };

            if let ProjectV2ItemField::DateValue { date, field: _ } = field_value {}
        }

        Ok(())
    }

    pub fn backspace(&mut self) {
        if let FieldBuffer::Text(ref mut text, ref mut cursor_pos) = self.input {
            if *cursor_pos == 0 {
                return;
            }

            *cursor_pos -= 1;
            text.remove(*cursor_pos as usize);
        }
    }

    pub fn insert_char(&mut self, c: char) {
        if let FieldBuffer::Text(ref mut text, ref mut cursor_pos) = &mut self.input {
            text.insert(*cursor_pos as usize, c);
            *cursor_pos += 1;
        }
    }

    pub fn cursor_left(&mut self) {
        if let FieldBuffer::Text(_, ref mut cursor_pos) = &mut self.input {
            if *cursor_pos != 0 {
                *cursor_pos -= 1;
            }
        }
    }

    pub fn cursor_right(&mut self) {
        if let FieldBuffer::Text(ref mut text, ref mut cursor_pos) = &mut self.input {
            if *cursor_pos != text.len() as u16 {
                *cursor_pos += 1;
            }
        }
    }

    pub fn save_field(&mut self) -> anyhow::Result<()> {
        if let Some(app_info) = &self.user_info {
            use ProjectV2ItemField::*;

            match self.get_field_at(self.item_state, self.field_state)? {
                Empty(v) => {}
                TextValue { text, field } => {
                    self.save_field_text()?;
                }
                DateValue { date, field } => {
                    self.save_field_date()?;
                }
                SingleSelectValue { name, field } => {
                    self.save_field_option()?;
                }
                NumberValue { number, field } => {
                    self.save_field_number()?;
                }
                IterationValue {
                    duration,
                    title,
                    field,
                } => {}
            };
        }

        Ok(())
    }

    pub fn save_field_option(&mut self) -> anyhow::Result<()> {
        if let Some(app_info) = &self.user_info {
            if let FieldBuffer::SingleSelect(options, index) = &self.input {
                let current_option = options[*index as usize].clone();

                let mutation = github::update_item_option(
                    &self
                        .id
                        .clone()
                        .ok_or_else(|| anyhow!("No Credential found"))?
                        .token,
                    &app_info.projects[self.project_state].id,
                    &app_info.items[self.item_state].id,
                    app_info.fields[self.field_state].get_id(),
                    &current_option.id,
                )?;

                return self.set_field_at(self.item_state, self.field_state, &current_option.name);
            }
        }

        Ok(())
    }

    pub fn save_field_number(&mut self) -> anyhow::Result<()> {
        if let Some(app_info) = &self.user_info {
            if let FieldBuffer::Text(text, cursor) = &self.input {
                let mutation = github::update_item_number(
                    &self
                        .id
                        .clone()
                        .ok_or_else(|| anyhow!("No Credential found"))?
                        .token,
                    &app_info.projects[self.project_state].id,
                    &app_info.items[self.item_state].id,
                    app_info.fields[self.field_state].get_id(),
                    text.parse()?,
                )?;

                return self.set_field_at(self.item_state, self.field_state, &text.clone());
            }
        }

        Ok(())
    }

    pub fn save_field_date(&mut self) -> anyhow::Result<()> {
        if let Some(app_info) = &self.user_info {
            if let FieldBuffer::Date(date) = self.input {
                let format = format_description::parse("[year]-[month]-[day]")?;
                let format_date = date.format(&format)?;

                let mutation = github::update_item_date(
                    &self
                        .id
                        .clone()
                        .ok_or_else(|| anyhow!("No Credential found"))?
                        .token,
                    &app_info.projects[self.project_state].id,
                    &app_info.items[self.item_state].id,
                    app_info.fields[self.field_state].get_id(),
                    &format_date,
                )?;

                return self.set_field_at(self.item_state, self.field_state, &format_date);
            }
        }

        Ok(())
    }

    pub fn save_field_text(&mut self) -> anyhow::Result<()> {
        if let Some(app_info) = &self.user_info {
            if let FieldBuffer::Text(text, cursor_pos) = &self.input {
                let mutation = github::update_item_text(
                    &self
                        .id
                        .clone()
                        .ok_or_else(|| anyhow!("No Credential found"))?
                        .token,
                    &app_info.projects[self.project_state].id,
                    &app_info.items[self.item_state].id,
                    app_info.fields[self.field_state].get_id(),
                    &text,
                )?;

                return self.set_field_at(self.item_state, self.field_state, &text.clone());
            }
        }

        Ok(())
    }

    pub fn set_field_at(&mut self, item: usize, field: usize, s: &str) -> anyhow::Result<()> {
        let info = self.info()?;
        let index = info.fields[field].get_name().to_string();
        let mut_info = self.mut_info()?;

        mut_info.items[item].field_values.set_value(&index, s);

        Ok(())
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

    pub fn shift_date(&mut self, d: i64) {
        if let FieldBuffer::Date(ref mut date) = &mut self.input {
            if let Some(new_date) = date.checked_add(Duration::days(d)) {
                *date = new_date;
            }
        }
    }

    pub fn shift_month_forward(&mut self) {
        if let FieldBuffer::Date(ref mut date) = &mut self.input {
            if let Ok(new_date) = date.replace_month(date.month().next()) {
                *date = new_date;
            }
        }
    }

    pub fn shift_month_back(&mut self) {
        if let FieldBuffer::Date(ref mut date) = &mut self.input {
            if let Ok(new_date) = date.replace_month(date.month().previous()) {
                *date = new_date;
            }
        }
    }

    pub fn shift_year_forward(&mut self) {
        if let FieldBuffer::Date(ref mut date) = &mut self.input {
            if let Ok(new_date) = date.replace_year(date.year() + 1) {
                *date = new_date;
            }
        }
    }

    pub fn shift_year_back(&mut self) {
        if let FieldBuffer::Date(ref mut date) = &mut self.input {
            if let Ok(new_date) = date.replace_year(date.year() - 1) {
                *date = new_date;
            }
        }
    }
}

pub fn insert_mode_keys(key: KeyEvent, app: &mut App) -> anyhow::Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.menu_state = InputMode::Normal;
            app.input = FieldBuffer::None;
        }
        _ => {}
    }

    if let Some(app_info) = &app.user_info {
        use ProjectV2ItemField::*;

        match app.get_field_at(app.item_state, app.field_state)? {
            // Single select editing
            SingleSelectValue { name, field } => match key.code {
                KeyCode::Char('j') | KeyCode::Down => app.shift_option_down(),
                KeyCode::Char('k') | KeyCode::Up => app.shift_option_up(),

                KeyCode::Enter => {
                    app.save_field()?;
                    app.menu_state = InputMode::Normal;
                    app.input = FieldBuffer::None;
                }

                _ => {}
            },

            // Text editing
            TextValue { text, field } => match key.code {
                KeyCode::Char(a) => app.insert_char(a),
                KeyCode::Backspace => app.backspace(),

                KeyCode::Enter => {
                    app.save_field()?;
                    app.menu_state = InputMode::Normal;
                    app.input = FieldBuffer::None;
                }

                KeyCode::Left => app.cursor_left(),
                KeyCode::Right => app.cursor_right(),

                _ => {}
            },

            // Date editing, uses calendar widget
            DateValue { date, field } => match key.code {
                KeyCode::Left | KeyCode::Char('h') => app.shift_date(1),
                KeyCode::Right | KeyCode::Char('l') => app.shift_date(-1),
                KeyCode::Up | KeyCode::Char('k') => app.shift_date(-7),
                KeyCode::Down | KeyCode::Char('j') => app.shift_date(7),

                KeyCode::Char('J') => app.shift_month_forward(),
                KeyCode::Char('K') => app.shift_month_back(),

                KeyCode::Char('L') => app.shift_year_forward(),
                KeyCode::Char('H') => app.shift_year_back(),

                KeyCode::Enter => {
                    app.save_field()?;
                    app.menu_state = InputMode::Normal;
                }

                _ => {}
            },

            NumberValue { number, field } => match key.code {
                KeyCode::Char(a) => {
                    if a.is_numeric() {
                        app.insert_char(a);
                    }
                }
                KeyCode::Backspace => app.backspace(),

                KeyCode::Enter => {
                    app.save_field()?;
                    app.menu_state = InputMode::Normal;
                    app.input = FieldBuffer::None;
                }

                KeyCode::Left => app.cursor_left(),
                KeyCode::Right => app.cursor_right(),

                _ => {}
            },
            IterationValue {
                duration,
                title,
                field,
            } => todo!(),
            Empty(_) => todo!(),
        }
    }

    Ok(())
}

pub fn normal_mode_keys(key: KeyEvent, app: &mut App) -> anyhow::Result<()> {
    Ok(match key.code {
        KeyCode::Char('q') => app.exit = true,

        KeyCode::Char('j') | KeyCode::Down => app.next(),
        KeyCode::Char('k') | KeyCode::Up => app.previous(),
        KeyCode::Char('h') | KeyCode::Left => app.left(),
        KeyCode::Char('l') | KeyCode::Right => app.right(),

        KeyCode::Char('i') => app.begin_editing()?,

        _ => {}
    })
}
