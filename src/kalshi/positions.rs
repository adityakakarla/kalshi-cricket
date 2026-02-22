use crate::kalshi::{
    kalshi::make_authenticated_get_request, markets::get_market_information_by_ticker,
};
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

async fn get_positions() -> Result<Positions> {
    let response = make_authenticated_get_request("/portfolio/positions").await?;
    let json = response.json::<Positions>().await?;
    Ok(json)
}

pub async fn get_positions_details() -> Result<String> {
    let positions = get_positions().await?;
    let mut position_details = String::new();

    for position in positions.market_positions {
        position_details.push_str(&format!("{:?}", position));
        position_details.push_str(
            format!(
                "{:?}",
                get_market_information_by_ticker(&position.ticker).await?
            )
            .as_str(),
        );
    }
    Ok(position_details)
}
