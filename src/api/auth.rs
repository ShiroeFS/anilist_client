use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use url::Url;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Clone)]
pub struct AuthManager {
    client: BasicClient,
    config: AuthConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
}

impl AuthManager {
    pub fn new(config: AuthConfig) -> Self {
        let client = BasicClient::new(
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
            AuthUrl::new("https://anilist.co/api/v2/oauth/authorize".to_string())
                .expect("Failed to parse auth URL"),
            Some(
                TokenUrl::new("https://anilist.co/api/v2/oauth/token".to_string())
                    .expect("Failed to parse token URL"),
            ),
        )
        .set_redirect_uri(
            RedirectUrl::new(config.redirect_uri.clone()).expect("Failed to parse redirect URL"),
        );

        Self { client, config }
    }

    pub async fn authenticate(&self) -> Result<AuthToken, Box<dyn std::error::Error>> {
        // Create the authorization URL
        let (auth_url, csrf_token) = self
            .client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("read".to_string()))
            .add_scope(Scope::new("write".to_string()))
            .url();

        // Open the browser for the user to authenticate
        println!("Opening the browser for authentication...");
        if webbrowser::open(auth_url.as_str()).is_err() {
            println!("Failed to open browser automatically. Please open this URL manually:");
            println!("{}", auth_url);
        }

        // Start a local web server to receive callback
        let listener = TcpListener::bind("127.0.0.1:8080")?;
        println!("Waiting for authentication callback...");

        // Wait for the callback
        let (mut stream, _) = listener.accept()?;
        let mut reader = BufReader::new(&stream);
        let mut request_line = String::new();
        reader.read_line(&mut request_line)?;

        // Extract the authorization code from the request URL
        let redirect_url = request_line
            .split_whitespace()
            .nth(1)
            .ok_or("Invalid request")?;
        let url = Url::parse(&format!("http://localhost{}", redirect_url))?;

        // Extract the code and state from the URL
        let code = url
            .query_pairs()
            .find(|(key, _)| key == "code")
            .map(|(_, value)| value.into_owned())
            .ok_or("No code in the response")?;

        let state = url
            .query_pairs()
            .find(|(key, _)| key == "state")
            .map(|(_, value)| value.into_owned())
            .ok_or("No state in the response")?;

        // Verify the CSRF token
        if &state != csrf_token.secret() {
            return Err("CSRF token mismatch".into());
        }

        // Send a success message to the browser
        let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body><h1>Successfully authenticated</h1><p>You can close this window now.</p></body></html>";
        stream.write_all(response.as_bytes())?;

        // Exchange the code for a token
        let token_result = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(async_http_client)
            .await?;

        // Create and return our token object
        let auth_token = AuthToken {
            access_token: token_result.access_token().secret().clone(),
            token_type: format!("{:?}", token_result.token_type()),
            expires_in: token_result.expires_in().map(|d| d.as_secs()),
            refresh_token: token_result.refresh_token().map(|t| t.secret().clone()),
        };

        Ok(auth_token)
    }

    pub async fn refresh_token(
        &self,
        refresh_token: String,
    ) -> Result<AuthToken, Box<dyn std::error::Error>> {
        // Create a request to refresh the token
        let token_result = self
            .client
            .exchange_refresh_token(&oauth2::RefreshToken::new(refresh_token))
            .request_async(async_http_client)
            .await?;

        // Create and return our token object
        let auth_token = AuthToken {
            access_token: token_result.access_token().secret().clone(),
            token_type: format!("{:?}", token_result.token_type()),
            expires_in: token_result.expires_in().map(|d| d.as_secs()),
            refresh_token: token_result.refresh_token().map(|t| t.secret().clone()),
        };

        Ok(auth_token)
    }

    pub fn is_token_expired(token: &AuthToken) -> bool {
        if let Some(expires_in) = token.expires_in {
            // Check if token is about to expire (within 5 minutes)
            if expires_in <= 300 {
                return true;
            }
        }

        false
    }

    pub async fn ensure_authenticated(
        &self,
        db: &mut crate::data::database::Database,
    ) -> Result<AuthToken, Box<dyn std::error::Error>> {
        // Try to get stored auth token
        let auth_info = db.get_auth()?;

        if let Some((user_id, access_token, refresh_token, _)) = auth_info {
            // Create auth token from stored data
            let token = AuthToken {
                access_token,
                token_type: "Bearer".to_string(),
                expires_in: None, // We might not have this stored
                refresh_token,
            };

            // If we have a refresh token and the access token might be expired
            if let Some(refresh_token_str) = &token.refresh_token {
                // Try to refresh the token
                match self.refresh_token(refresh_token_str.clone()).await {
                    Ok(new_token) => {
                        // Save the new token
                        db.save_auth(
                            user_id,
                            &new_token.access_token,
                            new_token.refresh_token.as_deref(),
                            new_token.expires_in.map(|secs| {
                                let now = chrono::Utc::now();
                                now + chrono::Duration::seconds(secs as i64)
                            }),
                        )?;
                        return Ok(new_token);
                    }
                    Err(_) => {
                        // If refresh fails, we'll need to authenticate from scratch
                    }
                }
            } else {
                // If we have a token but no refresh token, return what we have
                return Ok(token);
            }
        }

        // If we got here, we need to authenticate from scratch
        let token = self.authenticate().await?;

        // Get user ID from the API using the token
        let user_id = match Self::get_user_id(&token.access_token).await {
            Ok(id) => id,
            Err(e) => return Err(format!("Failed to get user ID: {}", e).into()),
        };

        // Store the new token

        db.save_auth(
            user_id,
            &token.access_token,
            token.refresh_token.as_deref(),
            token.expires_in.map(|secs| {
                let now = chrono::Utc::now();
                now + chrono::Duration::seconds(secs as i64)
            }),
        )?;

        Ok(token)
    }

    // Helper function to get the user ID for the authenticated user
    async fn get_user_id(access_token: &str) -> Result<i32, Box<dyn std::error::Error>> {
        // This would use the API client to call the Viewer query
        // For now, we'll implement a simplified version
        let client = reqwest::Client::new();
        let response = client
            .post("https://graphql.anilist.co")
            .header("Authorization", format!("Bearer {}", access_token))
            .json(&serde_json::json!({
                "query": "query { Viewer { id } }"
            }))
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;

        // Extract the ID from the response
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
}
