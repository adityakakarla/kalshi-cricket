use anyhow::Result;
use serde::{Deserialize, Serialize};

const OPENF1_BASE_URL: &str = "https://api.openf1.org/v1";

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub circuit_key: i32,
    pub circuit_short_name: String,
    pub country_code: String,
    pub country_key: i32,
    pub country_name: String,
    pub date_end: String,
    pub date_start: String,
    pub gmt_offset: String,
    pub location: String,
    pub meeting_key: i32,
    pub session_key: i32,
    pub session_name: String,
    pub session_type: String,
    pub year: i32,
}

#[derive(Debug, Default)]
pub struct SessionParams {
    pub session_key: Option<String>,
    pub meeting_key: Option<String>,
    pub circuit_key: Option<i32>,
    pub circuit_short_name: Option<String>,
    pub country_code: Option<String>,
    pub country_key: Option<i32>,
    pub country_name: Option<String>,
    pub location: Option<String>,
    pub session_name: Option<String>,
    pub session_type: Option<String>,
    pub year: Option<i32>,
    pub date_start_from: Option<String>,
    pub date_start_to: Option<String>,
}

fn build_sessions_url(params: &SessionParams) -> String {
    let mut query_parts: Vec<String> = Vec::new();

    if let Some(v) = &params.session_key {
        query_parts.push(format!("session_key={}", v));
    }
    if let Some(v) = &params.meeting_key {
        query_parts.push(format!("meeting_key={}", v));
    }
    if let Some(v) = params.circuit_key {
        query_parts.push(format!("circuit_key={}", v));
    }
    if let Some(v) = &params.circuit_short_name {
        query_parts.push(format!("circuit_short_name={}", v));
    }
    if let Some(v) = &params.country_code {
        query_parts.push(format!("country_code={}", v));
    }
    if let Some(v) = params.country_key {
        query_parts.push(format!("country_key={}", v));
    }
    if let Some(v) = &params.country_name {
        query_parts.push(format!("country_name={}", v));
    }
    if let Some(v) = &params.location {
        query_parts.push(format!("location={}", v));
    }
    if let Some(v) = &params.session_name {
        query_parts.push(format!("session_name={}", v));
    }
    if let Some(v) = &params.session_type {
        query_parts.push(format!("session_type={}", v));
    }
    if let Some(v) = params.year {
        query_parts.push(format!("year={}", v));
    }
    if let Some(v) = &params.date_start_from {
        query_parts.push(format!("date_start>={}", v));
    }
    if let Some(v) = &params.date_start_to {
        query_parts.push(format!("date_start<{}", v));
    }

    if query_parts.is_empty() {
        format!("{}/sessions", OPENF1_BASE_URL)
    } else {
        format!("{}/sessions?{}", OPENF1_BASE_URL, query_parts.join("&"))
    }
}

fn format_sessions(sessions: &[Session]) -> String {
    if sessions.is_empty() {
        return "No sessions found for the given parameters.".to_string();
    }
    let lines: Vec<String> = sessions
        .iter()
        .map(|s| {
            format!(
                "[{}] {}, {} ({}) | {} to {} (session={}, meeting={})",
                s.session_name,
                s.location,
                s.country_name,
                s.year,
                s.date_start,
                s.date_end,
                s.session_key,
                s.meeting_key,
            )
        })
        .collect();
    lines.join("\n")
}

pub async fn get_sessions(params: SessionParams) -> Result<Vec<Session>> {
    let url = build_sessions_url(&params);
    let client = reqwest::Client::new();
    let res = client.get(&url).send().await?;

    if let Err(err) = res.error_for_status_ref() {
        return Err(err.into());
    }

    Ok(res.json::<Vec<Session>>().await?)
}

pub async fn get_session_details(params: SessionParams) -> Result<String> {
    let sessions = get_sessions(params).await?;
    Ok(format_sessions(&sessions))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_sessions_url() {
        let params = SessionParams {
            country_name: Some("Belgium".to_string()),
            session_name: Some("Sprint Qualifying".to_string()),
            year: Some(2023),
            ..Default::default()
        };
        let url = build_sessions_url(&params);
        assert_eq!(
            url,
            "https://api.openf1.org/v1/sessions?country_name=Belgium&session_name=Sprint Qualifying&year=2023"
        );
    }

    #[test]
    fn test_format_sessions() {
        let sessions = vec![Session {
            circuit_key: 7,
            circuit_short_name: "Spa-Francorchamps".to_string(),
            country_code: "BEL".to_string(),
            country_key: 16,
            country_name: "Belgium".to_string(),
            date_end: "2023-07-29T15:35:00+00:00".to_string(),
            date_start: "2023-07-29T15:05:00+00:00".to_string(),
            gmt_offset: "02:00:00".to_string(),
            location: "Spa-Francorchamps".to_string(),
            meeting_key: 1216,
            session_key: 9140,
            session_name: "Sprint Qualifying".to_string(),
            session_type: "Sprint Qualifying".to_string(),
            year: 2023,
        }];
        let output = format_sessions(&sessions);
        assert_eq!(
            output,
            "[Sprint Qualifying] Spa-Francorchamps, Belgium (2023) | 2023-07-29T15:05:00+00:00 to 2023-07-29T15:35:00+00:00 (session=9140, meeting=1216)"
        );
    }

    #[test]
    fn test_format_sessions_empty() {
        let output = format_sessions(&[]);
        assert_eq!(output, "No sessions found for the given parameters.");
    }
}
