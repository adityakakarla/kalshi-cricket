use crate::kalshi::kalshi::make_authenticated_get_request;
use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
struct BalanceOutput {
    balance: i64,
    portfolio_value: i64,
}

async fn get_raw_balance() -> Result<BalanceOutput> {
    let response = make_authenticated_get_request("/portfolio/balance").await?;
    let json = response.json::<BalanceOutput>().await?;
    Ok(json)
}

pub async fn get_balance() -> Result<String> {
    let response = get_raw_balance().await?;
    Ok(response.balance.to_string())
}

pub async fn get_portfolio_value() -> Result<String> {
    let response = get_raw_balance().await?;
    Ok(response.portfolio_value.to_string())
}
