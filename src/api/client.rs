use crate::api::auth::{AuthManager, AuthToken};
use crate::utils::error::AppError;
use graphql_client::{GraphQLQuery, Response};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

// -----------------------------------------------------------------------------
// GraphQL query structs (one per .graphql file)
// -----------------------------------------------------------------------------

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/api/queries/schema.graphql",
    query_path = "src/api/queries/anime_details.graphql",
    response_derives = "Debug, Clone"
)]
pub struct AnimeDetails;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/api/queries/schema.graphql",
    query_path = "src/api/queries/anime_search.graphql",
    response_derives = "Debug, Clone"
)]
pub struct AnimeSearch;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/api/queries/schema.graphql",
    query_path = "src/api/queries/user_anime_list.graphql",
    response_derives = "Debug, Clone"
)]
pub struct UserAnimeList;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/api/queries/schema.graphql",
    query_path = "src/api/queries/user_profile.graphql",
    response_derives = "Debug, Clone"
)]
pub struct UserProfile;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/api/queries/schema.graphql",
    query_path = "src/api/queries/viewer.graphql",
    response_derives = "Debug, Clone"
)]
pub struct Viewer;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/api/queries/schema.graphql",
    query_path = "src/api/queries/update_media_list.graphql",
    response_derives = "Debug, Clone"
)]
pub struct UpdateMediaList;

#[derive(Debug, Clone)]
pub struct AniListClient {
    client: Client,
    endpoint: String,
    auth_manager: Option<Arc<AuthManager>>,
    auth_token: Arc<Mutex<Option<AuthToken>>>,
}

impl AniListClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            endpoint: "https://graphql.anilist.co".to_string(),
            auth_manager: None,
            auth_token: Arc::new(Mutex::new(None)),
        }
    }

    pub fn with_token(token: String) -> Self {
        let client = Self::new();

        if let Ok(mut token_guard) = client.auth_token.lock() {
            *token_guard = Some(AuthToken {
                access_token: token,
                token_type: "Bearer".to_string(),
                expires_in: None,
                refresh_token: None,
                created_at: chrono::Utc::now(),
            });
        }

        client
    }

    pub fn with_auth_manager(auth_manager: AuthManager) -> Self {
        let mut client = Self::new();
        client.auth_manager = Some(Arc::new(auth_manager));
        client
    }

    pub async fn get_current_token(&self) -> Result<Option<String>, AppError> {
        // First check if we have a token in memory
        let token_option = {
            if let Ok(token_guard) = self.auth_token.lock() {
                token_guard.clone()
            } else {
                return Err(AppError::ApiError("Failed to access auth token".into()));
            }
        };

        // If we have a token that's not expired, use it
        if let Some(token) = &token_option {
            if !token.is_expired() {
                return Ok(Some(token.access_token.clone()));
            }
        }

        // If we have an auth manager, try to refresh or reauthenticate
        if let Some(auth_manager) = &self.auth_manager {
            match auth_manager.ensure_authenticated().await {
                Ok(new_token) => {
                    // Update our in-memory token
                    if let Ok(mut token_guard) = self.auth_token.lock() {
                        *token_guard = Some(new_token.clone());
                    }

                    Ok(Some(new_token.access_token))
                }
                Err(e) => {
                    eprintln!("Authentication error: {}", e);
                    // Clear any existing token
                    if let Ok(mut token_guard) = self.auth_token.lock() {
                        *token_guard = None;
                    }

                    // For unauthenticated requests, we can still return Ok(None)
                    Ok(None)
                }
            }
        } else {
            // If we don't have an auth manager, just return what we have (if anything)
            if let Some(token) = token_option {
                Ok(Some(token.access_token))
            } else {
                Ok(None)
            }
        }
    }

    async fn execute_query<Q>(&self, variables: Q::Variables) -> Result<Q::ResponseData, AppError>
    where
        Q: GraphQLQuery,
    {
        let request_body = Q::build_query(variables);
        let mut request_builder = self.client.post(&self.endpoint).json(&request_body);

        // Add auth token if available
        if let Ok(Some(token)) = self.get_current_token().await {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", token));
        }

        let response = request_builder.send().await?;
        let status = response.status();

        // Handle rate limiting
        if status == StatusCode::TOO_MANY_REQUESTS {
            return Err(AppError::ApiError(
                "Rate limit exceeded. Please try again later.".into(),
            ));
        }

        // Handle other error statuses
        if !status.is_success() {
            return Err(AppError::ApiError(format!(
                "API request failed with status: {}",
                status
            )));
        }

        let response_body: Response<Q::ResponseData> = response.json().await?;

        // Check for GraphQL errors
        if let Some(errors) = response_body.errors {
            if !errors.is_empty() {
                let error_msg = errors
                    .iter()
                    .map(|e| e.message.clone())
                    .collect::<Vec<String>>()
                    .join(", ");
                return Err(AppError::ApiError(error_msg));
            }
        }

        match response_body.data {
            Some(data) => Ok(data),
            None => Err(AppError::ApiError("No data returned".into())),
        }
    }

    pub async fn get_anime_details(
        &self,
        id: i32,
    ) -> Result<anime_details::ResponseData, AppError> {
        let variables = anime_details::Variables {
            id: Some(id.into()),
        };
        self.execute_query::<AnimeDetails>(variables).await
    }

    pub async fn search_anime(
        &self,
        search: String,
        page: Option<i32>,
        per_page: Option<i32>,
    ) -> Result<anime_search::ResponseData, AppError> {
        let variables = anime_search::Variables {
            search: Some(search),
            page: page.map(|p| p.into()),
            per_page: per_page.map(|pp| pp.into()),
        };
        self.execute_query::<AnimeSearch>(variables).await
    }

    pub async fn get_user_anime_list(
        &self,
        user_id: i32,
        status: Option<user_anime_list::MediaListStatus>,
    ) -> Result<user_anime_list::ResponseData, AppError> {
        let variables = user_anime_list::Variables {
            user_id: Some(user_id.into()),
            status,
        };
        self.execute_query::<UserAnimeList>(variables).await
    }

    pub async fn get_user_profile(
        &self,
        name: String,
    ) -> Result<user_profile::ResponseData, AppError> {
        let variables = user_profile::Variables { name: Some(name) };
        self.execute_query::<UserProfile>(variables).await
    }

    pub async fn update_media_list(
        &self,
        id: Option<i32>,
        media_id: Option<i32>,
        status: Option<update_media_list::MediaListStatus>,
        score: Option<f64>,
        progress: Option<i32>,
    ) -> Result<update_media_list::ResponseData, AppError> {
        // This mutation requires authentication
        if let Ok(None) = self.get_current_token().await {
            return Err(AppError::ApiError(
                "Authentication required for this operation".into(),
            ));
        }

        let variables = update_media_list::Variables {
            id: id.map(|i| i.into()),
            media_id: media_id.map(|i| i.into()),
            status,
            score,
            progress: progress.map(|i| i.into()),
        };
        self.execute_query::<UpdateMediaList>(variables).await
    }

    pub async fn get_viewer(&self) -> Result<viewer::ResponseData, AppError> {
        // This query requires authentication
        if let Ok(None) = self.get_current_token().await {
            return Err(AppError::ApiError(
                "Authentication required for this operation".into(),
            ));
        }

        let variables = viewer::Variables {};
        self.execute_query::<Viewer>(variables).await
    }

    pub async fn is_authenticated(&self) -> bool {
        match self.get_current_token().await {
            Ok(Some(_)) => true,
            _ => false,
        }
    }

    pub async fn logout(&self) -> Result<(), AppError> {
        // Clear the in-memory token
        if let Ok(mut token_guard) = self.auth_token.lock() {
            *token_guard = None;
        }

        // If we have an auth manager, use it to clear the database
        if let Some(auth_manager) = &self.auth_manager {
            auth_manager.logout().await?;
        }

        Ok(())
    }
}
