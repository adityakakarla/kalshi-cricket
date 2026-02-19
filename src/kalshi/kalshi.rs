use anyhow::Result;
use base64::{Engine as _, engine::general_purpose};
use reqwest::{
    Response,
    header::{HeaderMap, HeaderValue},
};
use rsa::{
    RsaPrivateKey,
    pkcs1::DecodeRsaPrivateKey,
    pss::SigningKey,
    signature::{RandomizedSigner, SignatureEncoding},
};
use serde::Deserialize;
use sha2::Sha256;

#[derive(Deserialize)]
struct BalanceOutput {
    balance: i64,
    portfolio_value: i64,
}

use crate::config::{get_kalshi_api_key, get_kalshi_key_id};
use chrono::Utc;

pub async fn get_balance() -> Result<String> {
    let response = make_authenticated_request("GET", "/trade-api/v2/portfolio/balance").await?;
    let json = response.json::<BalanceOutput>().await?;
    Ok(json.balance.to_string())
}

pub async fn get_portfolio() -> Result<String> {
    let response = make_authenticated_request("GET", "/trade-api/v2/portfolio").await?;
    let json = response.json::<BalanceOutput>().await?;
    Ok(json.portfolio_value.to_string())
}

async fn make_request(method: &str, path: &str) -> Result<Response> {
    let res = match method {
        "GET" => {
            let client = reqwest::Client::new();
            client
                .get(format!("https://api.elections.kalshi.com{}", path))
                .send()
                .await?
        }
        _ => return Err(anyhow::Error::msg("Unsupported method")),
    };

    if let Err(err) = res.error_for_status_ref() {
        return Err(err.into());
    }

    Ok(res)
}

async fn make_authenticated_request(method: &str, path: &str) -> Result<Response> {
    let kalshi_key_id = get_kalshi_key_id()?;
    let kalshi_private_key = get_kalshi_api_key()?;
    let current_timestamp = Utc::now().timestamp_millis();
    let signature = sign_authenticated_request(
        &kalshi_private_key,
        &current_timestamp.to_string(),
        method,
        path,
    )?;

    let mut headers = HeaderMap::new();
    headers.insert("KALSHI-ACCESS-KEY", HeaderValue::from_str(&kalshi_key_id)?);
    headers.insert(
        "KALSHI-ACCESS-SIGNATURE",
        HeaderValue::from_str(&signature)?,
    );
    headers.insert(
        "KALSHI-ACCESS-TIMESTAMP",
        HeaderValue::from_str(&current_timestamp.to_string())?,
    );

    let res = match method {
        "GET" => {
            let client = reqwest::Client::new();
            client
                .get(format!("https://api.elections.kalshi.com{}", path))
                .headers(headers)
                .send()
                .await?
        }
        _ => return Err(anyhow::Error::msg("Unsupported method")),
    };

    if let Err(err) = res.error_for_status_ref() {
        return Err(err.into());
    }
    Ok(res)
}

fn sign_authenticated_request(
    private_key: &str,
    timestamp: &str,
    method: &str,
    path: &str,
) -> Result<String> {
    let path_without_query = path.split('?').next().unwrap();
    let message = format!("{}{}{}", timestamp, method, path_without_query);
    let rsa_private_key = RsaPrivateKey::from_pkcs1_pem(private_key)?;
    let signing_key = SigningKey::<Sha256>::new(rsa_private_key);
    let signature = signing_key.sign_with_rng(&mut rand::thread_rng(), message.as_bytes());
    Ok(general_purpose::STANDARD.encode(signature.to_bytes()))
}
