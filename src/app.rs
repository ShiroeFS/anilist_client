use crate::api::client::AniListClient;
use crate::data::database::Database;
use crate::ui::AniListApp;
use crate::utils::config::{load_config, Config};
use crate::utils::error::AppError;

pub struct App {
    config: Config,
    client: AniListClient,
    db: Database,
}

impl App {
    pub async fn new() -> Result<Self, AppError> {
        // Load configuration
        let config = load_config().unwrap_or_else(|_| {
            println!("Failed to load config, using defaults");
            Config::default()
        });

        // Initialize database
        let db = Database::new()?;

        // Initialize API client
        let auth_info = db.get_auth()?;
        let client = if let Some((_, token, _, _)) = auth_info {
            AniListClient::with_token(token)
        } else {
            AniListClient::new()
        };

        Ok(Self { config, client, db })
    }

    pub fn get_api_client(&self) -> &AniListClient {
        &self.client
    }

    pub fn _get_database(&self) -> &Database {
        &self.db
    }

    pub fn _get_config(&self) -> &Config {
        &self.config
    }

    pub fn create_ui_app(&self) -> AniListApp {
        AniListApp::new(
            self.client.clone(),
            self.db.clone(),
            self.config.auth_config.clone(),
        )
    }
}
