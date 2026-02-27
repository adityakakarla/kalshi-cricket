use std::io;

use crate::kalshi::{api::make_authenticated_post_request, orders::Order};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct CreateOrderRequest {
    pub ticker: String,
    pub side: String,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_price: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_price: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_ts: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_max_cost: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel_order_on_pause: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderResponse {
    pub order: Order,
}

async fn create_order(request: CreateOrderRequest) -> Result<CreateOrderResponse> {
    let body = serde_json::to_value(&request)?;
    let response = make_authenticated_post_request("/portfolio/orders", &body).await?;
    let json = response.json::<CreateOrderResponse>().await?;
    Ok(json)
}

pub async fn place_order(
    ticker: &str,
    side: &str,
    action: &str,
    count: i32,
    yes_price: Option<i32>,
    no_price: Option<i32>,
) -> Result<String> {
    let request = CreateOrderRequest {
        ticker: ticker.to_string(),
        side: side.to_string(),
        action: action.to_string(),
        client_order_id: None,
        count: Some(count),
        yes_price,
        no_price,
        time_in_force: Some("good_till_canceled".to_string()),
        expiration_ts: None,
        buy_max_cost: None,
        post_only: None,
        reduce_only: None,
        cancel_order_on_pause: Some(true),
    };

    println!(
        "Reply with yes to approve the following order:
        {:?}",
        request
    );

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input != "yes" {
        return Ok(String::from("Order was denied by the user"));
    }

    let response = create_order(request).await?;
    Ok(format!(
        "Order placed successfully: order_id={}, ticker={}, side={}, action={}, status={}, yes_price_dollars={}, no_price_dollars={}, fill_count={}",
        response.order.order_id,
        response.order.ticker,
        response.order.side,
        response.order.action,
        response.order.status,
        response.order.yes_price_dollars,
        response.order.no_price_dollars,
        response.order.fill_count,
    ))
}
