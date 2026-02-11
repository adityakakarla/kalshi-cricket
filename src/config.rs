use anyhow::Result;
use directories::ProjectDirs;
use figment::{
    Figment,
    providers::{Format, Toml},
};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct AppConfig {
    grok_api_key: String,
}

fn get_config_path() -> Result<PathBuf> {
    let project_directory =
        ProjectDirs::from("com", "fuji", "fuji").expect("Failed to get project directory");

    let path = project_directory.config_dir().join("config.toml");
    let parent = path.parent().expect("Failed to get parent directory");
    fs::create_dir_all(parent)?;
    Ok(path)
}

fn load_config() -> Result<AppConfig> {
    let path = get_config_path()?;
    Figment::new()
        .merge(Toml::file(path))
        .extract::<AppConfig>()
        .map_err(anyhow::Error::from)
}

pub fn get_api_key() -> Result<String> {
    let config = load_config()?;
    Ok(config.grok_api_key)
}

pub fn set_api_key(api_key: &String) -> Result<()> {
    let mut config = load_config().unwrap_or(AppConfig {
        grok_api_key: String::new(),
    });
    config.grok_api_key = api_key.clone();
    let path = get_config_path()?;
    fs::write(path, toml::to_string(&config)?).map_err(anyhow::Error::from)
}

pub fn view_key() -> Result<()> {
    let config = load_config().unwrap_or(AppConfig {
        grok_api_key: String::new(),
    });
    println!("API Key: {}", config.grok_api_key);
    Ok(())
}
