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

fn main() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let fields = get_values(cleaned_fields());

    loop {
        terminal.draw(|frame| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Length(3),
                    Constraint::Max(1000),
                    Constraint::Length(3),
                ])
                .split(frame.size());

            let title_block = Block::default()
                .borders(Borders::ALL)
                .style(Style::default());

            let title = Paragraph::new(Text::styled(
                "Create New Json",
                Style::default().fg(Color::Green),
            ))
                .block(title_block);

            frame.render_widget(title, layout[0]);

            //frame.render_widget(draw_table(cleaned_items(), cleaned_fields()), layout[1])
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn get_values(vec: Vec<Value>, key: &str) -> Vec<Value> {
    vec.iter()
        .map(|map| (*map.as_object().unwrap().get(key).unwrap()).clone())
        .collect()
}
fn draw_table(items: Vec<Value>, fields: Vec<Value>) -> Table<'static> {
    let field_names = get_values(fields, "name");

    let header_cells = field_names
        .iter()
        .map(|h| Cell::from((*h.as_str().unwrap()).to_owned())
        .style(Style::default().fg(Color::Red)));

    let header = Row::new(header_cells)
        .height(1)
        .bottom_margin(0);

    let rows = (0..5).map(|item| {
        let cells = (2..8).map(|c| Cell::from(format!("{c}")));
        Row::new(cells).height(1).bottom_margin(1)
    });

    Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Table"))
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Min(8),
            Constraint::Min(8),
            Constraint::Min(8),
            Constraint::Min(8),
            Constraint::Min(8),
        ])
}