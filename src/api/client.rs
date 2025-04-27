use crate::api::models::{Media, User};
use crate::utils::error::AppError;
use graphql_client::{GraphQLQuery, Response};
use reqwest::Client;
use serde::{Deserialize, Serialize};

// Define the GraphQL queries
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/api/queries/schema.graphql",
    query_path = "src/api/queries/anime_details.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct AnimeDetails;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/api/queries/schema.graphql",
    query_path = "src/api/queries/anime_search.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct AnimeSearch;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/api/queries/schema.graphql",
    query_path = "src/api/queries/user_anime_list.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct UserAnimeList;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/api/queries/schema.graphql",
    query_path = "src/api/queries/user_profile.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct UserProfile;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/api/queries/schema.graphql",
    query_path = "src/api/queries/viewer.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct Viewer;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/api/queries/schema.graphql",
    query_path = "src/api/queries/update_media_list.graphql",
    response_derives = "Debug, Serialize, Deserialize"
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

    pub async fn get_anime_details(
        &self,
        id: i32,
    ) -> Result<anime_details::ResponseData, AppError> {
        let variables = anime_details::Variables { id: Some(id) };
        self.execute_query::<AnimeDetails>(variables).await
    }

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

    pub async fn get_user_anime_list(
        &self,
        user_id: i32,
        status: Option<String>,
    ) -> Result<user_anime_list::ResponseData, AppError> {
        let status_enum = status.map(|s| {
            // Map string to MediaListStatus enum
            match s.to_uppercase().as_str() {
                "CURRENT" => user_anime_list::MediaListStatus::CURRENT,
                "PLANNING" => user_anime_list::MediaListStatus::PLANNING,
                "COMPLETED" => user_anime_list::MediaListStatus::COMPLETED,
                "DROPPED" => user_anime_list::MediaListStatus::DROPPED,
                "PAUSED" => user_anime_list::MediaListStatus::PAUSED,
                "REPEATING" => user_anime_list::MediaListStatus::REPEATING,
                _ => user_anime_list::MediaListStatus::CURRENT, // Default
            }
        });

        let variables = user_anime_list::Variables {
            user_id: Some(user_id),
            status: status_enum,
        };
        self.execute_query::<UserAnimeList>(variables).await
    }

    pub async fn get_user_profile(
        &self,
        username: String,
    ) -> Result<user_profile::ResponseData, AppError> {
        let variables = user_profile::Variables {
            name: Some(username),
        };
        self.execute_query::<UserProfile>(variables).await
    }

    pub async fn get_viewer(&self) -> Result<viewer::ResponseData, AppError> {
        if self.token.is_none() {
            return Err(AppError::AuthError(
                "Authentication required to get viewer information".into(),
            ));
        }

        let variables = viewer::Variables {};
        self.execute_query::<Viewer>(variables).await
    }

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

        let status_enum = status.map(|s| {
            // Map string to MediaListStatus enum
            match s.to_uppercase().as_str() {
                "CURRENT" => update_media_list::MediaListStatus::CURRENT,
                "PLANNING" => update_media_list::MediaListStatus::PLANNING,
                "COMPLETED" => update_media_list::MediaListStatus::COMPLETED,
                "DROPPED" => update_media_list::MediaListStatus::DROPPED,
                "PAUSED" => update_media_list::MediaListStatus::PAUSED,
                "REPEATING" => update_media_list::MediaListStatus::REPEATING,
                _ => update_media_list::MediaListStatus::CURRENT, // Default
            }
        });

        let variables = update_media_list::Variables {
            id,
            media_id,
            status: status_enum,
            score,
            progress,
        };
        self.execute_query::<UpdateMediaList>(variables).await
    }
}
