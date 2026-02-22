use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::kalshi::kalshi::make_get_request;

#[derive(Debug, Serialize, Deserialize)]
struct Markets {
    markets: Vec<IndividualMarket>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Market {
    market: IndividualMarket,
}

#[derive(Debug, Serialize, Deserialize)]
struct IndividualMarket {
    ticker: String,
    event_ticker: String,
    title: String,
    subtitle: String,
    yes_sub_title: String,
    no_sub_title: String,
    status: String,
    yes_bid_dollars: String,
    yes_ask_dollars: String,
    no_bid_dollars: String,
    no_ask_dollars: String,
    volume: u64,
}

async fn get_markets_by_series_ticker(series_ticker: &str) -> Result<Markets> {
    let request = make_get_request(&format!(
        "/markets?series_ticker={}&status=open",
        series_ticker
    ))
    .await?;
    let response = request.json::<Markets>().await?;
    Ok(response)
}

pub async fn get_t20_market_details() -> Result<String> {
    Ok(format!(
        "{:?}",
        get_markets_by_series_ticker("KXT20MATCH").await?
    ))
}

pub async fn get_market_information_by_ticker(ticker: &str) -> Result<Market> {
    let request = make_get_request(&format!("/markets/{}", ticker)).await?;
    let response = request.json::<Market>().await?;
    Ok(response)
}
