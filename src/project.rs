// TODO: fix on release
#![allow(dead_code)]
#![allow(unused_assignments)]
#![allow(unused_attributes)]
#![allow(unused_imports)]
#![allow(unused_macros)]
#![allow(unused_mut)]
#![allow(unused_variables)]

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::*;

#[derive(Debug, Deserialize)]
pub struct User {
    pub login: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: String,
    pub field_values: Nodes<ProjectV2ItemFieldValue>,
    pub content: Option<Content>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ProjectV2ItemFieldValue {
    ProjectV2ItemFieldTextValue {
        text: String,
        field: ProjectV2FieldCommon,
    },
    ProjectV2ItemFieldDateValue {
        date: String, // Assuming date is a string
        field: ProjectV2FieldCommon,
    },
    ProjectV2ItemFieldSingleSelectValue {
        name: String,
        field: ProjectV2FieldCommon,
    },
    Empty(Value), // Represents the empty field
}

impl Nodes<ProjectV2ItemFieldValue> {
    pub fn get_from_field(&self, s: &str) -> &str {
        use ProjectV2ItemFieldValue::*;

        let field = self.nodes.iter().find(|v| 
        match v {
            Empty(v) => "",
            ProjectV2ItemFieldTextValue { text, field } => &field.name, 
            ProjectV2ItemFieldDateValue { date, field } => &field.name,
            ProjectV2ItemFieldSingleSelectValue { name, field } => &field.name,
        } == s).unwrap_or(&Empty(Value::Null));

        match field {
            Empty(v) => "",
            ProjectV2ItemFieldTextValue { text, field } => &field.name, 
            ProjectV2ItemFieldDateValue { date, field } => &field.name,
            ProjectV2ItemFieldSingleSelectValue { name, field } => &field.name,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Content {
    DraftIssue {
        title: String,
        body: String,
    },
    Issue {
        title: String,
        assignees: Nodes<User>,
    },
    PullRequest {
        title: String,
        assignees: Nodes<User>,
    },
    Empty(Value),
}

#[derive(Debug, Deserialize)]
pub struct ProjectV2FieldCommon {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Nodes<T> {
    pub nodes: Vec<T>,
}

#[derive(Debug, Deserialize)]
pub struct Project {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Field {
    ProjectV2Field(ProjectV2Field),
    ProjectV2IterationField(ProjectV2IterationField),
    ProjectV2SingleSelectField(ProjectV2SingleSelectField),
    Empty,
}

impl Field {
    pub fn get_name(&self) -> &str {
        match self {
            Field::ProjectV2Field(pf) => &pf.name,
            Field::ProjectV2IterationField(pif) => &pif.name,
            Field::ProjectV2SingleSelectField(pssf) => &pssf.name,
            Field::Empty => "",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectV2Field {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "snake_case"))]
pub struct Iteration {
    pub start_date: String,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectV2IterationField {
    pub id: String,
    pub name: String,
    pub configuration: IterationConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IterationConfig {
    pub iterations: Vec<Iteration>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldOption {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectV2SingleSelectField {
    pub id: String,
    pub name: String,
    pub options: Vec<FieldOption>,
}

#[derive(Debug, Deserialize)]
pub struct Card {
    pub url: String,
    pub id: u32,
    pub note: String,
    pub creator: User,
}


// #[derive(Debug, Deserialize)]
// pub struct Project {
//     info: ProjectInfo,
//     fields: Fields,
//     items: Items,
// }
//
// #[derive(Debug, Deserialize)]
// #[serde(rename_all = "snake_case")]
// pub struct Projects {
//     pub(crate) projects: Vec<ProjectInfo>,
//     #[serde(rename = "totalCount")]
//     total_count: u8,
// }
//
// #[derive(Debug, Deserialize)]
// #[serde(rename_all = "snake_case")]
// pub struct ProjectInfo {
//     number: u8,
//     url: String,
//     #[serde(rename = "shortDescription")]
//     short_description: String,
//     public: bool,
//     closed: bool,
//     template: bool,
//     id: String,
//     pub title: String,
//     readme: String,
//     items: Count,
//     fields: Count,
//     owner: Owner,
// }
//
// #[derive(Debug, Deserialize)]
// pub struct Count {
//     #[serde(rename = "totalCount")]
//     total_count: u8,
// }
//
// #[derive(Debug, Deserialize)]
// pub struct Owner {
//     login: String,
// }
//
// #[derive(Debug, Deserialize)]
// pub(crate) struct Fields {
//     pub fields: Vec<Field>,
// }
//
// #[derive(Debug, Deserialize)]
// pub struct Field {
//     pub id: String,
//     pub name: String,
//     pub options: Option<Vec<FieldOption>>,
//     //type: String,
// }
//
// #[derive(Debug, Deserialize, Clone)]
// pub struct FieldOption {
//     id: String,
//     pub(crate) name: String,
// }
//
// #[derive(Debug, Deserialize)]
// pub(crate) struct Items {
//     pub(crate) items: Vec<Item>,
// }
//
// #[derive(Debug, Deserialize, Clone)]
// pub struct Item {
//     content: Content,
//     id: String,
//     #[serde(flatten)]
//     pub(crate) fields: HashMap<String, Value>,
// }
//
// #[derive(Debug, Deserialize, Clone)]
// pub struct Content {
//     body: String,
//     title: String,
// }
