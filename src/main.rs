use crossterm::terminal::disable_raw_mode;

use crate::github::*;

mod app;
mod github;
mod project;
mod ui;

fn main() {
    let app = app::App::setup();

    let t = ui::start_app(app);
    disable_raw_mode();
    println!("{:?}", t);

    //add_draft_issue(&cred.token, &ids[0].id, "test body", "test title");
}
