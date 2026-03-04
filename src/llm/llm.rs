use crate::f1::race_control::{RaceControlParams, get_race_control_details};
use crate::kalshi::balance::get_balance;
use crate::kalshi::markets::get_f1_market_details;
use crate::kalshi::orders::get_open_order_details;
use crate::kalshi::positions::get_positions_details;
use crate::kalshi::purchase::place_order;
use crate::llm::search_agent::search_agent;
use crate::{config, kalshi::balance::get_portfolio_value};
use anyhow::Result;
use reqwest::{
    Client,
    header::{HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize};

const KALSHI_GROK_MODEL: &str = "grok-4-1-fast-non-reasoning";
const PRICING_GROK_MODEL: &str = "grok-4-1-fast-reasoning";
const GROK_URL: &str = "https://api.x.ai/v1/responses";

#[derive(Debug, Serialize)]
struct LLMInput {
    model: String,
    input: Vec<LLMMessage>,
    tools: Option<Vec<LLMTool>>,
    previous_response_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum LLMTool {
    WebSearch,
    XSearch,
    Function {
        name: String,
        description: String,
        parameters: serde_json::Value,
    },
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
pub struct IntermediateLLMResponse {
    pub output: String,
    pub error: Option<String>,
    pub cost: f32,
    pub is_complete: bool,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanLLMResponse {
    pub output: String,
    pub error: Option<String>,
    pub cost: f32,
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
        arguments: Option<String>,
    },
    Message {
        content: Vec<LLMContent>,
        status: String,
    },
    WebSearchCall {
        id: String,
        status: String,
    },
    XSearchCall {
        id: String,
        status: String,
    },
    CustomToolCall {
        id: String,
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

pub async fn query_agent(question: &str) -> Result<CleanLLMResponse> {
    let mut response = query_llm_with_kalshi_tools(None, question).await?;
    let mut total_cost = response.cost;
    let mut total_iterations = 0;

    while !response.is_complete && total_iterations < 10 {
        response = query_llm_with_kalshi_tools(Some(response.id.clone()), &response.output).await?;
        total_cost += response.cost;
        total_iterations += 1;
    }

    Ok(CleanLLMResponse {
        output: response.output,
        error: response.error,
        cost: total_cost,
    })
}

fn build_grok_headers(api_key: &str) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_str("application/json")?);
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", api_key))?,
    );
    Ok(headers)
}

pub async fn query_llm_with_built_in_tools(
    previous_response_id: Option<String>,
    prompt: String,
) -> Result<CleanLLMResponse> {
    let api_key = config::get_grok_api_key()?;
    let client = Client::new();
    let header_map = build_grok_headers(&api_key)?;

    let body = serde_json::to_string(&LLMInput {
        model: PRICING_GROK_MODEL.to_string(),
        input: vec![LLMMessage {
            role: "user".to_string(),
            content: format!(
                "User prompt: {}. Please be concise and do not take too long.",
                prompt
            ),
        }],
        tools: Some(vec![LLMTool::WebSearch, LLMTool::XSearch]),
        previous_response_id,
    })?;

    let res = client
        .post(GROK_URL)
        .headers(header_map)
        .body(body)
        .send()
        .await?;

    let status = res.status();
    if !status.is_success() {
        return Err(anyhow::Error::msg(res.text().await?));
    }

    let response = res.json::<RawLLMResponse>().await?;
    let cost = response.usage.cost_in_usd_ticks / 10_000_000_000.0;

    let text = response
        .output
        .into_iter()
        .find_map(|o| match o {
            LLMOutput::Message { content, .. } => Some(content[0].text.clone()),
            _ => None,
        })
        .ok_or_else(|| anyhow::anyhow!("No message found in response output"))?;

    Ok(CleanLLMResponse {
        output: text,
        error: response.error,
        cost,
    })
}

pub async fn query_llm_with_kalshi_tools(
    previous_response_id: Option<String>,
    prompt: &str,
) -> Result<IntermediateLLMResponse> {
    let api_key = config::get_grok_api_key()?;
    let client = Client::new();
    let header_map = build_grok_headers(&api_key)?;

    let body = serde_json::to_string(&LLMInput {
        model: KALSHI_GROK_MODEL.to_string(),
        input: vec![LLMMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }],
        tools: Some(vec![
            LLMTool::Function {
                name: "getBalance".to_string(),
                description: "Get the current Kalshi cash balance in cents (ex: 100 = $1.00). This is different from portfolio value."
                    .to_string(),
                parameters: serde_json::Value::Object(serde_json::Map::new()),
            },
            LLMTool::Function {
                name: "getPortfolioValue".to_string(),
                description: "Get the current Kalshi portfolio value in cents (ex: 100 = $1.00). This is different from balance."
                    .to_string(),
                parameters: serde_json::Value::Object(serde_json::Map::new()),
            },
            LLMTool::Function {
                name: "getF1Markets".to_string(),
                description: "Get the current Kalshi F1 race markets.".to_string(),
                parameters: serde_json::Value::Object(serde_json::Map::new()),
            },
            LLMTool::Function {
                name: "getOrders".to_string(),
                description: "Get the current Kalshi orders."
                    .to_string(),
                parameters: serde_json::Value::Object(serde_json::Map::new()),
            },
            LLMTool::Function {
                name: "getPositions".to_string(),
                description: "Get the current Kalshi positions."
                    .to_string(),
                parameters: serde_json::Value::Object(serde_json::Map::new()),
            },
            LLMTool::Function {
                name: "searchAgent".to_string(),
                description: "Send a query to a sub-agent that answers it using web and X (Twitter) search. Use this to look up current information, news, or any question that benefits from real-time search.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The question or search query for the sub-agent to answer"
                        }
                    },
                    "required": ["query"]
                }),
            },
            LLMTool::Function {
                name: "getRaceControl".to_string(),
                description: "Fetch race control events from the OpenF1 API (flags, safety cars, messages, etc.). All parameters are optional filters. Setting session key equal to the string 'latest' will get you the current/latest session".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "session_key": {
                            "type": "string",
                            "description": "Filter by OpenF1 session key, or 'latest' for the current session"
                        },
                        "meeting_key": {
                            "type": "integer",
                            "description": "Filter by OpenF1 meeting key"
                        },
                        "driver_number": {
                            "type": "integer",
                            "description": "Filter events for a specific driver number"
                        },
                        "flag": {
                            "type": "string",
                            "description": "Filter by flag type, e.g. 'BLACK AND WHITE', 'YELLOW', 'RED', 'SAFETY CAR'"
                        },
                        "category": {
                            "type": "string",
                            "description": "Filter by event category, e.g. 'Flag', 'SafetyCar', 'Other'"
                        },
                        "lap_number": {
                            "type": "integer",
                            "description": "Filter by lap number"
                        },
                        "date_from": {
                            "type": "string",
                            "description": "Start of date range (ISO 8601, e.g. '2023-01-01')"
                        },
                        "date_to": {
                            "type": "string",
                            "description": "End of date range (ISO 8601, exclusive, e.g. '2023-09-01')"
                        }
                    },
                    "required": []
                }),
            },
            LLMTool::Function {
                name: "createOrder".to_string(),
                description: "Place an order on Kalshi. Use yes_price for buying/selling Yes contracts, or no_price for buying/selling No contracts. Prices are in cents (1-99). Only provide the price field relevant to your side.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "ticker": {
                            "type": "string",
                            "description": "The market ticker to place the order on"
                        },
                        "side": {
                            "type": "string",
                            "enum": ["yes", "no"],
                            "description": "Which side to trade: yes or no"
                        },
                        "action": {
                            "type": "string",
                            "enum": ["buy", "sell"],
                            "description": "Whether to buy or sell"
                        },
                        "count": {
                            "type": "integer",
                            "description": "Number of contracts to trade (minimum 1)",
                            "minimum": 1
                        },
                        "yes_price": {
                            "type": "integer",
                            "description": "Limit price in cents for the Yes side (1-99). Provide when side is yes.",
                            "minimum": 1,
                            "maximum": 99
                        },
                        "no_price": {
                            "type": "integer",
                            "description": "Limit price in cents for the No side (1-99). Provide when side is no.",
                            "minimum": 1,
                            "maximum": 99
                        }
                    },
                    "required": ["ticker", "side", "action", "count"]
                }),
            },
        ]),
        previous_response_id,
    })?;

    let res = client
        .post(GROK_URL)
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
        LLMOutput::FunctionCall { name, arguments } => {
            println!("{}, arguments: {}", name, arguments.clone().unwrap());
            match name.as_str() {
                "getBalance" => {
                    return Ok(IntermediateLLMResponse {
                        output: get_balance().await?,
                        error: response.error,
                        cost,
                        is_complete: false,
                        id: response.id,
                    });
                }
                "getPortfolioValue" => {
                    return Ok(IntermediateLLMResponse {
                        output: get_portfolio_value().await?,
                        error: response.error,
                        cost,
                        is_complete: false,
                        id: response.id,
                    });
                }
                "getF1Markets" => {
                    return Ok(IntermediateLLMResponse {
                        output: get_f1_market_details().await?,
                        error: response.error,
                        cost,
                        is_complete: false,
                        id: response.id,
                    });
                }
                "getOrders" => {
                    return Ok(IntermediateLLMResponse {
                        output: get_open_order_details().await?,
                        error: response.error,
                        cost,
                        is_complete: false,
                        id: response.id,
                    });
                }
                "getPositions" => {
                    return Ok(IntermediateLLMResponse {
                        output: get_positions_details().await?,
                        error: response.error,
                        cost,
                        is_complete: false,
                        id: response.id,
                    });
                }
                "getRaceControl" => {
                    let args_str = arguments.as_deref().unwrap_or("{}");
                    let args: serde_json::Value = serde_json::from_str(args_str)
                        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

                    let params = RaceControlParams {
                        session_key: args["session_key"].as_str().map(String::from),
                        meeting_key: args["meeting_key"].as_i64().map(|v| v as i32),
                        driver_number: args["driver_number"].as_i64().map(|v| v as i32),
                        flag: args["flag"].as_str().map(String::from),
                        category: args["category"].as_str().map(String::from),
                        lap_number: args["lap_number"].as_i64().map(|v| v as i32),
                        date_from: args["date_from"].as_str().map(String::from),
                        date_to: args["date_to"].as_str().map(String::from),
                    };

                    return Ok(IntermediateLLMResponse {
                        output: get_race_control_details(params).await?,
                        error: response.error,
                        cost,
                        is_complete: false,
                        id: response.id,
                    });
                }
                "searchAgent" => {
                    let args_str = arguments.as_deref().unwrap_or("{}");
                    let args: serde_json::Value = serde_json::from_str(args_str)
                        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

                    let query = args["query"]
                        .as_str()
                        .ok_or_else(|| anyhow::anyhow!("searchAgent: missing required field 'query'"))?
                        .to_string();

                    return Ok(IntermediateLLMResponse {
                        output: search_agent(query).await?,
                        error: response.error,
                        cost,
                        is_complete: false,
                        id: response.id,
                    });
                }
                "createOrder" => {
                    let args_str = arguments.as_deref().unwrap_or("{}");
                    let args: serde_json::Value = serde_json::from_str(args_str)
                        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

                    let ticker = args["ticker"].as_str().ok_or_else(|| {
                        anyhow::anyhow!("createOrder: missing required field 'ticker'")
                    })?;
                    let side = args["side"].as_str().ok_or_else(|| {
                        anyhow::anyhow!("createOrder: missing required field 'side'")
                    })?;
                    let action = args["action"].as_str().ok_or_else(|| {
                        anyhow::anyhow!("createOrder: missing required field 'action'")
                    })?;
                    let count = args["count"].as_i64().ok_or_else(|| {
                        anyhow::anyhow!("createOrder: missing required field 'count'")
                    })? as i32;
                    let yes_price = args["yes_price"].as_i64().map(|p| p as i32);
                    let no_price = args["no_price"].as_i64().map(|p| p as i32);

                    return Ok(IntermediateLLMResponse {
                        output: place_order(ticker, side, action, count, yes_price, no_price)
                            .await?,
                        error: response.error,
                        cost,
                        is_complete: false,
                        id: response.id,
                    });
                }
                _ => {
                    return Err(anyhow::Error::msg(format!(
                        "Unknown function call: {}",
                        name
                    )));
                }
            }
        }
        LLMOutput::Message { content, .. } => {
            println!("{}", content[0].text);
            return Ok(IntermediateLLMResponse {
                output: content[0].text.clone(),
                error: response.error,
                cost,
                is_complete: true,
                id: response.id,
            });
        }
        LLMOutput::WebSearchCall { .. }
        | LLMOutput::XSearchCall { .. }
        | LLMOutput::CustomToolCall { .. } => {
            return Err(anyhow::Error::msg(
                "Unexpected built-in tool call in kalshi tools function",
            ));
        }
    };
}
