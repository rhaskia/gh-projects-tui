use crate::github::*;

mod app;
mod project;
mod ui;
mod github;

fn main() {
    let app = app::App::setup();

    let err: Result<app::App, anyhow::Error> = ui::start_app(app);

    //println!("App failed at {:?}", err);
    println!("{:?}", err.unwrap().info().unwrap().items);

    // add_draft_issue(&cred.token, &ids[0].id, st body", "test title");
    // //
}
