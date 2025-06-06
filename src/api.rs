//! Docs reference for the critical endpoints:
//!   • Create snapshot → `POST /servers/{id}/actions/create_image`
//! https://docs.hetzner.cloud/
use crate::error::HbpError;
use reqwest::{
    blocking::{Client, Response},
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{thread, time::Duration};

/// Hetzner Cloud base endpoint.
const BASE: &str = "https://api.hetzner.cloud/v1";

const POLL_INTERVAL: u64 = 5;

#[derive(Debug, Clone, Deserialize)]
pub struct Action {
    pub id: u64,
    pub command: String, // e.g. `"create_image"`, `"export_image"`
    pub status: String,  // `"running" | "success" | "error"`
    pub progress: u8,    // 0-100
    pub error: Option<HetznerError>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HetznerError {
    pub _code: String,
    pub _message: String,
}

pub fn build_client(api_token: &str) -> Result<Client, HbpError> {
    let mut headers = HeaderMap::new();

    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {api_token}"))
            .map_err(|e| HbpError::Other(e.to_string()))?,
    );
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    Ok(Client::builder().default_headers(headers).build()?)
}

pub fn wait_for_action(client: &Client, action: &Action) -> Result<(), HbpError> {
    let mut current = action.clone();

    while current.status == "running" {
        eprintln!(
            "🔄  Action {} ({}) … {} %",
            current.id, current.command, current.progress
        );
        thread::sleep(Duration::from_secs(POLL_INTERVAL));
        current = get_action(client, current.id)?;
    }

    if current.status == "success" {
        Ok(())
    } else {
        Err(HbpError::ActionFailed(format!(
            "{}: {:?}",
            current.command, current.error
        )))
    }
}

/// Fetch the latest state of an **Action** by ID.
fn get_action(client: &Client, id: u64) -> Result<Action, HbpError> {
    let raw: Value = client.get(format!("{BASE}/actions/{id}")).send()?.json()?;

    serde_json::from_value(raw["action"].clone())
        .map_err(|e| HbpError::Parse(format!("action deserialise: {e}")))
}

pub fn create_snapshot(
    client: &Client,
    server_id: u64,
    description: &str,
) -> Result<(u64, Action), HbpError> {
    let body = json!({
        "description": description,
        "type": "snapshot"
    });

    let raw: Value = client
        .post(format!("{BASE}/servers/{server_id}/actions/create_image"))
        .json(&body)
        .send()?
        .json()?;

    let image_id = raw["image"]["id"]
        .as_u64()
        .ok_or_else(|| HbpError::Parse("image.id missing".into()))?;

    let action: Action = serde_json::from_value(raw["action"].clone())
        .map_err(|e| HbpError::Parse(format!("action deserialise: {e}")))?;

    Ok((image_id, action))
}

pub fn export_image(client: &Client, image_id: u64) -> Result<(String, Action), HbpError> {
    let raw: Value = client
        .post(format!("{BASE}/images/{image_id}/actions/export"))
        .json(&json!({})) // empty body is valid
        .send()?
        .json()?;

    let url = raw["image"]["url"]
        .as_str()
        .ok_or_else(|| HbpError::Parse("image.url missing".into()))?
        .to_owned();

    let action: Action = serde_json::from_value(raw["action"].clone())
        .map_err(|e| HbpError::Parse(format!("action deserialise: {e}")))?;

    Ok((url, action))
}

pub fn stream_download(client: &Client, url: &str) -> Result<Response, HbpError> {
    Ok(client.get(url).send()?)
}
