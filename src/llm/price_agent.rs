use crate::{
    kalshi::markets::{
        IndividualMarket, get_market_details_without_price, get_market_information_by_ticker,
    },
    llm::llm::query_llm_with_built_in_tools,
};
use anyhow::Result;

pub async fn price_markets_from_tickers(tickers: Vec<String>) -> Result<String> {
    let mut result = String::new();
    for ticker in tickers {
        let market = get_market_information_by_ticker(&ticker).await?;
        let output = price_market(market).await?;
        result.push_str(&output);
        result.push('\n');
    }
    Ok(result)
}

async fn price_market(market: IndividualMarket) -> Result<String> {
    let details = get_market_details_without_price(&market);
    let response = query_llm_with_built_in_tools(
        None,
        format!(
            "
    Your task is to determine an exact price for the following market.
Based on history, give me an exact valuation for the highest yes bid
you would take. Return just the number and nothing else. Your output
should be the yes bid as dollars.

In other words, how much money would you pay (in dollars) to win $1
output.

Ex, if you think there is a 60% chance of winning, your answer should
be 0.60 and nothing else.

Return your answer as a decimal number.

Market details: {}
",
            details?
        ),
    )
    .await?;

    let output = format!(
        "A fair yes bid price for {} is {}",
        market.ticker, response.output
    );
    Ok(output)
}
