use crate::api::models::{Media, User};
use crate::utils::error::AppError;
use graphql_client::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};

// Define the GraphQL queries
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/api/queries/schema.graphql",
    query_path = "src/api/queries/anime_details.graphql",
    response_derives = "Debug, Clone, Serialize, Deserialize"
)]
pub struct AnimeDetails;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/api/queries/schema.graphql",
    query_path = "src/api/queries/anime_search.graphql",
    response_derives = "Debug, Clone, Serialize, Deserialize"
)]
pub struct AnimeSearch;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/api/queries/schema.graphql",
    query_path = "src/api/queries/user_anime_list.graphql",
    response_derives = "Debug, Clone, Serialize, Deserialize"
)]
pub struct UserAnimeList;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/api/queries/schema.graphql",
    query_path = "src/api/queries/user_profile.graphql",
    response_derives = "Debug, Clone, Serialize, Deserialize"
)]
pub struct UserProfile;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/api/queries/schema.graphql",
    query_path = "src/api/queries/viewer.graphql",
    response_derives = "Debug, Clone, Serialize, Deserialize"
)]
pub struct Viewer;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/api/queries/schema.graphql",
    query_path = "src/api/queries/update_media_list.graphql",
    response_derives = "Debug, Clone, Serialize, Deserialize"
)]
pub struct UpdateMediaList;

// Main API client
pub struct AniListClient {
    client: Client,
    endpoint: String,
    token: Option<String>,
}

impl AniListClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            endpoint: "https://graphql.anilist.co".to_string(),
            token: None,
        }
    }

    pub fn with_token(token: String) -> Self {
        let mut client = Self::new();
        client.token = Some(token);
        client
    }

    // Generic function to execute GraphQL queries
    async fn execute_query<Q: GraphQLQuery>(
        &self,
        variables: Q::Variables,
    ) -> Result<Q::ResponseData, AppError> {
        let request_body = Q::build_query(variables);
        let mut request_builder = self.client.post(&self.endpoint).json(&request_body);

        // Add auth token if available
        if let Some(token) = &self.token {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", token));
        }

        let response = request_builder.send().await?;
        let response_body: Response<Q::ResponseData> = response.json().await?;

        match response_body.data {
            Some(data) => Ok(data),
            None => {
                // Process GraphQL errors
                let errors = response_body.errors.unwrap_or_default();
                if !errors.is_empty() {
                    let error_msg = errors
                        .iter()
                        .map(|e| e.message.clone())
                        .collect::<Vec<String>>()
                        .join(", ");
                    Err(AppError::ApiError(error_msg))
                } else {
                    Err(AppError::ApiError("No data returned".into()))
                }
            }
        }
    }

    // Get details for a specific anime
    pub async fn get_anime_details(
        &self,
        id: i32,
    ) -> Result<anime_details::ResponseData, AppError> {
        let variables = anime_details::Variables { id: Some(id) };
        self.execute_query::<AnimeDetails>(variables).await
    }

    // Search for anime by title
    pub async fn search_anime(
        &self,
        query: String,
        page: Option<i32>,
        per_page: Option<i32>,
    ) -> Result<anime_search::ResponseData, AppError> {
        let variables = anime_search::Variables {
            search: Some(query),
            page,
            per_page,
        };
        self.execute_query::<AnimeSearch>(variables).await
    }

    // Get a user's anime list
    pub async fn get_user_anime_list(
        &self,
        user_id: i32,
        status: Option<String>,
    ) -> Result<user_anime_list::ResponseData, AppError> {
        // Convert string status to MediaListStatus enum
        let status_enum = match status {
            Some(s) => {
                match s.to_uppercase().as_str() {
                    "CURRENT" => Some(user_anime_list::MediaListStatus::CURRENT),
                    "PLANNING" => Some(user_anime_list::MediaListStatus::PLANNING),
                    "COMPLETED" => Some(user_anime_list::MediaListStatus::COMPLETED),
                    "DROPPED" => Some(user_anime_list::MediaListStatus::DROPPED),
                    "PAUSED" => Some(user_anime_list::MediaListStatus::PAUSED),
                    "REPEATING" => Some(user_anime_list::MediaListStatus::REPEATING),
                    _ => Some(user_anime_list::MediaListStatus::CURRENT), // Default
                }
            }
            None => None,
        };

        let variables = user_anime_list::Variables {
            user_id: Some(user_id),
            status: status_enum,
        };
        self.execute_query::<UserAnimeList>(variables).await
    }

    // Get a user's profile
    pub async fn get_user_profile(
        &self,
        username: String,
    ) -> Result<user_profile::ResponseData, AppError> {
        let variables = user_profile::Variables {
            name: Some(username),
        };
        self.execute_query::<UserProfile>(variables).await
    }

    // Get the authenticated user's information
    pub async fn get_viewer(&self) -> Result<viewer::ResponseData, AppError> {
        if self.token.is_none() {
            return Err(AppError::AuthError(
                "Authentication required to get viewer information".into(),
            ));
        }

        let variables = viewer::Variables {};
        self.execute_query::<Viewer>(variables).await
    }

    // Update a media list entry
    pub async fn update_media_list(
        &self,
        id: Option<i32>,
        media_id: Option<i32>,
        status: Option<String>,
        score: Option<f64>,
        progress: Option<i32>,
    ) -> Result<update_media_list::ResponseData, AppError> {
        if self.token.is_none() {
            return Err(AppError::AuthError(
                "Authentication required to update media list".into(),
            ));
        }

        // Convert string status to MediaListStatus enum
        let status_enum = match status {
            Some(s) => {
                match s.to_uppercase().as_str() {
                    "CURRENT" => Some(update_media_list::MediaListStatus::CURRENT),
                    "PLANNING" => Some(update_media_list::MediaListStatus::PLANNING),
                    "COMPLETED" => Some(update_media_list::MediaListStatus::COMPLETED),
                    "DROPPED" => Some(update_media_list::MediaListStatus::DROPPED),
                    "PAUSED" => Some(update_media_list::MediaListStatus::PAUSED),
                    "REPEATING" => Some(update_media_list::MediaListStatus::REPEATING),
                    _ => Some(update_media_list::MediaListStatus::CURRENT), // Default
                }
            }
            None => None,
        };

        // Convert parameters to the expected types
        let id_i64 = id.map(|i| i as i64);
        let media_id_i64 = media_id.map(|i| i as i64);
        let progress_i64 = progress.map(|i| i as i64);

        let variables = update_media_list::Variables {
            id: id_i64,
            media_id: media_id_i64,
            status: status_enum,
            score,
            progress: progress_i64,
        };
        self.execute_query::<UpdateMediaList>(variables).await
    }
}
