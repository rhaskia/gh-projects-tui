use std::collections::HashMap;
use crate::app::App;
use crate::github::{get_project_ids, load_id, get_user, fetch_project_fields, fetch_project_items};
use crate::project::{Fields, Items, Projects};

mod app;
mod project;
mod ui;
mod github;

const FIELDS: &str = include_str!("fields.json");
const ITEMS: &str = include_str!("items.json");
const PROJECTS: &str = r#"{"projects":[{"number":2,"url":"https://github.com/users/rhaskia/projects/2","shortDescription":"","public":false,"closed":true,"template":false,"title":"@rhaskia's untitled project","id":"PVT_kwHOBaITmc4AXdY3","readme":"","items":{"totalCount":0},"fields":{"totalCount":8},"owner":{"type":"User","login":"rhaskia"}},{"number":1,"url":"https://github.com/users/rhaskia/projects/1","shortDescription":"","public":false,"closed":false,"template":false,"title":"Backrooms Survival","id":"PVT_kwHOBaITmc4AXboQ","readme":"","items":{"totalCount":19},"fields":{"totalCount":11},"owner":{"type":"User","login":"rhaskia"}}],"totalCount":2}"#;
use serde::{Deserialize, Serialize};

use reqwest::blocking::{Client, Response};


fn main() {
    let cred = load_id();

    println!("{:?}", cred);
    let user = get_user(&cred.token);
    let ids = get_project_ids(&cred.token, &user.login);
    println!("{:?}, {:?}", ids, user);
    let fields = fetch_project_fields(&cred.token, &ids[2].id); 
    //pfieldsrintln!("{:?}", fields);
    let items = fetch_project_items(&cred.token, &ids[2].id);
    println!("{:?}", items);
}
