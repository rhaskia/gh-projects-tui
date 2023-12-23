// TODO: fix on release
#![allow(dead_code)]
#![allow(unused_assignments)]
#![allow(unused_attributes)]
#![allow(unused_imports)]
#![allow(unused_macros)]
#![allow(unused_mut)]
#![allow(unused_variables)]

const CLIENT_ID: &str = include_str!("client_id");
const CLIENT_SECRET: &str = include_str!("client_secret");

use anyhow::anyhow;
use github_device_flow::authorize;
use reqwest::blocking::Response;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, Value};
use std::fs;

use crate::project::*;

#[derive(Debug, Serialize)]
struct AccessTokenRequest {
    client_id: String,
    client_secret: String,
    code: String,
    grant_type: String,
}

use serde_json::{from_value, json};

pub fn send_query_request(token: &str, query: &str) -> anyhow::Result<Response> {
    let client = reqwest::blocking::Client::new();

    // Make the POST request
    Ok(client
        .post("https://api.github.com/graphql")
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", token))
        .header(reqwest::header::USER_AGENT, "Projects TUI")
        .json(&serde_json::json!({ "query": query }))
        .send()?)
}

pub fn get_user(token: &str) -> anyhow::Result<User> {
    let client = reqwest::blocking::Client::new();

    let response = client
        .get("https://api.github.com/user")
        .header("Accept", "application/vnd.github+json")
        .header("Authorization", format!("Bearer {}", token))
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "Projects-TUI")
        .send();

    Ok(response?.json::<User>()?)
}

pub fn get_project_ids(token: &str, login: &str) -> Result<Vec<Project>, anyhow::Error> {
    let query =
        "{user(login: \"USER\") {projectsV2(first: 20) {nodes {id title}}}}".replace("USER", login);

    let res = send_query_request(token, &query)?;

    let res_json = res.json::<serde_json::Value>()?;

    let values = res_json
        .get("data")
        .and_then(|v| v.get("user"))
        .and_then(|v| v.get("projectsV2"))
        .and_then(|v| v.get("nodes"))
        .ok_or_else(|| anyhow!("JSON object did not contain items"))?
        .as_array()
        .ok_or_else(|| anyhow!("JSON object could not be turned into array"))?;

    // return values turned into a Vec<Project> instead of a Vec<Value>
    Ok(values
        .iter()
        .map(|c| from_value(c.clone()).expect("Broken project struct"))
        .collect())
}

/// Returns all fields that a project has
pub fn fetch_project_fields(token: &str, project_id: &str) -> Result<Vec<Field>, anyhow::Error> {
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
                                    }
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
        .send()?;

    let response_json: Value = response.json()?;

    let nodes = rip_data(&response_json, "fields");

    Ok(serde_json::from_value(nodes.clone())?)
}

pub fn fetch_project_items(token: &str, project_id: &str) -> anyhow::Result<Vec<Item>> {
    let query = r#"
        query {
            node(id: "PROJECT_ID") {
                ... on ProjectV2 {
                    items(first: 100) {
                        nodes {
                            id
                            fieldValues(first: 20) {
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
        }"#
    .replace("PROJECT_ID", project_id);

    let response = send_query_request(token, &query)?;

    let response_json = response.json::<Value>()?;
    let nodes = rip_data(&response_json, "items");

    Ok(serde_json::from_value(nodes.clone())?)
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

    let _ = fs::write(
        "./access_token",
        serde_json::to_string(&cred).expect("Failed to serialize"),
    );

    cred
}

pub fn update_item_field(token: &str, project_id: &str, item_id: &str, field_id: &str,) {
    let client = reqwest::Client::new();

    let query = r#"mutation {
        updateProjectV2ItemFieldValue(
            input: {
                projectId: ""
                itemId: ""
                fieldId: ""
                value: {
                    text: "Updated text"
                }
            }
        ) {
            projectV2Item {
                id
            }
        }
    }"#;

    let query = query
        .replace("\"", "\\\"")
        .replace("PROJECT_ID", project_id)
        .replace("ITEM_ID", item_id)
        .replace("FIELD_ID", field_id);
}

pub fn add_draft_issue(token: &str, project_id: &str, body: &str, title: &str) -> anyhow::Result<String> {
    let query = r#"mutation {
        addProjectV2DraftIssue(
            input: {
                projectId: "PROJECT_ID"
                title: "TITLE"
                body: "BODY"
            }
        ) {
            projectItem {
                id
            }
        }
    }"#;

    let query = query
        .replace("PROJECT_ID", project_id)
        .replace("TITLE", title)
        .replace("BODY", body); 

    let response = send_query_request(token, &query)?;

    println!("{:?}", response.text());

    Ok(String::new())
}
