pub enum CurrentlyEditing {
    Key,
    Value,
}

pub struct App {
    pub fields: Vec<String>,
    pub values: Vec<Vec<String>>,
}