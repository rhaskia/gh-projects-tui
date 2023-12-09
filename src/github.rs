use std::collections::HashMap;
use reqwest::{Client, Method, Response, StatusCode, Url};

const CLIENT_ID: &str = include_str!("client_id");
const CLIENT_SECRET: &str = include_str!("client_secret");

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

fn rand_str() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(|c| c as char)
        .collect::<String>()
}

pub(crate) async fn authorize() -> String {
    let state = rand_str();

    let url = format!("https://github.com/login/oauth/authorize?client_id=12345&state=abcdefg");

    let mut map = HashMap::new();
    map.insert("client_id", "Iv1.4b2ae1ac18527f05");
    map.insert("state", &*state);

    let response = Client::new()
        .post(url)
        .json(&map)
        .send()
        .await;

    format!("{:?}", response)
}

pub(crate) async fn get_access_code(code: &str) -> Response {
    let mut map = HashMap::new();
    map.insert("client_secret", CLIENT_SECRET);
    map.insert("code", code);
    map.insert("client_id", CLIENT_ID);

    Client::new()
        .post("https://github.com/login/oauth/access_token")
        .json(&map)
        .send()
        .await?
}