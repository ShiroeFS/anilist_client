use chrono::{DateTime, Duration, Utc};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use url::Url;

use crate::data::database::Database;
use crate::utils::error::AppError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone)]
pub struct AuthManager {
    client: BasicClient,
    config: AuthConfig,
    db: Arc<Mutex<Database>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl AuthToken {
    pub fn is_expired(&self) -> bool {
        if let Some(expires_in) = self.expires_in {
            let now = Utc::now();
            let expires_at = self.created_at + Duration::seconds(expires_in as i64);

            // Consider the token expired if it's within 5 minutes of expiration
            now + Duration::minutes(5) > expires_at
        } else {
            // If we don't know when it expires, assume it's not expired
            false
        }
    }
}

impl AuthManager {
    pub fn new(config: AuthConfig, db: Database) -> Self {
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

        Self {
            client,
            config,
            db: Arc::new(Mutex::new(db)),
        }
    }

    pub async fn authenticate(&self) -> Result<AuthToken, AppError> {
        // Create oneshot channel for the auth callback
        let (tx, rx) = oneshot::channel();
        let tx = Arc::new(Mutex::new(Some(tx)));

        // Create the authorization URL with state for CSRF protection
        let (auth_url, csrf_token) = self
            .client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("read".to_string()))
            .add_scope(Scope::new("write".to_string()))
            .url();

        // Spawn a background task to start a server for the callback
        let tx_clone = Arc::clone(&tx);
        let csrf_token_secret = csrf_token.secret().clone();

        let handle = tokio::spawn(async move {
            let server_result = Self::start_auth_server(tx_clone, csrf_token_secret).await;
            if let Err(e) = server_result {
                eprintln!("Error in auth server: {}", e);
            }
        });

        // Open the browser for the user to authenticate
        if webbrowser::open(auth_url.as_str()).is_err() {
            println!("Failed to open browser automatically. Please open this URL manually:");
            println!("{}", auth_url);
        }

        // Wait for the callback to complete
        let code = rx
            .await
            .map_err(|_| AppError::ApiError("Authentication was cancelled".into()))?;

        // Once we have the code, we can shut down the server task
        handle.abort();

        // Exchange the code for a token
        let token_result = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(async_http_client)
            .await
            .map_err(|e| AppError::ApiError(format!("Token exchange failed: {}", e)))?;

        //Create our token object
        let auth_token = AuthToken {
            access_token: token_result.access_token().secret().clone(),
            token_type: format!("{:?}", token_result.token_type()),
            expires_in: token_result.expires_in().map(|d| d.as_secs()),
            refresh_token: token_result.refresh_token().map(|t| t.secret().clone()),
            created_at: Utc::now(),
        };

        // Get user ID from the API using the token
        let user_id = match self.get_user_id(&auth_token.access_token).await {
            Ok(id) => id,
            Err(e) => return Err(AppError::ApiError(format!("Failed to get user ID: {}", e))),
        };

        // Store the token in the database
        let expires_at = auth_token
            .expires_in
            .map(|secs| auth_token.created_at + Duration::seconds(secs as i64));

        if let Ok(mut db) = self.db.lock() {
            db.save_auth(
                user_id,
                &auth_token.access_token,
                auth_token.refresh_token.as_deref(),
                expires_at,
            )
            .map_err(|e| AppError::DatabaseError(format!("Failed to save auth token: {}", e)))?;
        } else {
            return Err(AppError::ApiError("Failed to access database".into()));
        }

        Ok(auth_token)
    }

    // Start a temporary HTTP server to receive the OAuth callback
    async fn start_auth_server(
        tx: Arc<Mutex<Option<oneshot::Sender<String>>>>,
        expected_state: String,
    ) -> Result<(), AppError> {
        // Bind to the redirect URI port
        let listener = TcpListener::bind("127.0.0.1:8080")
            .map_err(|e| AppError::ApiError(format!("Failed to start auth server: {}", e)))?;

        listener
            .set_nonblocking(true)
            .map_err(|e| AppError::ApiError(format!("Failed to set non-blocking mode: {}", e)))?;

        // Convert to async listener using async-std
        let listener = async_std::net::TcpListener::from_std(listener)
            .map_err(|e| AppError::ApiError(format!("Failed to create async listener: {}", e)))?;

        println!("Waiting for authentication callback at http://localhost:8080/callback...");

        // Accept one connection
        let (stream, _) = listener
            .accept()
            .await
            .map_err(|e| AppError::ApiError(format!("Failed to accept connection: {}", e)))?;

        let mut reader = BufReader::new(&stream);
        let mut request_line = String::new();

        // Async read from the stream
        reader
            .read_line(&mut request_line)
            .await
            .map_err(|e| AppError::ApiError(format!("Failed to read request: {}", e)))?;

        // Extract the authorization code from the request URL
        let redirect_url = request_line
            .split_whitespace()
            .nth(1)
            .ok_or_else(|| AppError::ApiError("Invalid request format".into()))?;

        let url = Url::parse(&format!("http://localhost{}", redirect_url))
            .map_err(|e| AppError::ApiError(format!("Failed to parse redirect URL: {}", e)))?;

        // Extract the code and state from the URL
        let code = url
            .query_pairs()
            .find(|(key, _)| key == "code")
            .map(|(_, value)| value.into_owned())
            .ok_or_else(|| AppError::ApiError("No code in the response".into()))?;

        let state = url
            .query_pairs()
            .find(|(key, _)| key == "state")
            .map(|(_, value)| value.into_owned())
            .ok_or_else(|| AppError::ApiError("No state in the response".into()))?;

        // Verify the CSRF token
        if state != expected_state {
            return Err(AppError::ApiError(
                "CSRF token mismatch, possible security breach".into(),
            ));
        }

        // Send a success message to the browser
        let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body><h1>Successfully authenticated with AniList</h1><p>You can close this window now and return to the application.</p></body></html>";

        stream
            .write_all(response.as_bytes())
            .await
            .map_err(|e| AppError::ApiError(format!("Failed to send response: {}", e)))?;

        // Send the code through the channel
        if let Ok(mut tx_guard) = tx.lock() {
            if let Some(sender) = tx_guard.take() {
                if sender.send(code).is_err() {
                    return Err(AppError::ApiError("Failed to communicate auth code".into()));
                }
            }
        }

        Ok(())
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<AuthToken, AppError> {
        // Create a request to refresh the token
        let token_result = self
            .client
            .exchange_refresh_token(&oauth2::RefreshToken::new(refresh_token.to_string()))
            .request_async(async_http_client)
            .await
            .map_err(|e| AppError::ApiError(format!("Token refresh failed: {}", e)))?;

        // Create our token object
        let auth_token = AuthToken {
            access_token: token_result.access_token().secret().clone(),
            token_type: format!("{:?}", token_result.token_type()),
            expires_in: token_result.expires_in().map(|d| d.as_secs()),
            refresh_token: token_result.refresh_token().map(|t| t.secret().clone()),
            created_at: Utc::now(),
        };

        // Get user ID from the stored data
        let user_id = {
            if let Ok(db) = self.db.lock() {
                if let Ok(Some((id, _, _, _))) = db.get_auth() {
                    id
                } else {
                    // If we can't get the user ID, try to get it from the API
                    self.get_user_id(&auth_token.access_token).await?
                }
            } else {
                return Err(AppError::ApiError("Failed to access database".into()));
            }
        };

        // Store the refreshed token
        let expires_at = auth_token
            .expires_in
            .map(|secs| auth_token.created_at + Duration::seconds(secs as i64));

        if let Ok(mut db) = self.db.lock() {
            db.save_auth(
                user_id,
                &auth_token.access_token,
                auth_token.refresh_token.as_deref(),
                expires_at,
            )
            .map_err(|e| {
                AppError::DatabaseError(format!("Failed to save refreshed token: {}", e))
            })?;
        } else {
            return Err(AppError::ApiError("Failed to access database".into()));
        }

        Ok(auth_token)
    }

    pub async fn ensure_authenticated(&self) -> Result<AuthToken, AppError> {
        // Try to get stored auth data
        let auth_info = {
            if let Ok(db) = self.db.lock() {
                db.get_auth()
                    .map_err(|e| AppError::DatabaseError(format!("Database error: {}", e)))?
            } else {
                return Err(AppError::ApiError("Failed to access database".into()));
            }
        };

        if let Some((user_id, access_token, refresh_token, expires_at)) = auth_info {
            // Check if token has expired
            let is_expired = match expires_at {
                Some(expiry) => Utc::now() + Duration::minutes(5) > expiry,
                None => false, // If we don't know when it expires, assume it's valid
            };

            if !is_expired {
                // Token is still valid
                return Ok(AuthToken {
                    access_token,
                    token_type: "Bearer".to_string(),
                    expires_in: expires_at
                        .map(|exp| (exp - Utc::now()).num_seconds().max(0) as u64),
                    refresh_token,
                    created_at: Utc::now() - Duration::seconds(60), // Just an approximation
                });
            }

            // Token is expired, try to refresh if we have a refresh token
            if let Some(refresh_token_str) = refresh_token {
                match self.refresh_token(&refresh_token_str).await {
                    Ok(new_token) => return Ok(new_token),
                    Err(e) => {
                        eprintln!("Failed to refresh token: {}", e);
                        // Fall through to re-authentication
                    }
                }
            }
        }

        // If we get here, we need to authenticate from scratch
        self.authenticate().await
    }

    pub async fn logout(&self) -> Result<(), AppError> {
        // Clear auth data from database
        if let Ok(mut db) = self.db.lock() {
            db.clear_auth().map_err(|e| {
                AppError::DatabaseError(format!("Failed to clear auth data: {}", e))
            })?;
        } else {
            return Err(AppError::ApiError("Failed to access database".into()));
        }

        Ok(())
    }

    // Helper function to get the user ID for the authenticated user
    async fn get_user_id(&self, access_token: &str) -> Result<i32, AppError> {
        let client = reqwest::Client::new();
        let response = client
            .post("https://graphql.anilist.co")
            .header("Authorization", format!("Bearer {}", access_token))
            .json(&serde_json::json!({
                "query": "query { Viewer { id } }"
            }))
            .send()
            .await
            .map_err(|e| AppError::NetworkError(format!("Network error: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            return Err(AppError::ApiError(format!("API error: HTTP {}", status)));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::ApiError(format!("Failed to parse response: {}", e)))?;

        // Check for GraphQL errors
        if let Some(errors) = json.get("errors") {
            if let Some(errors_array) = errors.as_array() {
                if !errors_array.is_empty() {
                    let error_msg = errors_array
                        .iter()
                        .filter_map(|e| e.get("message").and_then(|m| m.as_str()))
                        .collect::<Vec<&str>>()
                        .join(", ");

                    return Err(AppError::ApiError(format!("GraphQL error: {}", error_msg)));
                }
            }
        }

        // Extract the ID from the response
        if let Some(id) = json
            .get("data")
            .and_then(|data| data.get("Viewer"))
            .and_then(|viewer| viewer.get("id"))
            .and_then(|id| id.as_i64())
        {
            Ok(id as i32)
        } else {
            Err(AppError::ApiError(
                "Failed to extract user ID from response".into(),
            ))
        }
    }
}
