use serde::{Deserialize, Serialize};
use serde_json::*;
use ratatui::style::{Style, Stylize};


#[derive(Debug, Deserialize)]
pub struct User {
    pub login: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: String,
    #[serde(default)]
    pub field_values: Nodes<ProjectV2ItemField>,
    #[serde(rename(deserialize = "type"))]
    pub item_type: String,
}

#[derive(Debug, Deserialize)]
pub struct ItemMutation {
    id: String  
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum ProjectV2ItemField {
    TextValue {
        text: String,
        field: Field,
    },
    DateValue {
        date: String, // Assuming date is a string
        field: Field,
    },
    SingleSelectValue {
        name: String,
        field: ProjectV2SingleSelectField,
    },
    NumberValue {
        number: f32,
        field: Field,
    },
    IterationValue {
        duration: u8,
        title: String,
        field: ProjectV2IterationField,
    },
    Empty(Value), // Represents the empty field
}

impl ProjectV2ItemField {
    pub fn value(&self) -> String {
        use ProjectV2ItemField::*;

        match self {
            Empty(_) => String::new(),
            TextValue { text, field: _ } => text.to_owned(), 
            DateValue { date, field: _ } => date.to_owned(),
            SingleSelectValue { name, field: _ } => name.to_owned(), 
            NumberValue { number, field: _ } => number.to_string(),
            IterationValue { duration: _, title, field: _} => title.to_owned(), 
        }
    }

    pub fn style(&self) -> Style {
        use ProjectV2ItemField::*;
        
        match self {
            TextValue { text: _, field: _ } => Style::default(),
            DateValue { date: _, field: _ } => Style::default().bold(),
            SingleSelectValue { name, field } => 
                field.options.iter().find(|v| &v.name == name).unwrap().style() 
            ,
            NumberValue { number: _, field: _ } => Style::default().light_blue(),
            IterationValue { duration: _, title: _, field: _ } => Style::default().bold(), 
            Empty(_) => Style::default(),
        }
    }

    pub fn get_type(&self) -> &str {
        use ProjectV2ItemField::*;

        match self {
            SingleSelectValue { name: _, field } => &field.data_type,
            DateValue { date: _, field } |
            NumberValue { number: _, field } |
            TextValue { text: _, field } => field.get_type(),
            IterationValue { duration: _, title: _, field } => &field.data_type,
            Empty(_) => "",
        }
    }
}

impl Nodes<ProjectV2ItemField> {
    pub fn get_from_field(&self, s: &str) -> &ProjectV2ItemField {
        use ProjectV2ItemField::*;

        self.nodes.iter().find(|v| 
        match v {
            Empty(_v) => "",
            TextValue { text: _, field } => &field.get_name(), 
            DateValue { date: _, field } => &field.get_name(),
            SingleSelectValue { name: _, field } => &field.name,
            NumberValue { number: _, field } => &field.get_name(),
            IterationValue { duration: _, title: _, field } => &field.name,
        } == s).unwrap_or(&Empty(Value::Null))
    }

    pub fn name_from_field(&self, s: &str) -> String {
        self.get_from_field(s).value()
    }

    pub fn set_value(&mut self, index: &str, value: &str) {
        use ProjectV2ItemField::*;

        let s = value.to_string();
        
        if let Some(item_field) = self.nodes.iter_mut().find(|v| 
            match v {
                Empty(_) => false,
                TextValue { text: _, field } => field.get_name() == index,
                DateValue { date: _, field } => field.get_name() == index,
                SingleSelectValue { name: _, field } => field.name == index,
                NumberValue { number: _, field } => field.get_name() == index,
                IterationValue { duration: _, title: _, field } => field.name == index,
            }) {
                match item_field {
                    Empty(_) => {} // Handle Empty variant as needed,
                    TextValue { text, field: _ } => *text = s,
                    DateValue { date, field: _ } => *date = s,
                    SingleSelectValue { name, field: _ } => *name = s,
                    NumberValue { number, field: _ } => *number = s.parse().unwrap(),
                    IterationValue { duration: _, title, field: _ } => *title = s,
                };
            }
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
pub struct Nodes<T> {
    pub nodes: Vec<T>,
}

impl Default for Nodes<ProjectV2ItemField> {
    fn default() -> Nodes<ProjectV2ItemField> {
        Nodes { nodes: Vec::new() }
    }
}

#[derive(Debug, Deserialize)]
pub struct Project {
    pub id: String,
    pub title: String,
}


#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum Field {
    ProjectV2SingleSelectField(ProjectV2SingleSelectField),
    ProjectV2IterationField(ProjectV2IterationField),
    ProjectV2Field(ProjectV2Field),
    Empty(Value),
}

impl Field {
    pub fn get_name(&self) -> &str {
        match self {
            Field::ProjectV2Field(pf)  => &pf.name,
            Field::ProjectV2IterationField(pf) => &pf.name,
            Field::ProjectV2SingleSelectField(pf) => &pf.name,
            Field::Empty(_) => "",
        }
    }

    pub fn get_id(&self) -> &str {
        match self {
            Field::ProjectV2Field(field) => &field.id,
            Field::ProjectV2IterationField(field) => &field.id,
            Field::ProjectV2SingleSelectField(field) =>  &field.id,
            Field::Empty(_) => "",
        }
    }

    pub fn get_type(&self) -> &str {
        match self {
            Field::ProjectV2Field(field) => &field.data_type,
            Field::ProjectV2IterationField(field) => &field.data_type,
            Field::ProjectV2SingleSelectField(field) =>  &field.data_type,
            Field::Empty(_) => "",
        }
    }

    pub fn is_editable(&self) -> bool {
        vec!["DATE", "NUMBER", "TEXT", "TITLE", "SINGLE_SELECT"].contains(&self.get_type())
    }

    pub fn default(&self) -> ProjectV2ItemField {
        use ProjectV2ItemField::*;

        match self {
            Field::ProjectV2SingleSelectField(f) => SingleSelectValue { name: f.options[0].name.clone(), field: f.clone() },
            Field::ProjectV2IterationField(f) => IterationValue { duration: 7, title: String::from("Iteration 1"),  field: f.clone() },
            Field::ProjectV2Field(_) => match self.get_type() {
                "DATE" => {
                    DateValue { date: String::from("1970-1-1"), field: self.clone() }
                },
                "NUMBER" => NumberValue { number: 0.0, field: self.clone() },
                "TEXT" | "TITLE" => TextValue { text: String::new(), field: self.clone() },
                _ => Empty(Value::Null)
            }

            Field::Empty(_) => todo!(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all="camelCase")]
pub struct ProjectV2Field {
    pub id: String,
    pub name: String,
    pub data_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct ProjectV2FieldCommon {
    pub id: String,
    pub name: String,
    pub data_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all="camelCase")]
pub struct ProjectV2IterationField {
    pub id: String,
    pub name: String,
    pub configuration: IterationConfig,
    pub data_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all="camelCase")]
pub struct ProjectV2SingleSelectField {
    pub id: String,
    pub name: String,
    pub options: Vec<FieldOption>,
    pub data_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all="camelCase")]
pub struct Iteration {
    pub start_date: String,
    pub id: String,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IterationConfig {
    pub iterations: Vec<Iteration>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FieldOption {
    pub id: String,
    pub name: String,
    pub color: String,
    pub description: String,
}

impl FieldOption {
    pub fn style(&self) -> Style {
        match self.color.as_str() {
            "BLUE" => Style::default().blue(),
            "GRAY" => Style::default().yellow(),
            "GREEN" => Style::default().green(),
            "ORANGE" => Style::default().light_red(),
            "PINK" => Style::default().light_magenta(),
            "PURPLE" => Style::default().magenta(),
            "YELLOW" => Style::default().yellow(),

            _ => Style::default(),
        }

    }
}

#[derive(Debug, Deserialize)]
pub struct Card {
    pub url: String,
    pub id: u32,
    pub note: String,
    pub creator: User,
}

