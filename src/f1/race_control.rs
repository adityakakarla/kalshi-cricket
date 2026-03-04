use anyhow::Result;
use serde::{Deserialize, Serialize};

const OPENF1_BASE_URL: &str = "https://api.openf1.org/v1";

#[derive(Debug, Serialize, Deserialize)]
pub struct RaceControlEvent {
    pub category: Option<String>,
    pub date: String,
    pub driver_number: Option<i32>,
    pub flag: Option<String>,
    pub lap_number: Option<i32>,
    pub meeting_key: i32,
    pub message: Option<String>,
    pub qualifying_phase: Option<String>,
    pub scope: Option<String>,
    pub sector: Option<i32>,
    pub session_key: i32,
}

#[derive(Debug, Default)]
pub struct RaceControlParams {
    pub session_key: Option<String>,
    pub meeting_key: Option<String>,
    pub driver_number: Option<i32>,
    pub flag: Option<String>,
    pub category: Option<String>,
    pub lap_number: Option<i32>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub message: Option<String>,
    pub qualifying_phase: Option<String>,
    pub scope: Option<String>,
    pub sector: Option<i32>,
}

fn build_race_control_url(params: &RaceControlParams) -> String {
    let mut query_parts: Vec<String> = Vec::new();

    if let Some(v) = &params.session_key {
        query_parts.push(format!("session_key={}", v));
    }
    if let Some(v) = &params.meeting_key {
        query_parts.push(format!("meeting_key={}", v));
    }
    if let Some(v) = &params.date_from {
        query_parts.push(format!("date>={}", v));
    }
    if let Some(v) = &params.date_to {
        query_parts.push(format!("date<{}", v));
    }
    if let Some(v) = &params.message {
        query_parts.push(format!("message={}", v));
    }
    if let Some(v) = &params.qualifying_phase {
        query_parts.push(format!("qualifying_phase={}", v));
    }
    if let Some(v) = &params.scope {
        query_parts.push(format!("scope={}", v));
    }
    if let Some(v) = params.sector {
        query_parts.push(format!("sector={}", v));
    }
    if let Some(v) = params.driver_number {
        query_parts.push(format!("driver_number={}", v));
    }
    if let Some(v) = &params.flag {
        query_parts.push(format!("flag={}", v));
    }
    if let Some(v) = &params.category {
        query_parts.push(format!("category={}", v));
    }
    if let Some(v) = params.lap_number {
        query_parts.push(format!("lap_number={}", v));
    }

    if query_parts.is_empty() {
        format!("{}/race_control", OPENF1_BASE_URL)
    } else {
        format!("{}/race_control?{}", OPENF1_BASE_URL, query_parts.join("&"))
    }
}

fn format_race_control_events(events: &[RaceControlEvent]) -> String {
    if events.is_empty() {
        return "No race control events found for the given parameters.".to_string();
    }
    let lines: Vec<String> = events
        .iter()
        .map(|e| {
            format!(
                "[{}] Lap {}: {} (session={}, driver={:?})",
                e.date,
                e.lap_number.map_or("?".to_string(), |l| l.to_string()),
                e.message.as_deref().unwrap_or(""),
                e.session_key,
                e.driver_number,
            )
        })
        .collect();
    lines.join("\n")
}

pub async fn get_race_control(params: RaceControlParams) -> Result<Vec<RaceControlEvent>> {
    let url = build_race_control_url(&params);
    let client = reqwest::Client::new();
    let res = client.get(&url).send().await?;

    if let Err(err) = res.error_for_status_ref() {
        return Err(err.into());
    }

    Ok(res.json::<Vec<RaceControlEvent>>().await?)
}

pub async fn get_race_control_details(params: RaceControlParams) -> Result<String> {
    let events = get_race_control(params).await?;
    Ok(format_race_control_events(&events))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_race_control_url() {
        let params = RaceControlParams {
            flag: Some("BLACK AND WHITE".to_string()),
            driver_number: Some(1),
            date_from: Some("2023-01-01".to_string()),
            date_to: Some("2023-09-01".to_string()),
            ..Default::default()
        };
        let url = build_race_control_url(&params);
        assert_eq!(
            url,
            "https://api.openf1.org/v1/race_control?date>=2023-01-01&date<2023-09-01&driver_number=1&flag=BLACK AND WHITE"
        );
    }

    #[test]
    fn test_format_race_control_events() {
        let events = vec![RaceControlEvent {
            category: Some("Flag".to_string()),
            date: "2023-06-04T14:21:01+00:00".to_string(),
            driver_number: Some(1),
            flag: Some("BLACK AND WHITE".to_string()),
            lap_number: Some(59),
            meeting_key: 1211,
            message: Some(
                "BLACK AND WHITE FLAG FOR CAR 1 (VER) - TRACK LIMITS".to_string(),
            ),
            qualifying_phase: None,
            scope: Some("Driver".to_string()),
            sector: None,
            session_key: 9102,
        }];
        let output = format_race_control_events(&events);
        assert_eq!(
            output,
            "[2023-06-04T14:21:01+00:00] Lap 59: BLACK AND WHITE FLAG FOR CAR 1 (VER) - TRACK LIMITS (session=9102, driver=Some(1))"
        );
    }

    #[test]
    fn test_format_race_control_events_empty() {
        let output = format_race_control_events(&[]);
        assert_eq!(
            output,
            "No race control events found for the given parameters."
        );
    }
}
