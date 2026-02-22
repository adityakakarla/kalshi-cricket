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
use sha2::Sha256;

use crate::config::{get_kalshi_api_key, get_kalshi_key_id};
use chrono::Utc;

pub async fn make_get_request(path: &str) -> Result<Response> {
    let client = reqwest::Client::new();
    let res = client
        .get(format!(
            "https://api.elections.kalshi.com/trade-api/v2{}",
            path
        ))
        .send()
        .await?;

    if let Err(err) = res.error_for_status_ref() {
        return Err(err.into());
    }

    Ok(res)
}

pub async fn make_authenticated_get_request(path: &str) -> Result<Response> {
    let kalshi_key_id = get_kalshi_key_id()?;
    let kalshi_private_key = get_kalshi_api_key()?;
    let current_timestamp = Utc::now().timestamp_millis();
    let signature = sign_authenticated_request(
        &kalshi_private_key,
        &current_timestamp.to_string(),
        "GET",
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

    let client = reqwest::Client::new();
    let res = client
        .get(format!(
            "https://api.elections.kalshi.com/trade-api/v2{}",
            path
        ))
        .headers(headers)
        .send()
        .await?;

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
    let message = format!("{}{}/trade-api/v2{}", timestamp, method, path_without_query);
    let rsa_private_key = RsaPrivateKey::from_pkcs1_pem(private_key)?;
    let signing_key = SigningKey::<Sha256>::new(rsa_private_key);
    let signature = signing_key.sign_with_rng(&mut rand::thread_rng(), message.as_bytes());
    Ok(general_purpose::STANDARD.encode(signature.to_bytes()))
}
