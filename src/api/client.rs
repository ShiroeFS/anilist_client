use crate::utils::error::AppError;
use graphql_client::{GraphQLQuery, Response};
use reqwest::Client;
use serde::{Deserialize, Serialize};

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

#[derive(Clone)]
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

    async fn execute_query<Q>(&self, variables: Q::Variables) -> Result<Q::ResponseData, AppError>
    where
        Q: GraphQLQuery,
    {
        let request_body = Q::build_query(variables);
        let mut request_builder = self.client.post(&self.endpoint).json(&request_body);

        // Add auth token if available
        if let Some(token) = &self.token {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", token));
        }

        let response = request_builder.send().await?;

        if !response.status().is_success() {
            return Err(AppError::ApiError(format!(
                "API request failed with status: {}",
                response.status()
            )));
        }

        let response_body: Response<Q::ResponseData> = response.json().await?;

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
        let variables = viewer::Variables {};
        self.execute_query::<Viewer>(variables).await
    }
}
