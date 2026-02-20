use crate::kalshi::{kalshi::make_authenticated_request, markets::get_market_basics_by_ticker};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Positions {
    market_positions: Vec<MarketPosition>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MarketPosition {
    ticker: String,
    total_traded_dollars: String,
    position: i32,
    market_exposure_dollars: String,
    realized_pnl_dollars: String,
    fees_paid_dollars: String,
}

pub async fn get_positions() -> Result<String> {
    let response = make_authenticated_request("GET", "/portfolio/positions").await?;
    let json = response.json::<Positions>().await?;
    let mut position_details = format!("{:?}", json.market_positions);

    for position in &json.market_positions {
        let market = get_market_basics_by_ticker(&position.ticker).await?;
        position_details.push_str(market.as_str());
    }
    Ok(position_details)
}
