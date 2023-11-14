use serde_json::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Project {
    id: String,
    fields: Fields,
    items: Items,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Fields {
    fields: Vec<Field>,
}

#[derive(Debug, Deserialize)]
pub struct Field {
    id: String,
    name: String,
    options: Option<Vec<FieldOption>>,
    //type: String,
}

#[derive(Debug, Deserialize)]
pub struct FieldOption {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Items {
    items: Vec<Item>,
}

#[derive(Debug, Deserialize)]
pub struct Item {
    content: Content,
    id: String,
    title: String,
    #[serde(flatten)]
    fields: Map<String, Value>,
}

#[derive(Debug, Deserialize)]
pub struct Content {
    body: String,
    title: String,
}