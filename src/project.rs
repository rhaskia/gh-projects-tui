// TODO: fix on release
#![allow(dead_code)]
#![allow(unused_assignments)]
#![allow(unused_attributes)]
#![allow(unused_imports)]
#![allow(unused_macros)]
#![allow(unused_mut)]
#![allow(unused_variables)]

use std::{collections::HashMap, borrow::Cow};
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
    pub field_values: Nodes<ProjectV2ItemField>,
    pub content: Option<Content>,
}

#[derive(Debug, Deserialize)]
pub struct ItemMutation {
    id: String  
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ProjectV2ItemField {
    TextValue {
        text: String,
        field: ProjectV2FieldCommon,
    },
    DateValue {
        date: String, // Assuming date is a string
        field: ProjectV2FieldCommon,
    },
    SingleSelectValue {
        name: String,
        field: ProjectV2FieldCommon,
    },
    NumberValue {
        number: u16,
        field: ProjectV2FieldCommon,
    },
    IterationValue {
        duration: u8,
        title: String,
        field: ProjectV2FieldCommon,
    },
    Empty(Value), // Represents the empty field
}

impl ProjectV2ItemField {
    pub fn value(&self) -> &str {
        use ProjectV2ItemField::*;

        match self {
            Empty(v) => "",
            TextValue { text, field } => &text, 
            DateValue { date, field } => &date,
            SingleSelectValue { name, field } => &name, 
            NumberValue { number, field } => "6",
            IterationValue { duration, title, field } => &title, 
        }
    }
}

impl Nodes<ProjectV2ItemField> {
    pub fn get_from_field(&self, s: &str) -> &ProjectV2ItemField {
        use ProjectV2ItemField::*;

        self.nodes.iter().find(|v| 
        match v {
            Empty(v) => "",
            TextValue { text, field } => &field.name, 
            DateValue { date, field } => &field.name,
            SingleSelectValue { name, field } => &field.name,
            NumberValue { number, field } => &field.name,
            IterationValue { duration, title, field } => &field.name,
        } == s).unwrap_or(&Empty(Value::Null))
    }

    pub fn name_from_field(&self, s: &str) -> &str {
        self.get_from_field(s).value()
    }
}

#[derive(Debug, Deserialize)]
struct LabelConnection {
    //edges: Vec<LabelEdge>,
    nodes: Vec<Label>,
}

#[derive(Debug, Deserialize)]
struct Milestone {
    closed: bool,
    description: String,
}

#[derive(Debug, Deserialize)]
struct Label {
    color: String,
    description: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Content {
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
    Empty(Value),
}

impl Field {
    pub fn get_name(&self) -> &str {
        match self {
            Field::ProjectV2Field(pf)  => &pf.name,
            Field::ProjectV2IterationField(pf) => &pf.name,
            Field::ProjectV2SingleSelectField(pf) => &pf.name,
            Field::Empty(v) => "",
        }
    }

    pub fn get_id(&self) -> &str {
        match self {
            Field::ProjectV2Field(field) => &field.id,
            Field::ProjectV2IterationField(field) => &field.id,
            Field::ProjectV2SingleSelectField(field) =>  &field.id,
            Field::Empty(v) => "",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectV2Field {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectV2FieldCommon {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectV2IterationField {
    pub id: String,
    pub name: String,
    pub configuration: IterationConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectV2SingleSelectField {
    pub id: String,
    pub name: String,
    pub options: Vec<FieldOption>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "snake_case"))]
pub struct Iteration {
    pub start_date: String,
    pub id: String,
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

#[derive(Debug, Deserialize)]
pub struct Card {
    pub url: String,
    pub id: u32,
    pub note: String,
    pub creator: User,
}

