use crate::config;
use crate::kalshi::kalshi::get_balance;
use anyhow::Result;
use reqwest::{
    Client, Response,
    header::{HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize};

const GROK_MODEL: &str = "grok-4-1-fast-non-reasoning";

#[derive(Debug, Serialize)]
struct LLMInput {
    model: String,
    input: Vec<LLMMessage>,
    tools: Vec<LLMTool>,
    previous_response_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct LLMTool {
    #[serde(rename = "type")]
    tool_type: String,
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct LLMMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RawLLMResponse {
    created_at: i32,
    completed_at: i32,
    id: String,
    model: String,
    output: Vec<LLMOutput>,
    temperature: f32,
    usage: LLMUsage,
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanLLMResponse {
    pub output: String,
    pub error: Option<String>,
    pub cost: f32,
    pub is_complete: bool,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LLMUsage {
    input_tokens: u32,
    output_tokens: u32,
    total_tokens: u32,
    num_sources_used: u32,
    num_server_side_tools_used: u32,
    cost_in_usd_ticks: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum LLMOutput {
    FunctionCall {
        name: String,
    },
    Message {
        content: Vec<LLMContent>,
        status: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct LLMToolCall {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LLMContent {
    text: String,
}

pub async fn answer_question(question: &str) -> Result<CleanLLMResponse> {
    let mut response = generate_text(None, question).await?;

    while !response.is_complete {
        println!("{:?}", response);
        response = generate_text(Some(response.id.clone()), &response.output).await?;
    }

    Ok(response)
}

pub async fn generate_text(
    previous_response_id: Option<String>,
    prompt: &str,
) -> Result<CleanLLMResponse> {
    let api_key = config::get_grok_api_key()?;
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
        tools: vec![LLMTool {
            tool_type: "function".to_string(),
            name: "getBalance".to_string(),
            description: "Get the current Kalshi balance".to_string(),
            parameters: serde_json::Value::Object(serde_json::Map::new()),
        }],
        previous_response_id: previous_response_id,
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

    let response = res.json::<RawLLMResponse>().await?;

    let output = &response.output[0];
    let cost = response.usage.cost_in_usd_ticks / 10_000_000_000.0;

    match output {
        LLMOutput::FunctionCall { .. } => {
            return Ok(CleanLLMResponse {
                output: get_balance().await?,
                error: response.error,
                cost,
                is_complete: false,
                id: response.id,
            });
        }
        LLMOutput::Message { content, .. } => {
            return Ok(CleanLLMResponse {
                output: content[0].text.clone(),
                error: response.error,
                cost,
                is_complete: true,
                id: response.id,
            });
        }
    };
}
