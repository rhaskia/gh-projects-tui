use std::fmt::{Error, format};
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{CrosstermBackend, Stylize, Terminal},
    widgets::Paragraph,
};
use std::io::{stdout, Result};
use ratatui::prelude::*;
use ratatui::widgets::*;

mod github_handler;
mod app;

use github_handler::*;
use serde_json::Value;

fn cleaned_items() -> Vec<Value> {
    let items_wrapper = serde_json::from_str::<Value>(&item_list()).unwrap();

    let items_object =
        if let Value::Object(o) = items_wrapper
        { o.get("items").unwrap().clone() }
        else { panic!("Unexpected Result"); };

    items_object.as_array().unwrap().clone()
}

fn cleaned_fields() -> Vec<Value> {
    let fields_wrapper = serde_json::from_str::<Value>(&field_list()).unwrap();

    let fields_object =
        if let Value::Object(o) = fields_wrapper
        { o.get("fields").unwrap().clone() }
        else { panic!("Unexpected Result"); };

    fields_object.as_array().unwrap().clone()
}

fn get_values(vec: Vec<Value>, key: &str) -> Vec<Value> {
    vec.iter()
        .map(|map| (*map.as_object().unwrap().get(key).unwrap()).clone())
        .collect()
}
