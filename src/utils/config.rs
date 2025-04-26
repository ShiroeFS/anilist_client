use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub auth_config: AuthConfig,
    pub theme: String,
    pub language: String,
    pub offline_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auth_config: AuthConfig {
                client_id: "your-client-id".to_string(),
                client_secret: "your-client-secret".to_string(),
                redirect_uri: "http://localhost:8080/callback".to_string(),
            },
            theme: "default".to_string(),
            language: "en".to_string(),
            offline_mode: false,
        }
    }
}

pub fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let proj_dirs = ProjectDirs::from("me", "camniel", "AniListClient")
        .ok_or_else(|| "Could not determine project directory".to_string())?;

    let config_dir = proj_dirs.config_dir();
    std::fs::create_dir_all(config_dir)?;

    Ok(config_dir.join("config.json"))
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        let default_config = Config::default();
        save_config(&default_config)?;
        return Ok(default_config);
    }

    let mut file = File::open(config_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let config: Config = serde_json::from_str(&contents)?;
    Ok(config)
}

pub fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path()?;
    let json = serde_json::to_string_pretty(config)?;

    let mut file = File::create(config_path)?;
    file.write_all(json.as_bytes())?;

    Ok(())
}
