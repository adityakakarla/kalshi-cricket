use crate::kalshi::{kalshi::make_authenticated_request, markets::get_market_basics_by_ticker};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Orders {
    orders: Vec<Order>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Order {
    order_id: String,
    ticker: String,
    side: String,
    action: String,
    #[serde(rename = "type")]
    order_type: String,
    yes_price_dollars: String,
    no_price_dollars: String,
    fill_count: u32,
    taker_fees: u32,
    maker_fees: u32,
    taker_fill_cost_dollars: String,
    maker_fill_cost_dollars: String,
}

pub async fn get_open_orders() -> Result<String> {
    let response = make_authenticated_request("GET", "/portfolio/orders").await?;
    let json = response.json::<Orders>().await?;
    let mut order_details = format!("{:?}", json.orders);

    for order in json.orders {
        order_details.push_str(get_market_basics_by_ticker(&order.ticker).await?.as_str())
    }
    Ok(order_details)
}
