use crate::github::*;

mod app;
mod project;
mod ui;
mod github;

fn main() {
    let cred = load_id();

    println!("{:?}", cred);
    let user = get_user(&cred.token).unwrap();
    let ids = get_project_ids(&cred.token, &user.login).unwrap();
    println!("{:?}, {:?}", ids, user);
    let fields = fetch_project_fields(&cred.token, &ids[2].id).unwrap(); 
    println!("fields {:?}\n", fields);
    let items = fetch_project_items(&cred.token, &ids[2].id).unwrap();
    println!("{:#?}", items);

    let app = app::App::setup();

    //add_draft_issue(&cred.token, &ids[0].id, "test body", "test title");
    //
}
