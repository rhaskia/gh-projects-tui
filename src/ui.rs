// TODO: fix on release
#![allow(dead_code)]
#![allow(unused_assignments)]
#![allow(unused_attributes)]
#![allow(unused_imports)]
#![allow(unused_macros)]
#![allow(unused_mut)]
#![allow(unused_variables)]

use crate::app::{insert_mode_keys, normal_mode_keys, App, InputMode};
use crate::github::load_id;
use crate::project::{Field, Item, ProjectV2ItemField};
use std::rc::Rc;

use anyhow::anyhow;
use crossterm::{
    event::{self, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*};
use serde_json::Value;
use std::cmp;
use std::io::{stdout, Result};

pub fn start_app(mut app: App) -> anyhow::Result<App> {
    // get id
    app.id = Some(load_id());

    // Load user info and load it into the App
    let err = app.reload_info();

    println!("{:?}, \n{:?}", err, app.user_info);

    // Actual UI once loaded
    let app = draw(app)?;

    println!("something went wrong ):");

    Ok(app)
}

pub(crate) fn draw(mut app: App) -> anyhow::Result<App> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    //let rows = get_rows(&app.items, &app.()fields);
    let n_widths = get_widths(&app.info()?.fields, &app.info()?.items);
    let headers = get_headers(&app.info()?.fields, &n_widths);
    let mut offset = 0;
    let widths = constrained_widths(&n_widths);

    loop {
        terminal.draw(|frame| {
            // TODO cut this ugly closure up

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Length(2),
                    Constraint::Min(5),
                    Constraint::Length(2),
                ])
                .split(frame.size());

            let title_block = Block::default()
                .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
                .style(Style::default());

            let title = Paragraph::new(Text::styled(
                app.info().unwrap().projects[1].title.clone(),
                Style::default().fg(Color::Green),
            ))
            .block(title_block);

            frame.render_widget(title, layout[0]);

            offset = find_minimum_offset(&n_widths, app.field_state, layout[1].width - 10);

            // Layout for Lists
            let lists_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(split_shift(&widths, offset))
                .split(layout[1].inner(&Margin::new(1, 1)));

            // Draw List Border
            let border_set = symbols::border::Set {
                top_right: symbols::line::NORMAL.vertical_left,
                top_left: symbols::line::NORMAL.vertical_right,
                ..symbols::border::PLAIN
            };

            frame.render_widget(
                Block::new().borders(Borders::ALL).border_set(border_set),
                layout[1],
            );

            let mut scrolled = layout[1].clone();

            // Tabs Drawing
            frame.render_widget(
                Tabs::new(headers[offset..].to_owned())
                    .padding("", "")
                    .select(app.field_state - offset)
                    .highlight_style(Style::new().red())
                    .divider("|"),
                scrolled.inner(&Margin::new(1, 0)),
            );

            // TODO: custom index list
            let list_state = ListState::default().with_selected(Some(app.item_state.clone()));

            for i in offset..app.info().unwrap().fields.len() {
                frame.render_stateful_widget(
                    draw_list(&app.info().unwrap().items, &app.info().unwrap().fields, i)
                        .highlight_style(if i == app.field_state {
                            Style::reversed(Default::default())
                        } else {
                            Style::not_reversed(Default::default())
                        }),
                    lists_layout[i - offset],
                    &mut list_state.clone(),
                );
            }

            // Wrapped Items
            // for i in 0..offset {
            //     frame.render_stateful_widget(
            //         draw_list(&app.items, &app.fields, i)
            //             .highlight_style(Style::not_reversed(Default::default())),
            //         lists_layout[app.fields.len() - offset + i],
            //         &mut list_state.clone(),
            //     );
            // }

            let cursor_pos = layout[1].height.min(app.item_state as u16 + 3);
            frame.render_widget(Paragraph::new(">"), Rect::new(0, cursor_pos, 1, 1));

            if app.menu_state == InputMode::Input {
                draw_editor(frame, &app, &lists_layout, offset);
            }

            frame.render_widget(
                Paragraph::new(get_info_text(
                    &app,
                    &app.info().unwrap().items,
                    &app.info().unwrap().fields,
                )),
                layout[2],
            );
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match &app.menu_state {
                        InputMode::Normal => normal_mode_keys(key, &mut app),
                        _ => insert_mode_keys(key, &mut app),
                    }
                }
            }
        }

        if app.exit {
            break;
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(app)
}

fn draw_editor(frame: &mut Frame, app: &App, lists_layout: &Rc<[Rect]>, offset: usize) -> anyhow::Result<()> {
    let mut position = lists_layout[app.field_state - offset].clone();

    use ProjectV2ItemField::*;
    match app.get_field_at(app.item_state, app.field_state)? {
        // Pure Text
        ProjectV2ItemField::TextValue { text, field } => {
            position.y = position.y + (app.item_state as u16);
            position.height = 1;

            frame.render_widget(Clear, position);

            frame.render_widget(
                Paragraph::new(app.input.current_input.clone()).style(Style::red(Default::default())),
                position,
            );

            frame.set_cursor(position.x + app.input.cursor_pos, position.y);
        }

        // With options
        SingleSelectValue { name, field } => {
            if let Field::ProjectV2SingleSelectField(full_field) = &app.info()?.fields[app.field_state] {
                position.y = position.y + (app.item_state as u16); //- (app.input.current_option as u16);

                position.x -= 1;
                position.width += 1;
                position.height = full_field.options.len() as u16;

                let block = Block::new().borders(Borders::LEFT | Borders::RIGHT);

                let option_names: Vec<ListItem> = full_field.options 
                    .iter()
                    .map(|n| ListItem::new(n.name.clone()))
                    .collect();

                frame.render_widget(Clear, position);

                frame.render_stateful_widget(
                    List::new(option_names)
                        .block(block)
                        .highlight_style(Style::new().reversed()),
                    position,
                    &mut state_wrapper(app.input.current_option),
                );
            }
        }

        // Date, calendar widget?
        DateValue { date, field } => {

        }

        // Ignore
        Empty(v) => {}
    }

    Ok(())
}

fn state_wrapper(i: usize) -> ListState {
    ListState::default().with_selected(Some(i))
}

fn find_minimum_offset(widths: &Vec<u16>, state: usize, max_width: u16) -> usize {
    for i in 0..widths.len() {
        if widths[i..state + 1].iter().sum::<u16>() < max_width {
            return i;
        }
    }

    0
}

// [1, 2, 3, 4], split at 2 => [3, 4, 1, 2]
// for wrapping fields around, no longer in use due to being tacky
fn split_shift(v: &Vec<Constraint>, index: usize) -> Vec<Constraint> {
    let (before, after) = v.split_at(index);

    let mut new_vec = Vec::<Constraint>::new();
    new_vec.extend_from_slice(after);
    new_vec.extend_from_slice(before);

    new_vec
}

fn get_headers(fields: &Vec<Field>, widths: &Vec<u16>) -> Vec<String> {
    (0..fields.len())
        .map(|i| {
            format!(
                "{:─<w$}",
                fields[i].get_name(),
                w = (widths[i] as usize - 1)
            )
        })
        .collect()
}

fn draw_list<'a>(items: &'a Vec<Item>, fields: &'a Vec<Field>, index: usize) -> List<'a> {
    List::new(get_column(items, fields, index))
        .block(Block::default())
        .highlight_style(Style::new().reversed())
}

fn get_info_text(app: &App, items: &Vec<Item>, fields: &Vec<Field>) -> String {
    format!(
        "{}/{:?}: {:?}",
        app.field_state,
        app.item_state,
        items[app.item_state]
            .field_values
            .get_from_field(fields[app.field_state].get_name())
    )
}

fn get_column<'a>(items: &'a Vec<Item>, fields: &'a Vec<Field>, index: usize) -> Vec<ListItem<'a>> {
    items
        .iter()
        .map(|item| ListItem::new(item.field_values.name_from_field(fields[index].get_name())))
        .collect()
}

fn get_rows<'a>(items: &'a Vec<Item>, fields: &'a Vec<Field>) -> Vec<Row<'a>> {
    items
        .iter()
        .map(|item| {
            let cells = fields
                .iter()
                .map(move |f| {
                    item.field_values
                        .name_from_field(&*f.get_name().to_ascii_lowercase())
                })
                .collect::<Vec<&str>>();

            Row::new(cells).height(1).bottom_margin(0)
        })
        .collect()
}

fn constrained_widths(i: &Vec<u16>) -> Vec<Constraint> {
    i.iter().map(|f| Constraint::Min(*f)).collect()
}

fn get_widths(fields: &Vec<Field>, items: &Vec<Item>) -> Vec<u16> {
    fields
        .iter()
        .map(|field| {
            cmp::max(
                field.get_name().len(),
                match &field {
                    Field::ProjectV2SingleSelectField(pf) => pf.options.iter().fold(0, |max, s| {
                        if s.name.len() > max {
                            s.name.len()
                        } else {
                            max
                        }
                    }),

                    // has options

                    // pure string
                    _ => items.iter().fold(0, |max, i| {
                        let l = i.field_values.name_from_field(&*field.get_name()).len();
                        if l > max {
                            l
                        } else {
                            max
                        }
                    }),
                },
            ) as u16
                + 1
        })
        .collect()
}

fn draw_table<'a>(rows: Vec<Row<'a>>, header: Row<'a>) -> Table<'a> {
    Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(Style::new().light_yellow())
}
