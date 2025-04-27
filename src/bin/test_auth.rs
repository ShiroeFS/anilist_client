use anilist_client::api::auth::{AuthConfig, AuthManager};
use anilist_client::data::database::Database;
use anilist_client::utils::config::load_config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = load_config().unwrap_or_else(|_| {
        println!("Failed to load config, using defaults");
        anilist_client::utils::config::Config::default()
    });

    println!("Starting OAuth2 flow test with AniList...");

    // Create auth config from loaded config
    let auth_config = AuthConfig {
        client_id: config.auth_config.client_id.clone(),
        client_secret: config.auth_config.client_secret.clone(),
        redirect_uri: config.auth_config.redirect_uri.clone(),
    };

    // Initialize auth manager
    let auth_manager = AuthManager::new(auth_config);

    // Initialize database for token storage
    let db = Database::new()?;

    // Try to authenticate
    println!("Starting authentication process...");
    println!("This will open a browser window. Please sign in to AniList when prompted.");
    match auth_manager.authenticate().await {
        Ok(token) => {
            println!("Authentication successful!");
            println!("Access token: {}", token.access_token);
            println!("Token type: {}", token.token_type);

            if let Some(expires_in) = token.expires_in {
                println!("Token expires in: {} seconds", expires_in);
            }

            if let Some(refresh_token) = &token.refresh_token {
                println!("Refresh token received");

                // Store the token in the database
                println!("Attempting to get user ID with the token...");

                // Make a test API request to verify the token
                let user_id = get_user_id(&token.access_token).await?;
                println!("User ID: {}", user_id);

                // Save the token to database
                db.save_auth(
                    user_id,
                    &token.access_token,
                    Some(refresh_token),
                    token.expires_in.map(|secs| {
                        let now = chrono::Utc::now();
                        now + chrono::Duration::seconds(secs as i64)
                    }),
                )?;

                println!("Token saved to database");

                // Verify we can retrieve the token
                let auth_data = db.get_auth()?;
                if let Some((stored_user_id, _, _, _)) = auth_data {
                    println!(
                        "Successfully retrieved token from database for user ID: {}",
                        stored_user_id
                    );
                } else {
                    println!("Failed to retrieve token from database");
                }
            } else {
                println!("No refresh token received");
            }
        }
        Err(e) => {
            println!("Authentication failed: {}", e);
        }
    }

    Ok(())
}

// Helper function to get the user ID using the token
async fn get_user_id(access_token: &str) -> Result<i32, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .post("https://graphql.anilist.co")
        .header("Authorization", format!("Bearer {}", access_token))
        .json(&serde_json::json!({
            "query": "query { Viewer { id name } }"
        }))
        .send()
        .await?;

    let status = response.status();
    let text = response.text().await?;

    println!("API response status: {}", status);
    println!("API response body: {}", text);

    // Parse the response JSON
    let json: serde_json::Value = serde_json::from_str(&text)?;

    // Extract the ID from response
    if let Some(id) = json
        .get("data")
        .and_then(|data| data.get("Viewer"))
        .and_then(|viewer| viewer.get("id"))
        .and_then(|id| id.as_i64())
    {
        Ok(id as i32)
    } else {
        Err("Failed to extract user ID from response".into())
    }
}
