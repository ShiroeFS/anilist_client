use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

// Main API client
#[derive(Clone)]
pub struct AniListClient {
    client: Client,
    endpoint: String,
    token: Option<String>,
}

// Define simplified request/response types
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnimeDetailsResponse {
    pub data: Option<AnimeData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnimeData {
    pub media: Option<Media>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Media {
    pub id: i32,
    pub title: MediaTitle,
    pub description: Option<String>,
    pub episodes: Option<i32>,
    pub genres: Option<Vec<String>>,
    pub average_score: Option<f64>,
    pub cover_image: Option<MediaCoverImage>,
    pub banner_image: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MediaTitle {
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MediaCoverImage {
    pub large: Option<String>,
    pub medium: Option<String>,
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
    ) -> Result<AnimeData, Box<dyn std::error::Error>> {
        // Simple GraphQL query with variables
        let query = r#"
        query ($id: Int) {
            Media(id: $id, type: ANIME) {
                id
                title {
                    romaji
                    english
                    native
                }
                description
                episodes
                genres
                averageScore
                coverImage {
                    large
                    medium
                }
                bannerImage
            }
        }
        "#;

        let request_body = json!({
            "query": query,
            "variables": {
                "id": id
            }
        });

        let mut request_builder = self.client.post(&self.endpoint).json(&request_body);

        // Add auth token if available
        if let Some(token) = &self.token {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", token));
        }

        let response = request_builder.send().await?;
        let response_body: AnimeDetailsResponse = response.json().await?;

        match response_body.data {
            Some(data) => Ok(data),
            None => Err("No data returned".into()),
        }
    }
}
