use crate::app::{App, InputMode, normal_mode_keys, insert_mode_keys};
use crate::project::{Field, Item};

use crossterm::{
    event::{self, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*};
use serde_json::Value;
use std::cmp;
use std::io::{stdout, Result};

pub(crate) fn draw(mut app: App) -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    //let rows = get_rows(&app.items, &app.()fields);
    let n_widths = get_widths(&app.fields, &app.items);
    let headers = get_headers(&app, &n_widths);
    let mut offset = 0;
    let widths = constrained_widths(&n_widths);

    loop {
        terminal.draw(|frame| {
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
                app.projects[1].title.clone(),
                Style::default().fg(Color::Green),
            ))
            .block(title_block);

            frame.render_widget(title, layout[0]);

            offset = find_minimum_offset(&n_widths, app.column_state, layout[1].width - 10);

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
                    .select(app.column_state - offset)
                    .highlight_style(Style::new().red())
                    .divider("|"),
                scrolled.inner(&Margin::new(1, 0)),
            );

            // TODO: custom index list
            let list_state = ListState::default().with_selected(Some(app.item_state.clone()));

            for i in offset..app.fields.len() {
                frame.render_stateful_widget(
                    draw_list(&app.items, &app.fields, i)
                        .highlight_style(
                            if i == app.column_state { Style::reversed(Default::default()) }
                            else { Style::not_reversed(Default::default()) }),
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
                let mut position = lists_layout[app.column_state - offset].clone();

                if let Some(ref options) = app.input.current_options {

                    //if app.item_state

                    position.y = position.y + (app.item_state as u16); //- (app.input.current_option as u16);

                    position.x -= 1;
                    position.width += 1;
                    position.height = options.len() as u16;

                    let block = Block::new().borders(Borders::LEFT | Borders::RIGHT);

                    let option_names: Vec<ListItem> = options.iter().map(|n| ListItem::new(n.name.clone())).collect();

                    frame.render_widget(Clear, position);

                    frame.render_stateful_widget(List::new(option_names).block(block)
                                                     .highlight_style(Style::new().reversed()),
                                                 position, &mut state_wrapper(app.input.current_option));
                }
                else {
                    position.y = position.y + (app.item_state as u16);
                    position.height = 1;

                    frame.render_widget(Clear, position);


                    frame.render_widget(Paragraph::new(app.input.current_input.clone())
                                            .style(Style::red(Default::default())),
                                        position);

                    frame.set_cursor(position.x + app.input.cursor_pos, position.y);
                }
            }

            frame.render_widget(Paragraph::new(get_info_text(&app)), layout[2]);
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
    Ok(())
}

fn state_wrapper(i: usize) -> ListState {
    ListState::default().with_selected(Some(i))
}

fn find_minimum_offset(widths: &Vec<u16>, state: usize, max_width: u16) -> usize {
    for i in 0..widths.len() {
        if widths[i..state+1].iter().sum::<u16>() < max_width { return i; }
    }

    0
}


fn split_shift(v: &Vec<Constraint>, index: usize) -> Vec<Constraint> {
    let (before, after) = v.split_at(index);

    let mut new_vec = Vec::<Constraint>::new();
    new_vec.extend_from_slice(after);
    new_vec.extend_from_slice(before);

    new_vec
}

fn get_headers(app: &App, widths: &Vec<u16>) -> Vec<String> {
    (0..app.fields.len())
        .map(|i| {
            format!(
                "{:─<w$}",
                app.fields[i].name.clone(),
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

fn get_info_text(app: &App) -> String {
    format!("{}/{:?}: {:?}", app.column_state, app.item_state, app.items[app.item_state].fields.get(&app.fields[app.column_state].name))
}

fn get_column<'a>(items: &'a Vec<Item>, fields: &'a Vec<Field>, index: usize) -> Vec<ListItem<'a>> {
    let index_field = fields[index].name.to_ascii_lowercase();

    items
        .iter()
        .map(|item| {
            ListItem::new(
                item.fields
                    .get(&*index_field)
                    .unwrap_or(&Value::Bool(false))
                    .as_str()
                    .unwrap_or(""),
            )
        })
        .collect()
}

fn get_rows<'a>(items: &'a Vec<Item>, fields: &'a Vec<Field>) -> Vec<Row<'a>> {
    items
        .iter()
        .map(|item| {
            let cells = fields
                .iter()
                .map(move |f| {
                    item.fields
                        .get(&*f.name.to_ascii_lowercase())
                        .unwrap_or(&Value::Bool(false))
                        .as_str()
                        .unwrap_or("")
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
                field.name.len(),
                match &field.options {
                    // has options
                    Some(o) => o.iter().fold(0, |max, s| {
                        if s.name.len() > max {
                            s.name.len()
                        } else {
                            max
                        }
                    }),

                    // pure string
                    None => items.iter().fold(0, |max, i| {
                        let l = i
                            .fields
                            .get(&*field.name.to_ascii_lowercase())
                            .unwrap_or(&Value::Bool(false))
                            .as_str()
                            .unwrap_or("")
                            .len();
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
