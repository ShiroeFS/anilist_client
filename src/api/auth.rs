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
        _refresh_token: String,
    ) -> Result<AuthToken, Box<dyn std::error::Error>> {
        // Implement token refresh logic here
        // This would use the refresh_token to get a new access_token
        // For brevity, this is not fully implemented in this example
        todo!()
    }
}

// Usage example
async fn auth_example() -> Result<(), Box<dyn std::error::Error>> {
    // load config from environment or config file
    let auth_config = AuthConfig {
        client_id: "your-client-id".to_string(),
        client_secret: "your-client-secret".to_string(),
        redirect_uri: "http://localhost:8080/callback".to_string(),
    };

    let auth_manager = AuthManager::new(auth_config);
    let token = auth_manager.authenticate().await?;

    println!("Successfully authenticated!");
    println!("Access token: {}", token.access_token);

    // Store the token securely for future use
    // You might want to use a secure storage like keyring

    Ok(())
}
