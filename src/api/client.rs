use graphql_client::{GraphQLQuery, Response};
use reqwest::Client;

// Define the GraphQL query
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/api/queries/schema.graphql",
    query_path = "src/api/queries/anime_details.graphql",
    response_derives = "Debug, Clone",
    variable_derives = "Debug, Clone"
)]
pub struct AnimeDetailsQuery;

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

    pub async fn get_anime_details(
        &self,
        id: i32,
    ) -> Result<anime_details_query::ResponseData, Box<dyn std::error::Error>> {
        let variables = anime_details_query::Variables { id: Some(id) };
        let request_body = AnimeDetailsQuery::build_query(variables);

        let mut request_builder = self.client.post(&self.endpoint).json(&request_body);

        // Add auth token if available
        if let Some(token) = &self.token {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", token));
        }

        let response = request_builder.send().await?;
        let response_body: Response<anime_details_query::ResponseData> = response.json().await?;

        match response_body.data {
            Some(data) => Ok(data),
            None => Err("No data returned".into()),
        }
    }
}
