use std::fmt;

#[derive(Debug)]
pub enum AppError {
    ApiError(String),
    DatabaseError(String),
    _AuthError(String),
    _ConfigError(String),
    NetworkError(String),
    _UiError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::ApiError(msg) => write!(f, "API Error: {}", msg),
            AppError::DatabaseError(msg) => write!(f, "Database Error: {}", msg),
            AppError::_AuthError(msg) => write!(f, "Authentication Error: {}", msg),
            AppError::_ConfigError(msg) => write!(f, "Configuration Error: {}", msg),
            AppError::NetworkError(msg) => write!(f, "Network Error: {}", msg),
            AppError::_UiError(msg) => write!(f, "UI Error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

// Convert from rusqlite errors
impl From<rusqlite::Error> for AppError {
    fn from(error: rusqlite::Error) -> Self {
        AppError::DatabaseError(error.to_string())
    }
}

// Convert from reqwest errors
impl From<reqwest::Error> for AppError {
    fn from(error: reqwest::Error) -> Self {
        AppError::NetworkError(error.to_string())
    }
}

// Convert from serde_json errors
impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        AppError::ApiError(format!("JSON parsing error: {}", error))
    }
}
