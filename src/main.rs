use anilist_client::app::App;
use anilist_client::ui::AniListApp;
use anilist_client::utils::logging;
use anyhow::{Context, Result};
use log::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // Set up logger
    logging::setup_logger().context("Failed to initialize logger")?;

    info!("Starting AniList Desktop Client v0.1.0");

    // Initialize the app
    let app = App::new()
        .await
        .context("Failed to initialize application")?;

    // Check if CLI arguments were provided
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        // Handle CLI arguments
        match args[1].as_str() {
            "--test-auth" => {
                info!("Testing authentication flow...");

                let auth_manager = app
                    .get_auth_manager()
                    .context("Failed to get auth manager")?;

                match auth_manager.authenticate().await {
                    Ok(token) => {
                        info!("Authentication successful!");
                        info!("Access token: {}", token.access_token);
                        if let Some(refresh_token) = &token.refresh_token {
                            info!("Refresh token received");
                        }
                        info!("Token expires in: {:?} seconds", token.expires_in);
                    }
                    Err(e) => {
                        error!("Authentication failed: {}", e);
                        return Err(anyhow::anyhow!("Authentication failed: {}", e));
                    }
                }
            }
            "--test-api" => {
                info!("Testing API access...");

                let client = app.get_api_client();

                // Try to authenticate if needed
                let is_authenticated = client.is_authenticated().await;

                if !is_authenticated {
                    info!("Not authenticated. Some features may not work.");
                } else {
                    info!("Successfully authenticated.");

                    // Try to get viewer data
                    match client.get_viewer().await {
                        Ok(data) => {
                            if let Some(viewer) = data.viewer {
                                info!("Logged in as: {}", viewer.name);
                            }
                        }
                        Err(e) => {
                            error!("Error fetching user data: {}", e);
                        }
                    }
                }

                // Test search functionality
                info!("Testing search with query 'Shingeki no Kyojin'...");
                match client
                    .search_anime("Shingeki no Kyojin".to_string(), Some(1), Some(5))
                    .await
                {
                    Ok(results) => {
                        if let Some(page) = results.page {
                            if let Some(media_list) = page.media {
                                info!("Found {} results:", media_list.len());
                                for (i, media) in media_list.iter().enumerate() {
                                    if let Some(media) = media {
                                        if let Some(title) = &media.title {
                                            info!(
                                                "{}. {} (ID: {})",
                                                i + 1,
                                                title
                                                    .romaji
                                                    .as_ref()
                                                    .unwrap_or(&"Unknown".to_string()),
                                                media.id
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Search failed: {}", e);
                    }
                }
            }
            "--help" => {
                println!("AniList Desktop Client");
                println!("Usage:");
                println!("  anilist_client            Start the GUI application");
                println!("  anilist_client --test-auth Test the authentication flow");
                println!("  anilist_client --test-api  Test the API connection");
                println!("  anilist_client --help      Show this help message");
            }
            _ => {
                println!("Unknown command: {}", args[1]);
                println!("Use --help to see available commands");
                return Err(anyhow::anyhow!("Unknown command: {}", args[1]));
            }
        }
    } else {
        // Launch the GUI application
        info!("Starting GUI application...");

        let ui_app = app.create_ui_app();

        if let Err(e) = AniListApp::launch() {
            error!("Application error: {}", e);
            return Err(anyhow::anyhow!("Failed to launch UI: {}", e));
        }
    }

    Ok(())
}
