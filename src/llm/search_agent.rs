use crate::llm::llm::query_llm_with_built_in_tools;
use anyhow::Result;

pub async fn search_agent(query: String) -> Result<String> {
    let response = query_llm_with_built_in_tools(None, query).await?;
    Ok(response.output)
}
