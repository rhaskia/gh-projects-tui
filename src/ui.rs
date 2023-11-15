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
use crate::project::{Fields, Item};

pub(crate) fn draw(items: project::Items, fields: project::Fields, projects: project::Projects) -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let rows = get_rows(&items, &fields);
    let headers = get_headers(&fields);
    let widths = get_widths(&fields);

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
                projects.projects[0].title.clone(),
                Style::default().fg(Color::Green),
            ))
                .block(title_block);

            frame.render_widget(title, layout[0]);

            frame.render_widget(draw_table(rows.clone(), headers.clone()).widths(&widths), layout[1])
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

fn get_rows<'a>(items: &'a project::Items, fields: &'a project::Fields) -> Vec<Row<'a>> {
     items.items.iter().map(|item| {
        let cells = fields.fields.iter()
            .map(move |f|
                item.fields
                    .get(&*f.name.to_ascii_lowercase()).unwrap_or(&Value::Bool(false))
                    .as_str().unwrap_or(""))
            .collect::<Vec<&str>>();

        Row::new(cells).height(1).bottom_margin(0)
    }).collect()
}

fn get_headers(fields: &Fields) -> Row {
    let header_cells = fields.fields
        .iter()
        .map(|h| Cell::from(h.name.clone())
            .style(Style::default().fg(Color::Red)));

    Row::new(header_cells)
        .height(1)
        .bottom_margin(0)
}

fn get_widths(fields: &project::Fields) -> Vec<Constraint> {
    fields.fields
        .iter().map(|field|
        Constraint::Min(match &field.options {
            // has options
            Some(o) => o.iter()
                .fold(0,
                      |max, s|
                          if s.name.len() > max { s.name.len() }
                          else { max }),

            // pure string
            None => field.name.len(),
        } as u16 + 1)
    ).collect()
}

fn draw_table<'a>(rows: Vec<Row<'a>>, header: Row<'a>) -> Table<'a>{
    Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::TOP).title("Table"))
        .highlight_symbol(">> ")
}