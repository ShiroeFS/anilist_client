use crate::utils::error::AppError;
use graphql_client::*;
use reqwest::Client;

// -----------------------------------------------------------------------------
// GraphQL query structs (one per .graphql file)
// -----------------------------------------------------------------------------

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/api/queries/schema.graphql",
    query_path = "src/api/queries/anime_details.graphql",
    response_derives = "Debug, Clone, serde::Serialize, serde::Deserialize"
)]
pub struct AnimeDetails;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/api/queries/schema.graphql",
    query_path = "src/api/queries/anime_search.graphql",
    response_derives = "Debug, Clone, serde::Serialize, serde::Deserialize"
)]
pub struct AnimeSearch;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/api/queries/schema.graphql",
    query_path = "src/api/queries/user_anime_list.graphql",
    response_derives = "Debug, Clone, serde::Serialize, serde::Deserialize"
)]
pub struct UserAnimeList;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/api/queries/schema.graphql",
    query_path = "src/api/queries/user_profile.graphql",
    response_derives = "Debug, Clone, serde::Serialize, serde::Deserialize"
)]
pub struct UserProfile;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/api/queries/schema.graphql",
    query_path = "src/api/queries/viewer.graphql",
    response_derives = "Debug, Clone, serde::Serialize, serde::Deserialize"
)]
pub struct Viewer;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/api/queries/schema.graphql",
    query_path = "src/api/queries/update_media_list.graphql",
    response_derives = "Debug, Clone, serde::Serialize, serde::Deserialize"
)]
pub struct UpdateMediaList;

// -----------------------------------------------------------------------------
// AniListClient – high‑level wrapper around the AniList GraphQL endpoint
// -----------------------------------------------------------------------------

pub struct AniListClient {
    client: Client,
    endpoint: String,
    token: Option<String>,
}

impl Default for AniListClient {
    fn default() -> Self {
        Self::new()
    }
}

impl AniListClient {
    /// Un‑authenticated client.
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            endpoint: "https://graphql.anilist.co".to_string(),
            token: None,
        }
    }

    /// Client that sends `Bearer <token>` on every request.
    pub fn with_token(token: impl Into<String>) -> Self {
        Self {
            token: Some(token.into()),
            ..Self::new()
        }
    }

    // ---------------------------------------------------------------------
    // internal generic executor – the only place we hit the network
    // ---------------------------------------------------------------------
    async fn execute_query<Q>(&self, variables: Q::Variables) -> Result<Q::ResponseData, AppError>
    where
        Q: GraphQLQuery,
    {
        let body = Q::build_query(variables);
        let mut req = self.client.post(&self.endpoint).json(&body);

        if let Some(t) = &self.token {
            req = req.header("Authorization", format!("Bearer {t}"));
        }

        let resp = req.send().await?;
        let resp_body: graphql_client::Response<Q::ResponseData> = resp.json().await?;

        match resp_body.data {
            Some(d) => Ok(d),
            None => {
                let msg = resp_body
                    .errors
                    .and_then(|mut e| e.pop())
                    .map(|e| e.message)
                    .unwrap_or_else(|| "AniList returned no data".to_string());
                Err(AppError::ApiError(msg))
            }
        }
    }

    // ---------------------------------------------------------------------
    // public high‑level helpers – thin, type‑safe wrappers
    // ---------------------------------------------------------------------

    pub async fn get_anime_details(
        &self,
        id: i32,
    ) -> Result<anime_details::ResponseData, AppError> {
        let variables = anime_details::Variables { id: Some(id) };
        self.execute_query::<AnimeDetails>(variables).await
    }

    pub async fn search_anime(
        &self,
        query: impl Into<String>,
        page: Option<i32>,
        per_page: Option<i32>,
    ) -> Result<anime_search::ResponseData, AppError> {
        let variables = anime_search::Variables {
            search: Some(query.into()),
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
        let status = status.map(|s| {
            use user_anime_list::MediaListStatus::*;
            match s.to_uppercase().as_str() {
                "CURRENT" => CURRENT,
                "PLANNING" => PLANNING,
                "COMPLETED" => COMPLETED,
                "DROPPED" => DROPPED,
                "PAUSED" => PAUSED,
                "REPEATING" => REPEATING,
                _ => CURRENT,
            }
        });

        let variables = user_anime_list::Variables {
            user_id: Some(user_id),
            status,
        };
        self.execute_query::<UserAnimeList>(variables).await
    }

    pub async fn get_user_profile(
        &self,
        username: impl Into<String>,
    ) -> Result<user_profile::ResponseData, AppError> {
        let variables = user_profile::Variables {
            name: Some(username.into()),
        };
        self.execute_query::<UserProfile>(variables).await
    }

    pub async fn get_viewer(&self) -> Result<viewer::ResponseData, AppError> {
        if self.token.is_none() {
            return Err(AppError::AuthError(
                "Authentication required to get viewer information".into(),
            ));
        }
        self.execute_query::<Viewer>(viewer::Variables {}).await
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

        let status = status.map(|s| {
            use update_media_list::MediaListStatus::*;
            match s.to_uppercase().as_str() {
                "CURRENT" => CURRENT,
                "PLANNING" => PLANNING,
                "COMPLETED" => COMPLETED,
                "DROPPED" => DROPPED,
                "PAUSED" => PAUSED,
                "REPEATING" => REPEATING,
                _ => CURRENT,
            }
        });

        let variables = update_media_list::Variables {
            id: id.map(|x| x as i64),
            media_id: media_id.map(|x| x as i64),
            status,
            score,
            progress: progress.map(|x| x as i64),
        };

        self.execute_query::<UpdateMediaList>(variables).await
    }
}
