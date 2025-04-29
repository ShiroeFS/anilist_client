use anilist_client::app::App;
use anilist_client::ui::AniListApp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the app
    let app = App::new().await?;

    println!("AniList Desktop Client v0.1.0");

    // Check if CLI arguments were provided
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        // Handle CLI arguments
        match args[1].as_str() {
            "--test-auth" => {
                println!("Testing authentication flow...");

                if let Some(auth_manager) = app.get_auth_manager() {
                    match auth_manager.authenticate().await {
                        Ok(token) => {
                            println!("Authentication successful!");
                            println!("Access token: {}", token.access_token);
                            if let Some(refresh_token) = token.refresh_token {
                                println!("Refresh token received");
                            }
                            println!("Token expires in: {:?} seconds", token.expires_in);
                        }
                        Err(e) => {
                            eprintln!("Authentication failed: {}", e);
                            return Err(e.into());
                        }
                    }
                } else {
                    eprintln!("Could not initialize auth manager");
                    return Err("Auth manager initialization failed".into());
                }
            }
            "--test-api" => {
                println!("Testing API access...");

                let client = app.get_api_client();

                // Try to authenticate if needed
                let is_authenticated = client.is_authenticated().await;

                if !is_authenticated {
                    println!("Not authenticated. Some features may not work.");
                } else {
                    println!("Successfully authenticated.");

                    // Try to get viewer data
                    match client.get_viewer().await {
                        Ok(data) => {
                            if let Some(viewer) = data.viewer {
                                println!("Logged in as: {}", viewer.name);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error fetching user data: {}", e);
                        }
                    }
                }

                // Test search functionality
                println!("\nTesting search with query 'Shingeki no Kyojin'...");
                match client
                    .search_anime("Shingeki no Kyojin".to_string(), Some(1), Some(5))
                    .await
                {
                    Ok(results) => {
                        if let Some(page) = results.page {
                            if let Some(media_list) = page.media {
                                println!("Found {} results:", media_list.len());
                                for (i, media) in media_list.iter().enumerate() {
                                    if let Some(media) = media {
                                        if let Some(title) = &media.title {
                                            println!(
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
                        eprintln!("Search failed: {}", e);
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
            }
        }
    } else {
        // Launch the GUI application
        let ui_app = AniListApp::new(
            app.get_api_client(),
            app.get_database(),
            app.get_config().auth_config.clone(),
        );

        if let Err(e) = AniListApp::launch() {
            eprintln!("Application error: {}", e);
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to launch UI: {}", e),
            )));
        }
    }

    Ok(())
}
