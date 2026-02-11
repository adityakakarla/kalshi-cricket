use crate::config;
use anyhow::Result;
use reqwest::{
    Client,
    header::{self, HeaderMap, HeaderValue},
};
use serde::Serialize;

const GROK_MODEL: &str = "grok-4-1-fast-reasoning";

#[derive(Debug, Serialize)]
struct LLMInput {
    model: String,
    input: Vec<LLMMessage>,
}

#[derive(Debug, Serialize)]
struct LLMMessage {
    role: String,
    content: String,
}

pub async fn generate_text(prompt: &str) -> Result<String> {
    let api_key = config::get_api_key()?;
    let client = Client::new();

    let mut header_map = HeaderMap::new();
    let content_type = HeaderValue::from_str("application/json")?;
    header_map.insert("Content-Type", content_type);
    let authorization = HeaderValue::from_str(format!("Bearer {}", api_key).as_str())?;
    header_map.insert("Authorization", authorization);

    let body = serde_json::to_string(&LLMInput {
        model: GROK_MODEL.to_string(),
        input: vec![LLMMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }],
    })?;

    let res = client
        .post("https://api.x.ai/v1/responses")
        .body(body)
        .headers(header_map)
        .send()
        .await?;

    let status = res.status();
    if !status.is_success() {
        return Err(anyhow::Error::msg(res.text().await?));
    }

    let response = res.text().await?;

    Ok(response)
}
