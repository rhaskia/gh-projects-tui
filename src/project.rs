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
    pub field_values: Nodes<ProjectV2ItemField>,
    pub content: Option<Content>,
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
    Empty(Value), // Represents the empty field
}

<<<<<<< HEAD
impl ProjectV2ItemField {
    pub fn value(&self) -> &str {
        use ProjectV2ItemField::*;

        match self {
            Empty(v) => "",
            TextValue { text, field } => &text, 
            DateValue { date, field } => &date,
            SingleSelectValue { name, field } => &name,
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
        } == s).unwrap_or(&Empty(Value::Null))
    }

    pub fn name_from_field(&self, s: &str) -> &str {
        self.get_from_field(s).value()
=======
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
>>>>>>> main
    }
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

