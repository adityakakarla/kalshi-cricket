use crate::{
    kalshi::markets::{IndividualMarket, get_market_details_without_price},
    llm::llm::query_llm_without_tools,
};
use anyhow::Result;

pub async fn price_market(market: IndividualMarket) -> Result<String> {
    let details = get_market_details_without_price(&market);
    let response = query_llm_without_tools(
        None,
        format!(
            "
    Your task is to determine an exact price for the following market.
Based on history, give me an exact valuation for the yes bid.

Market details: {}
",
            details?
        ),
    )
    .await?;
    Ok(response.output)
}
