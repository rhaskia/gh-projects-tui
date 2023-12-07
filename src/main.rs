use crate::app::App;
use crate::project::{Fields, Items, Projects};

mod app;
mod commands;
mod project;
mod ui;

const FIELDS: &str = include_str!("fields.json");
const ITEMS: &str = include_str!("items.json");
const PROJECTS: &str = r#"{"projects":[{"number":2,"url":"https://github.com/users/rhaskia/projects/2","shortDescription":"","public":false,"closed":true,"template":false,"title":"@rhaskia's untitled project","id":"PVT_kwHOBaITmc4AXdY3","readme":"","items":{"totalCount":0},"fields":{"totalCount":8},"owner":{"type":"User","login":"rhaskia"}},{"number":1,"url":"https://github.com/users/rhaskia/projects/1","shortDescription":"","public":false,"closed":false,"template":false,"title":"Backrooms Survival","id":"PVT_kwHOBaITmc4AXboQ","readme":"","items":{"totalCount":19},"fields":{"totalCount":11},"owner":{"type":"User","login":"rhaskia"}}],"totalCount":2}"#;

fn main() {
    let fields = serde_json::from_str::<Fields>(FIELDS).unwrap().fields;
    let items = serde_json::from_str::<Items>(ITEMS).unwrap().items;
    let projects = serde_json::from_str::<Projects>(PROJECTS).unwrap().projects;

    let app = App::new(items, fields, projects);
    ui::draw(app).expect("UI Drawing Failed");
}
