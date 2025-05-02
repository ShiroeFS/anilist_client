use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("API Error: {0}")]
    ApiError(String),

    #[error("Database Error: {0}")]
    DatabaseError(String),

    #[error("Authentication Error: {0}")]
    AuthError(String),

    #[error("Configuration Error: {0}")]
    ConfigError(String),

    #[error("Network Error: {0}")]
    NetworkError(String),

    #[error("UI Error: {0}")]
    UiError(String),

    #[error("I/O Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization/Deserialization Error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Unknown Error: {0}")]
    UnknownError(String),
}

// Convert from rusqlite errors
impl From<rusqlite::Error> for AppError {
    fn from(error: rusqlite::Error) -> Self {
        AppError::DatabaseError(error.to_string())
    }
}

// Convert from reqwest errors
impl From<reqwest::Error> for AppError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            AppError::NetworkError(format!("Network timeout: {}", error))
        } else if error.is_connect() {
            AppError::NetworkError(format!("Connection failed: {}", error))
        } else {
            AppError::NetworkError(error.to_string())
        }
    }
}

// Convert from OAuth2 errors
impl From<oauth2::basic::BasicRequestTokenError<oauth2::reqwest::Error<reqwest::Error>>>
    for AppError
{
    fn from(
        error: oauth2::basic::BasicRequestTokenError<oauth2::reqwest::Error<reqwest::Error>>,
    ) -> Self {
        AppError::AuthError(format!("OAuth2 error: {}", error))
    }
}

// Convert from URL parsing errors
impl From<url::ParseError> for AppError {
    fn from(error: url::ParseError) -> Self {
        AppError::ConfigError(format!("URL parsing error: {}", error))
    }
}

// Convert from Iced errors
impl From<iced::Error> for AppError {
    fn from(error: iced::Error) -> Self {
        AppError::UiError(error.to_string())
    }
}

// Convert from anyhow errors
impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        AppError::UnknownError(error.to_string())
    }
}
