const CLIENT_ID: &str = include_str!("client_id");
const CLIENT_SECRET: &str = include_str!("client_secret");

use github_device_flow::authorize;
use reqwest::blocking::Response;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, Value};
use std::fs;

#[derive(Debug, Serialize)]
struct AccessTokenRequest {
    client_id: String,
    client_secret: String,
    code: String,
    grant_type: String,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub login: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all(serialize = "snake_case"))]
pub struct Item {
    pub id: String,
    pub fieldValues: Vec<Value>,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectV2Field {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Iteration {
    pub startDate: String,
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
pub struct Option {
    id: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectV2SingleSelectField {
    pub id: String,
    pub name: String,
    pub options: Vec<Option>,
}

#[derive(Debug, Deserialize)]
pub struct Card {
    pub url: String,
    pub id: u32,
    pub note: String,
    pub creator: User,
}

use serde_json::{from_value, json};

pub fn send_query_request(token: &str, query: &str) -> Response {
    let client = reqwest::blocking::Client::new();

    // Make the POST request
    client
        .post("https://api.github.com/graphql")
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", token))
        .header(reqwest::header::USER_AGENT, "Projects TUI")
        .json(&serde_json::json!({ "query": query }))
        .send()
        .expect("Failed to request")
}

pub fn get_user(token: &str) -> User {
    let client = reqwest::blocking::Client::new();

    let response = client
        .get("https://api.github.com/user")
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", format!("Bearer {}", token))
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "Projects-TUI")
        .send();

    response
        .expect("Request failed")
        .json::<User>()
        .expect("Failed to convert to json")
}

pub fn get_project_ids(token: &str, login: &str) -> Vec<Project> {
    let query =
        "{user(login: \"USER\") {projectsV2(first: 20) {nodes {id title}}}}".replace("USER", login);

    let res = send_query_request(token, &query);

    let res_json = res.json::<serde_json::Value>().unwrap();

    let values = res_json
        .get("data")
        .and_then(|v| v.get("user"))
        .and_then(|v| v.get("projectsV2"))
        .and_then(|v| v.get("nodes"))
        .unwrap()
        .as_array()
        .unwrap();

    // return values turned into a Vec<Project> instead of a Vec<Value>
    values
        .iter()
        .map(|c| from_value(c.clone()).expect("Broken project struct"))
        .collect()
}

/// Returns all fields that a project has
pub fn fetch_project_fields(token: &str, project_id: &str) -> Vec<Field> {
    let graphql_query = json!({
        "query": r#"
            query {
                node(id: "PROJECT_ID") {
                    ... on ProjectV2 {
                        fields(first: 50) {
                            nodes {
                                ... on ProjectV2Field {
                                    id
                                    name
                                }
                                ... on ProjectV2IterationField {
                                    id
                                    name
                                    configuration {
                                        iterations {
                                            startDate
                                            id
                                        }
                                    }
                                }
                                ... on ProjectV2SingleSelectField {
                                    id
                                    name
                                    options {
                                        id
                                        name
                                    ]}
                                }
                            }
                        }
                    }
                }
            }
        "#.replace("PROJECT_ID", project_id),
    });

    let client = reqwest::blocking::Client::new();
    let response = client
        .post("https://api.github.com/graphql")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "Projects TUI")
        .json(&graphql_query)
        .send();

    let response_json: Value = response.unwrap().json().unwrap();

    let nodes = rip_data(&response_json, "fields");

    serde_json::from_value(nodes.clone()).unwrap()
}

#[derive(Deserialize, Debug)]
pub struct wrapper {
    fields: Vec<Item>,
}

pub fn fetch_project_items(token: &str, project_id: &str) -> wrapper {
    let query = r#"
        query {
            node(id: "PROJECT_ID") {
                ... on ProjectV2 {
                    items(first: 20) {
                        nodes {
                            id
                            fieldValues(first: 8) {
                                nodes {
                                    ... on ProjectV2ItemFieldTextValue {
                                        text
                                        field {
                                            ... on ProjectV2FieldCommon {
                                                name
                                            }
                                        }
                                    }
                                    ... on ProjectV2ItemFieldDateValue {
                                        date
                                        field {
                                            ... on ProjectV2FieldCommon {
                                                name
                                            }
                                        }
                                    }
                                    ... on ProjectV2ItemFieldSingleSelectValue {
                                        name
                                        field {
                                            ... on ProjectV2FieldCommon {
                                                name
                                            }
                                        }
                                    }
                                }
                            }
                            content {
                                ... on DraftIssue {
                                    title
                                    body
                                }
                                ... on Issue {
                                    title
                                    assignees(first: 10) {
                                        nodes {
                                            login
                                        }
                                    }
                                }
                                ... on PullRequest {
                                    title
                                    assignees(first: 10) {
                                        nodes {
                                            login
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }"#.replace("PROJECT_ID", project_id);

    let response = send_query_request(token, &query);
    let response_json = response.json::<Value>().expect("Bad Json input");
    let nodes = rip_data(&response_json, "items"); 

    println!("{:?}", nodes);
    
    serde_json::from_value(nodes.clone()).unwrap()
}

pub fn rip_data<'a>(value: &'a Value, path: &'a str) -> &'a Value {
    &value["data"]["node"][path]["nodes"]
}

/// Loads the user's credential, authorizing if it does not exist
// TODO: cut this up for use with UI
pub fn load_id() -> github_device_flow::Credential {
    if let Ok(content) = fs::read_to_string("./access_token") {
        if let Ok(cred) = serde_json::from_str(&content) {
            return cred;
        }
    }

    let cred = authorize(
        String::from("c9dd703e0babd1c77c16"),
        None,
        Some(String::from("project,user:email")),
    )
    .expect("Failure loading credential");

    fs::write(
        "./access_token",
        serde_json::to_string(&cred).expect("Failed to serialize"),
    );

    cred
}
