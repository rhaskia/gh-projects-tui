use std::cmp;
use std::fmt::{Error, format};
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::*,
};
use std::io::{stdout, Result};
use crate::project;
use serde_json::Value;
use crate::project::{Field, Item, ProjectInfo};
use crate::app::App;

pub(crate) fn draw(mut app: App) -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let rows = get_rows(&app.items, &app.fields);
    let headers = get_headers(&app.fields);
    let widths = get_widths(&app.fields, &app.items);

    loop {
        terminal.draw(|frame| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Length(2),
                    Constraint::Min(5),
                    Constraint::Length(3),
                ])
                .split(frame.size());

            let title_block = Block::default()
                .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
                .style(Style::default());

            let title = Paragraph::new(Text::styled(
                app.projects[1].title.clone(),
                Style::default().fg(Color::Green),
            ))
                .block(title_block);

            frame.render_widget(title, layout[0]);

            let lists_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(widths.clone())
                .split(layout[1].inner(&Margin::new(1, 1)));

            let border_set = symbols::border::Set {
                top_right: symbols::line::NORMAL.vertical_left,
                top_left: symbols::line::NORMAL.vertical_right,
                ..symbols::border::PLAIN
            };

            frame.render_widget(Block::new().borders(Borders::ALL).border_set(border_set), layout[1]);

            // TODO: custom index list
            for i in 0..app.fields.len() {
                if i == app.column_state {
                    frame.render_stateful_widget(draw_list(&app.items, &app.fields, i),
                                                 lists_layout[i], &mut app.item_state);
                }
                else {
                    frame.render_widget(draw_list(&app.items, &app.fields, i), lists_layout[i]);
                }
            }
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if key.code == KeyCode::Char('q') { break; }

                    if key.code == KeyCode::Char('j') { app.next(); }
                    if key.code == KeyCode::Char('k') { app.previous(); }
                    if key.code == KeyCode::Char('h') { app.left(); }
                    if key.code == KeyCode::Char('l') { app.right(); }
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn draw_list<'a>(items: &'a Vec<Item>, fields: &'a Vec<Field>, index: usize) -> List<'a> {
    List::new(get_column(items, fields, index))
        .block(Block::default())
        .highlight_style(Style::new().black().on_yellow())
}

fn get_column<'a>(items: &'a Vec<Item>, fields: &'a Vec<Field>, index: usize) -> Vec<ListItem<'a>> {
    let index_field = fields[index].name.to_ascii_lowercase();

    let mut c = items.iter()
        .map(|item|
            ListItem::new(item.fields.get(&*index_field)
                .unwrap_or(&Value::Bool(false))
                .as_str().unwrap_or("")))
        .collect::<Vec<ListItem<'a>>>();
    
    c.insert(0, ListItem::new(fields[index].name.clone()));
    c
}

fn get_rows<'a>(items: &'a Vec<Item>, fields: &'a Vec<Field>) -> Vec<Row<'a>> {
     items.iter().map(|item| {
        let cells = fields.iter()
            .map(move |f|
                item.fields
                    .get(&*f.name.to_ascii_lowercase()).unwrap_or(&Value::Bool(false))
                    .as_str().unwrap_or(""))
            .collect::<Vec<&str>>();

        Row::new(cells).height(1).bottom_margin(0)
    }).collect()
}

fn get_headers(fields: &Vec<Field>) -> Row {
    let header_cells = fields
        .iter()
        .map(|h| Cell::from(h.name.clone())
            .style(Style::default().fg(Color::Red)));

    Row::new(header_cells)
        .height(1)
        .bottom_margin(0)
}

fn get_widths(fields: &Vec<Field>, items: &Vec<Item>) -> Vec<Constraint> {
    fields
        .iter().map(|field|
        Constraint::Min(
        cmp::max(field.name.len(),
        match &field.options {
            // has options
            Some(o) => o.iter()
                .fold(0,
                      |max, s|
                          if s.name.len() > max { s.name.len() }
                          else { max }),

            // pure string
            None => items.iter()
                .fold(0,
                    |max, i| {
                    let l = i.fields.get(&*field.name.to_ascii_lowercase())
                        .unwrap_or(&Value::Bool(false))
                        .as_str().unwrap_or("")
                        .len();
                    if l > max { l } else { max }}),
        }) as u16 + 1)
    ).collect()
}

fn draw_table<'a>(rows: Vec<Row<'a>>, header: Row<'a>) -> Table<'a>{
    Table::new(rows)
        .header(header)
        .block(Block::default()
        .borders(Borders::ALL))
        .highlight_style(Style::new().light_yellow())
}