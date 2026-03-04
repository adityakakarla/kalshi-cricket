use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::kalshi::api::make_get_request;

#[derive(Debug, Serialize, Deserialize)]
struct Markets {
    markets: Vec<IndividualMarket>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Market {
    market: IndividualMarket,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndividualMarket {
    pub ticker: String,
    pub event_ticker: String,
    pub title: String,
    pub subtitle: String,
    pub yes_sub_title: String,
    pub no_sub_title: String,
    pub status: String,
    pub yes_bid_dollars: String,
    pub yes_ask_dollars: String,
    pub no_bid_dollars: String,
    pub no_ask_dollars: String,
    pub volume: u64,
}

async fn get_markets_by_series_ticker(series_ticker: &str) -> Result<Vec<IndividualMarket>> {
    let request = make_get_request(&format!(
        "/markets?series_ticker={}&status=open",
        series_ticker
    ))
    .await?;
    let response = request.json::<Markets>().await?;
    let mut result = Vec::new();

    for market in response.markets {
        if market.volume > 1000 {
            result.push(market)
        }
    }
    Ok(result)
}

pub async fn get_f1_market_details() -> Result<String> {
    Ok(format!(
        "{:?}",
        get_markets_by_series_ticker("KXF1RACE").await?
    ))
}

pub async fn get_market_information_by_ticker(ticker: &str) -> Result<IndividualMarket> {
    let request = make_get_request(&format!("/markets/{}", ticker)).await?;
    let response = request.json::<Market>().await?;
    Ok(response.market)
}

