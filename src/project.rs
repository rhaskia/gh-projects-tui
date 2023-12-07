use serde::{Deserialize, Serialize};
use serde_json::*;

#[derive(Debug, Deserialize)]
pub struct Project {
    info: ProjectInfo,
    fields: Fields,
    items: Items,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Projects {
    pub(crate) projects: Vec<ProjectInfo>,
    #[serde(rename = "totalCount")]
    total_count: u8,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProjectInfo {
    number: u8,
    url: String,
    #[serde(rename = "shortDescription")]
    short_description: String,
    public: bool,
    closed: bool,
    template: bool,
    id: String,
    pub title: String,
    readme: String,
    items: Count,
    fields: Count,
    owner: Owner,
}

#[derive(Debug, Deserialize)]
pub struct Count {
    #[serde(rename = "totalCount")]
    total_count: u8,
}

#[derive(Debug, Deserialize)]
pub struct Owner {
    login: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Fields {
    pub fields: Vec<Field>,
}

#[derive(Debug, Deserialize)]
pub struct Field {
    pub id: String,
    pub name: String,
    pub options: Option<Vec<FieldOption>>,
    //type: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FieldOption {
    id: String,
    pub(crate) name: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Items {
    pub(crate) items: Vec<Item>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Item {
    content: Content,
    id: String,
    #[serde(flatten)]
    pub(crate) fields: Map<String, Value>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Content {
    body: String,
    title: String,
}
