use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue};
use rsa::{
    RsaPrivateKey,
    pkcs8::DecodePrivateKey,
    pss::{Signature, SigningKey},
    signature::RandomizedSigner,
};
use sha2::Sha256;

use crate::config::{get_kalshi_api_key, get_kalshi_key_id};
use chrono::Utc;

pub async fn get_kalshi_cricket_events() -> Result<String> {
    let response = make_authenticated_request("GET", "/trade-api/v2/portfolio/balance").await?;
    Ok(response)
}

async fn make_authenticated_request(method: &str, path: &str) -> Result<String> {
    let kalshi_key_id = get_kalshi_key_id()?;
    let kalshi_private_key = get_kalshi_api_key()?;
    let current_timestamp = Utc::now().timestamp();
    let signature = sign_authenticated_request(
        &kalshi_private_key,
        &current_timestamp.to_string(),
        method,
        path,
    )?;

    let mut headers = HeaderMap::new();
    headers.insert("KALSHI_ACCESS_KEY", HeaderValue::from_str(&kalshi_key_id)?);
    headers.insert(
        "KALSHI_ACCESS_SIGNATURE",
        HeaderValue::from_str(&signature.to_string())?,
    );
    headers.insert(
        "KALSHI_ACCESS_TIMESTAMP",
        HeaderValue::from_str(&current_timestamp.to_string())?,
    );

    let client = reqwest::Client::new();
    let res = client
        .get(format!("https://demo-api.kalshi.co{}", path))
        .headers(headers)
        .send()
        .await?;
    Ok(res.text().await?)
}

fn sign_authenticated_request(
    private_key: &str,
    timestamp: &str,
    method: &str,
    path: &str,
) -> Result<Signature> {
    let path_without_query = path.split('?').next().unwrap();
    let message = format!("{}{}{}", timestamp, method, path_without_query);
    let rsa_private_key = RsaPrivateKey::from_pkcs8_pem(private_key)?;
    let signing_key = SigningKey::<Sha256>::new(rsa_private_key);
    let signature = signing_key.sign_with_rng(&mut rand::thread_rng(), message.as_bytes());
    Ok(signature)
}
