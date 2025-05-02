use crate::api::auth::AuthManager;
use crate::api::client::AniListClient;
use crate::data::database::Database;
use crate::ui::AniListApp;
use crate::utils::config::{load_config, Config};
use crate::utils::error::AppError;
use log::{debug, error, info};
use std::sync::Arc;

pub struct App {
    config: Config,
    client: AniListClient,
    db: Database,
    auth_manager: Option<AuthManager>,
}

impl App {
    pub async fn new() -> Result<Self, AppError> {
        info!("Initializing application...");

        // Load configuration
        let config = load_config().unwrap_or_else(|e| {
            error!("Failed to load config, using defaults: {}", e);
            Config::default()
        });

        debug!(
            "Config loaded: theme={}, language={}",
            config.theme, config.language
        );

        // Initialize database
        let db = Database::new()?;
        info!("Database initialized");

        // Create auth manager
        let auth_manager = AuthManager::new(config.auth_config.clone(), db.clone());
        debug!("Auth manager created");

        // Initialize API client with auth manager
        let client = AniListClient::with_auth_manager(auth_manager.clone());
        info!("API client initialized");

        Ok(Self {
            config,
            client,
            db,
            auth_manager: Some(auth_manager),
        })
    }

    pub fn get_api_client(&self) -> &AniListClient {
        &self.client
    }

    pub fn get_auth_manager(&self) -> Option<AuthManager> {
        self.auth_manager.clone()
    }

    pub fn _get_database(&self) -> &Database {
        &self.db
    }

    pub fn _get_config(&self) -> &Config {
        &self.config
    }

    pub fn create_ui_app(&self) -> AniListApp {
        info!("Creating UI application...");
        AniListApp::new(
            self.client.clone(),
            self.db.clone(),
            self.config.auth_config.clone(),
        )
    }
}
