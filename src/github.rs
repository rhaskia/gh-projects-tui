use anyhow::anyhow;
use reqwest::blocking::Response;
use serde::Serialize;
use serde_json::Value;

use crate::project::*;

#[derive(Debug, Serialize)]
struct AccessTokenRequest {
    client_id: String,
    client_secret: String,
    code: String,
    grant_type: String,
}

use serde_json::from_value;

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
        .send()?;

    Ok(response.json::<User>()?)
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
    let query = r#"
            query {
                node(id: "PROJECT_ID") {
                    ... on ProjectV2 {
                        fields(first: 50) {
                            nodes {
                                ... on ProjectV2IterationField {
                                    id
                                    name
                                    dataType
                                    configuration {
                                        iterations {
                                            startDate
                                            id
                                            title
                                        }
                                    }
                                }
                                ... on ProjectV2SingleSelectField {
                                    id
                                    name
                                    dataType
                                    options {
                                        id
                                        name
                                        color
                                        description
                                    }
                                }
                                ... on ProjectV2Field {
                                    id
                                    name
                                    dataType
                                }
                            }
                        }
                    }
                }
            }
        "#.replace("PROJECT_ID", project_id);

    let _client = reqwest::blocking::Client::new();
    let response = send_query_request(&token, &query)?;
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
                                                dataType
                                                id
                                            }
                                        }
                                    }
                                    ... on ProjectV2ItemFieldNumberValue {
                                        number
                                        field {
                                            ... on ProjectV2FieldCommon {
                                                name
                                                dataType
                                                id
                                            }
                                        }
                                    }
                                    ... on ProjectV2ItemFieldDateValue {
                                        date
                                        field {
                                            ... on ProjectV2FieldCommon {
                                                name
                                                dataType
                                                id
                                            }

                                        }
                                    }
                                    ... on ProjectV2ItemFieldIterationValue {
                                        duration
                                        title
                                        field {
                                            ... on ProjectV2IterationField {
                                                id
                                                name
                                                dataType
                                                configuration {
                                                    iterations {
                                                        startDate
                                                        id
                                                        title
                                                    }
                                                }
                                            }
                                            ... on ProjectV2FieldCommon {
                                                name
                                                id
                                            }

                                        }
                                    }
                                    ... on ProjectV2ItemFieldSingleSelectValue {
                                        name
                                        field {
                                            ... on ProjectV2SingleSelectField {
                                                id
                                                name
                                                dataType
                                                options {
                                                    id
                                                    name
                                                    color
                                                    description
                                                }
                                            }
                                            ... on ProjectV2FieldCommon {
                                                name
                                                id
                                            }
                                        }
                                    }
                                }
                            }
                            type
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

pub fn update_item_number(
    token: &str,
    project_id: &str,
    item_id: &str,
    field_id: &str,
    new_number: f32,
) -> anyhow::Result<ItemMutation> {
    let _client = reqwest::Client::new();

    let query = r#"mutation {
        updateProjectV2ItemFieldValue(
            input: {
                projectId: "PROJECT_ID"
                itemId: "ITEM_ID"
                fieldId: "FIELD_ID"
                value: {
                    number: NEW_TEXT
                }
            }
        ) {
            projectV2Item {
                id
            }
        }
    }"#;

    let query = query
        .replace("PROJECT_ID", project_id)
        .replace("ITEM_ID", item_id)
        .replace("FIELD_ID", field_id)
        .replace("NEW_TEXT", &new_number.to_string());

    let response = send_query_request(token, &query)?;
    let response_json = response.json::<Value>()?;

    let mutation = &response_json["data"]["updateProjectV2ItemFieldValue"]["projectV2Item"];

    Ok(serde_json::from_value(mutation.clone())?)
}

pub fn update_item_date(
    token: &str,
    project_id: &str,
    item_id: &str,
    field_id: &str,
    new_date: &str,
) -> anyhow::Result<ItemMutation> {
    let query = r#"mutation {
        updateProjectV2ItemFieldValue(
            input: {
                projectId: "PROJECT_ID"
                itemId: "ITEM_ID"
                fieldId: "FIELD_ID"
                value: {
                    date: "NEW_TEXT"
                }
            }
        ) {
            projectV2Item {
                id
            }
        }
    }"#;

    let query = query
        .replace("PROJECT_ID", project_id)
        .replace("ITEM_ID", item_id)
        .replace("FIELD_ID", field_id)
        .replace("NEW_TEXT", new_date);

    let response = send_query_request(token, &query)?;
    let response_json = response.json::<Value>()?;

    let mutation = &response_json["data"]["updateProjectV2ItemFieldValue"]["projectV2Item"];

    Ok(serde_json::from_value(mutation.clone())?)
}

pub fn update_item_text(
    token: &str,
    project_id: &str,
    item_id: &str,
    field_id: &str,
    new_text: &str,
) -> anyhow::Result<ItemMutation> {
    let query = r#"mutation {
        updateProjectV2ItemFieldValue(
            input: {
                projectId: "PROJECT_ID"
                itemId: "ITEM_ID"
                fieldId: "FIELD_ID"
                value: {
                    text: "NEW_TEXT"
                }
            }
        ) {
            projectV2Item {
                id
            }
        }
    }"#;

    let query = query
        .replace("PROJECT_ID", project_id)
        .replace("ITEM_ID", item_id)
        .replace("FIELD_ID", field_id)
        .replace("NEW_TEXT", new_text);

    let response = send_query_request(token, &query)?;
    let response_json = response.json::<Value>()?;
    
    if let Value::String(err) = &response_json["errors"][0]["message"] {
       return Err(anyhow!(err.to_owned()));
    }

    let mutation = &response_json["data"]["updateProjectV2ItemFieldValue"]["projectV2Item"];

    Ok(serde_json::from_value(mutation.clone())?)
}

pub fn update_item_option(
    token: &str,
    project_id: &str,
    item_id: &str,
    field_id: &str,
    option_id: &str,
) -> anyhow::Result<ItemMutation> {
    let query = r#"mutation {
        updateProjectV2ItemFieldValue(
            input: {
                projectId: "PROJECT_ID" 
                itemId: "ITEM_ID" 
                fieldId: "FIELD_ID" 
                value: { 
                    singleSelectOptionId: "OPTION_ID" 
                }
            }
            ) { 
            projectV2Item {
                id 
            }
        }
    }"#;

    let query = query
        .replace("PROJECT_ID", project_id)
        .replace("ITEM_ID", item_id)
        .replace("FIELD_ID", field_id)
        .replace("OPTION_ID", option_id);

    let response = send_query_request(token, &query)?;
    let response_json = response.json::<Value>()?;

    if let Value::String(err) = &response_json["errors"][0]["message"] {
       return Err(anyhow!(err.to_owned()));
    }

    let mutation = &response_json["data"]["updateProjectV2ItemFieldValue"]["projectV2Item"];

    Ok(serde_json::from_value(mutation.clone())?)
}

pub fn add_draft_issue(
    token: &str,
    project_id: &str,
    body: &str,
    title: &str,
) -> anyhow::Result<Item> {
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
                type
            }
        }
    }"#;

    let query = query
        .replace("PROJECT_ID", project_id)
        .replace("TITLE", title)
        .replace("BODY", body);

    let response = send_query_request(token, &query)?;
    let response_json = response.json::<Value>()?;

    if let Value::String(err) = &response_json["errors"][0]["message"] {
       return Err(anyhow!(err.to_owned()));
    }

    let mutation = &response_json["data"]["addProjectV2DraftIssue"]["projectItem"];

    Ok(serde_json::from_value(mutation.clone())?)
}
