use crate::github::*;

mod app;
mod project;
mod ui;
mod github;

fn main() {
    let app = app::App::setup();

    ui::start_app(app);

    //add_draft_issue(&cred.token, &ids[0].id, "test body", "test title");
    //
}
