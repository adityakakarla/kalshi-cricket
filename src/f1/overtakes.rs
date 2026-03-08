use anyhow::Result;
use serde::{Deserialize, Serialize};

const OPENF1_BASE_URL: &str = "https://api.openf1.org/v1";

#[derive(Debug, Serialize, Deserialize)]
pub struct Overtake {
    pub date: String,
    pub meeting_key: i32,
    pub overtaken_driver_number: i32,
    pub overtaking_driver_number: i32,
    pub position: i32,
    pub session_key: i32,
}

#[derive(Debug, Default)]
pub struct OvertakesParams {
    pub session_key: Option<String>,
    pub meeting_key: Option<String>,
    pub overtaking_driver_number: Option<i32>,
    pub overtaken_driver_number: Option<i32>,
    pub position: Option<i32>,
}

fn build_overtakes_url(params: &OvertakesParams) -> String {
    let mut query_parts: Vec<String> = Vec::new();

    if let Some(v) = &params.session_key {
        query_parts.push(format!("session_key={}", v));
    }
    if let Some(v) = &params.meeting_key {
        query_parts.push(format!("meeting_key={}", v));
    }
    if let Some(v) = params.overtaking_driver_number {
        query_parts.push(format!("overtaking_driver_number={}", v));
    }
    if let Some(v) = params.overtaken_driver_number {
        query_parts.push(format!("overtaken_driver_number={}", v));
    }
    if let Some(v) = params.position {
        query_parts.push(format!("position={}", v));
    }

    if query_parts.is_empty() {
        format!("{}/overtakes", OPENF1_BASE_URL)
    } else {
        format!("{}/overtakes?{}", OPENF1_BASE_URL, query_parts.join("&"))
    }
}

fn format_overtakes(overtakes: &[Overtake]) -> String {
    if overtakes.is_empty() {
        return "No overtakes found for the given parameters.".to_string();
    }
    let lines: Vec<String> = overtakes
        .iter()
        .map(|o| {
            format!(
                "[{}] Driver {} overtook driver {} to P{} (session={})",
                o.date,
                o.overtaking_driver_number,
                o.overtaken_driver_number,
                o.position,
                o.session_key,
            )
        })
        .collect();
    lines.join("\n")
}

pub async fn get_overtakes(params: OvertakesParams) -> Result<Vec<Overtake>> {
    let url = build_overtakes_url(&params);
    let client = reqwest::Client::new();
    let res = client.get(&url).send().await?;

    if let Err(err) = res.error_for_status_ref() {
        return Err(err.into());
    }

    Ok(res.json::<Vec<Overtake>>().await?)
}

pub async fn get_overtakes_details(params: OvertakesParams) -> Result<String> {
    let overtakes = get_overtakes(params).await?;
    Ok(format_overtakes(&overtakes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_overtakes_url_no_params() {
        let params = OvertakesParams::default();
        let url = build_overtakes_url(&params);
        assert_eq!(url, "https://api.openf1.org/v1/overtakes");
    }

    #[test]
    fn test_build_overtakes_url_with_params() {
        let params = OvertakesParams {
            session_key: Some("9636".to_string()),
            overtaking_driver_number: Some(63),
            overtaken_driver_number: Some(4),
            position: Some(1),
            ..Default::default()
        };
        let url = build_overtakes_url(&params);
        assert_eq!(
            url,
            "https://api.openf1.org/v1/overtakes?session_key=9636&overtaking_driver_number=63&overtaken_driver_number=4&position=1"
        );
    }

    #[test]
    fn test_format_overtakes() {
        let overtakes = vec![Overtake {
            date: "2024-11-03T15:50:07.565000+00:00".to_string(),
            meeting_key: 1249,
            overtaken_driver_number: 4,
            overtaking_driver_number: 63,
            position: 1,
            session_key: 9636,
        }];
        let output = format_overtakes(&overtakes);
        assert_eq!(
            output,
            "[2024-11-03T15:50:07.565000+00:00] Driver 63 overtook driver 4 to P1 (session=9636)"
        );
    }

    #[test]
    fn test_format_overtakes_empty() {
        let output = format_overtakes(&[]);
        assert_eq!(output, "No overtakes found for the given parameters.");
    }
}
