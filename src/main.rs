mod api;
mod data;
mod ui;
mod utils;

use api::auth::AuthManager;
use api::client::AniListClient;
use data::database::Database;
use ui::AniListApp;
use utils::config::{Config, load_config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = load_config().unwrap_or_else(|_| {
        println!("Failed to load config, using defaults");
        Config::default()
    });

    // Initialize database
    let db = Database::new()?;

    // Initialize API client
    let api_client = AniListClient::new();

    // Check for stored authentication
    let auth_info = db.get_auth()?;
    let authenticated_client = if let Some((user_id, token, _)) = auth_info {
        println!("Loaded stored authentication for user ID: {}", user_id);
        AniListClient::with_token(token)
    } else {
        api_client
    };

    // Initialize authentication manager if needed
    let auth_config = api::auth::AuthConfig {
        client_id: config.auth_config.client_id.clone(),
        client_secret: config.auth_config.client_secret.clone(),
        redirect_uri: config.auth_config.redirect_uri.clone(),
    };

    let auth_manager = AuthManager::new(auth_config);

    // Start the UI
    let app = AniListApp::new(authenticated_client, db, auth_manager);

    // Start the application - this will need to be implemented properly
    // with iced's run method
    println!("Starting AniList Desktop Client...");

    // Placeholder for actual application launch
    // In a complete app, you'd use something like:
    // iced::run(Settings::with_flags(app))?;

    Ok(())
}
