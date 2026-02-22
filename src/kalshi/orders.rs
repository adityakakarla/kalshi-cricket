use crate::kalshi::{
    kalshi::make_authenticated_get_request, markets::get_market_information_by_ticker,
};
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

async fn get_open_orders() -> Result<Orders> {
    let response = make_authenticated_get_request("/portfolio/orders").await?;
    let json = response.json::<Orders>().await?;
    Ok(json)
}

pub async fn get_open_order_details() -> Result<String> {
    let orders = get_open_orders().await?;
    let mut order_details = String::new();

    for order in orders.orders {
        order_details.push_str(&format!("{:?}", order));
        order_details.push_str(
            format!(
                "{:?}",
                get_market_information_by_ticker(&order.ticker).await?
            )
            .as_str(),
        );
    }
    Ok(order_details)
}
