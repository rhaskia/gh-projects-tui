use crate::app::{insert_mode_keys, normal_mode_keys, App, FieldBuffer, InputMode};
use crate::project::{Field, Item, ProjectV2ItemField};
use std::rc::Rc;

use crossterm::{
    event::{self, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use github_device_flow::{Credential, DeviceFlow, DeviceFlowError};
use ratatui::widgets::calendar::CalendarEventStore;
use ratatui::{prelude::*, widgets::*};
use std::io::stdout;
use std::result::Result;
use std::{cmp, fs, vec};

type CTerminal = Terminal<CrosstermBackend<std::io::Stdout>>;

pub fn disable_terminal() -> anyhow::Result<()> {
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}

pub fn draw_app(mut app: App) -> anyhow::Result<App> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    // Auth and load info
    let cred = draw_auth(&mut terminal)?;

    let _ = fs::write(
        "./access_token",
        serde_json::to_string(&cred).expect("Failed to serialize"),
    );

    app.id = Some(cred);
    app.reload_info()?;

    // Actual UI once loaded
    draw_projects_editor(&mut app, &mut terminal)?;

    //disable_terminal()?;

    Ok(app)
}

pub fn draw_auth(terminal: &mut CTerminal) -> Result<Credential, DeviceFlowError> {
    if let Ok(content) = fs::read_to_string("./access_token") {
        if let Ok(cred) = serde_json::from_str(&content) {
            return Ok(cred);
        }
    }

    let client_id = include_str!("client_id");
    let scope = Some("project,user");
    let host = Some("github.com");

    let mut flow = DeviceFlow::start(client_id, host, scope)?;

    terminal
        .draw(|frame| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Percentage(40),
                    Constraint::Min(4),
                    Constraint::Percentage(40),
                ])
                .split(frame.size());

            let text = format!(
                "Please visit https://github.com/login/device \nAnd paste in the code {}",
                flow.user_code.as_ref().unwrap()
            );

            frame.render_widget(
                Paragraph::new(text).block(Block::default().borders(Borders::ALL)),
                layout[1],
            );
        })
        .expect("Auth rendering failed");

    //thread::sleep(time::Duration::new(1, 0));

    flow.poll(20)
}

// pub fn draw_error(, error: anyhow::Error) -> anyhow::Result<()> {
//     let err = format!("{:#?}", error);
//
//     terminal.draw(|frame| {
//         let layout = Layout::default()
//             .direction(Direction::Vertical)
//             .constraints(vec![
//                 Constraint::Min(0),
//                 Constraint::Max(5),
//                 Constraint::Min(0),
//             ])
//             .split(frame.size());
//
//         frame.render_widget(Paragraph::new(err), layout[1]);
//     })?;
//
//     Ok(())
// }

pub(crate) fn draw_projects_editor(
    mut app: &mut App,
    terminal: &mut CTerminal,
) -> anyhow::Result<()> {
    //let rows = get_rows(&app.items, &app.()fields);
    let mut n_widths;
    let mut widths;
    let mut headers;
    let mut offset = 0;

    loop {
        n_widths = get_widths(&app, &app.info()?.fields, &app.info()?.items);
        widths = constrained_widths(&n_widths);
        headers = get_headers(&app.info()?.fields, &n_widths);

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

            let scrolled = layout[1].clone();

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

            let cursor_pos = layout[1].height.min(app.item_state as u16 + 3);
            frame.render_widget(Paragraph::new(">"), Rect::new(0, cursor_pos, 1, 1));

            if app.menu_state == InputMode::Input {
                draw_editor(frame, &app, &lists_layout, offset);
            }

            frame.render_widget(guide(&app), layout[2]);
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match &app.menu_state {
                        InputMode::Normal => normal_mode_keys(key, &mut app)?,
                        _ => insert_mode_keys(key, &mut app)?,
                    };
                }
            }
        }

        if app.exit {
            return Ok(());
        }
    }
}

fn draw_editor(
    frame: &mut Frame,
    app: &App,
    lists_layout: &Rc<[Rect]>,
    offset: usize,
) -> anyhow::Result<()> {
    let mut position = lists_layout[app.field_state - offset].clone();

    use ProjectV2ItemField::*;
    match app.get_field_at(app.item_state, app.field_state)? {
        // Pure Text
        TextValue { text: _, field } | NumberValue { number: _, field } => {
            position.y = position.y + (app.item_state as u16);
            position.height = 1;

            frame.render_widget(Clear, position);

            if let FieldBuffer::Text(text, cursor_pos) = &app.input {
                frame.render_widget(
                    Paragraph::new(text.clone()).style(Style::default().italic()),
                    position,
                );

                frame.set_cursor(position.x + cursor_pos, position.y);
            }
        }

        // With options
        SingleSelectValue { name: _, field: _ } => {
            if let Field::ProjectV2SingleSelectField(field) = &app.info()?.fields[app.field_state] {
                position.y = position.y + (app.item_state as u16); //- (app.input.current_option as u16);

                position.x -= 1;
                position.width += 1;
                position.height = field.options.len() as u16;

                let block = Block::new().borders(Borders::LEFT | Borders::RIGHT);

                let option_names: Vec<ListItem> = field
                    .options
                    .iter()
                    .map(|n| ListItem::new(n.name.clone()).style(n.style()))
                    .collect();

                if let FieldBuffer::SingleSelect(options, index) = &app.input {
                    frame.render_widget(Clear, position);

                    frame.render_stateful_widget(
                        List::new(option_names)
                            .block(block)
                            .highlight_style(Style::new().reversed()),
                        position,
                        &mut state_wrapper(*index as usize),
                    );
                }
            }
        }

        //Whatever
        IterationValue {
            duration,
            title,
            field,
        } => {}

        // Date, calendar widget?
        DateValue { date, field } => {
            if let FieldBuffer::Date(date) = app.input {
                let mut events = CalendarEventStore::default();
                events.add(date, Style::default().on_red());

                let calendar_widget = calendar::Monthly::new(date, events)
                    .show_month_header(Style::default())
                    .show_weekdays_header(Style::new().italic())
                    .show_surrounding(Style::new().gray())
                    .block(Block::new().borders(Borders::all()));

                position.height = 10;
                position.width = 24;
                position.x -= cmp::min(position.x, 6);
                position.y = position.y + (app.item_state as u16); //- (app.input.current_option as u16);
                frame.render_widget(Clear, position);

                frame.render_widget(calendar_widget, position);
            }
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

fn guide<'a>(app: &App) -> Table<'a> {
    let rows_raw = match app.menu_state {
        InputMode::Input => insert_mode_guide(),
        InputMode::Normal => normal_mode_guide(),
    };

    let rows = rows_raw.iter().map(|r| {
        Row::new(
            r.iter()
                .map(|i| Line::from(vec![i.0.clone().bold(), Span::from(i.1.clone())])),
        )
    });

    Table::new(rows)
}

fn insert_mode_guide() -> Vec<Vec<(String, String)>> {
    vec![
        vec![
            (String::from("q"), String::from("quit")),
            (String::from("i"), String::from("insert")),
            (String::from("r"), String::from("add new")),
            (String::from("h"), String::from("left")),
            (String::from("l"), String::from("right")),
        ],
        vec![
            (String::from("p"), String::from("switch project")),
            (String::from("x"), String::from("hide field")),
            (String::from("s"), String::from("sort by")),
            (String::from("k"), String::from("up")),
            (String::from("j"), String::from("down")),
        ],
    ]
}

fn normal_mode_guide() -> Vec<Vec<(String, String)>> {
    vec![
        vec![
            (String::from("q"), String::from("quit")),
            (String::from("i"), String::from("insert")),
            (String::from("a"), String::from("add new")),
            (String::from("h"), String::from("left")),
            (String::from("l"), String::from("right")),
        ],
        vec![
            (String::from("p"), String::from("switch project")),
            (String::from("x"), String::from("hide field")),
            (String::from("s"), String::from("sort by")),
            (String::from("k"), String::from("up")),
            (String::from("j"), String::from("down")),
        ],
    ]
}

fn get_column<'a>(items: &'a Vec<Item>, fields: &'a Vec<Field>, index: usize) -> Vec<ListItem<'a>> {
    items
        .iter()
        .map(|item| {
            let item = item.field_values.get_from_field(fields[index].get_name());
            ListItem::new(item.value()).style(item.style())
        })
        .collect()
}

fn constrained_widths(i: &Vec<u16>) -> Vec<Constraint> {
    i.iter().map(|f| Constraint::Min(*f)).collect()
}

fn get_widths(app: &App, fields: &Vec<Field>, items: &Vec<Item>) -> Vec<u16> {
    fields
        .iter()
        .map(|field| get_width(app, field, fields, items) as u16 + 1)
        .collect()
}

fn get_width(app: &App, field: &Field, fields: &Vec<Field>, items: &Vec<Item>) -> usize {
    let currently_editing = field.get_id() == fields[app.field_state].get_id();

    let max = cmp::max(
        field.get_name().len(),
        match &field {
            Field::ProjectV2SingleSelectField(field) => field.options.iter().fold(0, |max, s| {
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
    );

    if currently_editing {
        cmp::max(max, app.input.len())
    } else {
        max
    }
}

fn draw_table<'a>(rows: Vec<Row<'a>>, header: Row<'a>) -> Table<'a> {
    Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(Style::new().light_yellow())
}
