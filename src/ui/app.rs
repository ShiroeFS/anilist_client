use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Application, Command, Element, Length, Settings, Subscription, Theme};

use crate::api::client::AniListClient;
use crate::data::database::Database;
use crate::utils::config::AuthConfig;

// Application state
#[derive(Debug, Clone)]
pub enum Message {
    Search(String),
    SearchSubmitted,
    AnimeSelected(i32),
    AnimeLoaded(Result<AnimeDetails, String>),
    Login,
    Logout,
    // AuthenticateRequested,
    // AuthenticationStarted,
    // AuthenticationCompleted(Result<(), String>),
    // TokenRefreshed(Result<(), String>),
    LogoutRequested,
    LogoutCompleted,
    Error(String),
}

#[derive(Clone)]
pub struct AniListApp {
    // App state
    search_query: String,
    anime_list: Vec<AnimeListItem>,
    selected_anime: Option<AnimeDetails>,
    is_logged_in: bool,
    api_client: AniListClient,
    db: Database,
    auth_config: AuthConfig,
}

#[derive(Debug, Clone)]
pub struct AnimeListItem {
    pub id: i32,
    pub title: String,
    pub image_url: String,
}

#[derive(Debug, Clone)]
pub struct AnimeDetails {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub episodes: i32,
    pub genres: Vec<String>,
    pub score: f32,
    pub image_url: String,
}

impl AniListApp {
    pub fn new(client: AniListClient, db: Database, auth_config: AuthConfig) -> Self {
        // Check if we're logged in
        let is_logged_in = if let Ok(Some(_)) = db.get_auth() {
            true
        } else {
            false
        };

        Self {
            search_query: String::new(),
            anime_list: Vec::new(),
            selected_anime: None,
            is_logged_in,
            api_client: client,
            db,
            auth_config,
        }
    }

    pub fn _launch() -> iced::Result {
        Self::run(Settings::default())
    }
}

impl Application for AniListApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        // This should not be called directly, but we need to provide a default
        // implementation to satisfy the trait.
        let default_app = Self {
            search_query: String::new(),
            anime_list: Vec::new(),
            selected_anime: None,
            is_logged_in: false,
            api_client: AniListClient::new(),
            db: Database::new().unwrap_or_else(|_| panic!("Failed to initialize database")),
            auth_config: AuthConfig {
                client_id: "default".to_string(),
                client_secret: "default".to_string(),
                redirect_uri: "http://localhost:8080/callback".to_string(),
            },
        };

        (default_app, Command::none())
    }

    fn title(&self) -> String {
        String::from("AniList Desktop")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Search(query) => {
                self.search_query = query;
                Command::none()
            }
            Message::SearchSubmitted => {
                let query = self.search_query.clone();
                let client = self.api_client.clone();
                Command::perform(
                    async move {
                        let result = client.search_anime(query, Some(1), Some(10)).await;
                        match result {
                            Ok(data) => {
                                // Transform the data to our model
                                let items = if let Some(page) = data.page {
                                    if let Some(media_list) = page.media {
                                        media_list
                                            .into_iter()
                                            .filter_map(|media| {
                                                let media = media?;
                                                let title =
                                                    if let Some(title_obj) = media.title.clone() {
                                                        title_obj.romaji.or(title_obj.english)?
                                                    } else {
                                                        return None;
                                                    };
                                                let image_url = media.cover_image?.medium?;

                                                Some(AnimeListItem {
                                                    id: media.id as i32,
                                                    title,
                                                    image_url,
                                                })
                                            })
                                            .collect()
                                    } else {
                                        Vec::new()
                                    }
                                } else {
                                    Vec::new()
                                };
                                Ok(items)
                            }
                            Err(err) => Err(err.to_string()),
                        }
                    },
                    |result: Result<Vec<AnimeListItem>, String>| match result {
                        Ok(items) => {
                            if !items.is_empty() {
                                Message::AnimeSelected(items[0].id)
                            } else {
                                Message::Error("No results found".to_string())
                            }
                        }
                        Err(err) => Message::Error(err),
                    },
                )
            }
            Message::AnimeSelected(id) => {
                let client = self.api_client.clone();

                Command::perform(
                    async move {
                        let result = client.get_anime_details(id).await;
                        match result {
                            Ok(data) => {
                                if let Some(media) = data.media {
                                    let title = media.title.map_or_else(
                                        || "Unknown Title".to_string(),
                                        |t| {
                                            t.romaji
                                                .or(t.english)
                                                .unwrap_or_else(|| "Unknown Title".to_string())
                                        },
                                    );

                                    let image_url = media.cover_image.map_or_else(
                                        || "".to_string(),
                                        |img| img.medium.unwrap_or_default(),
                                    );

                                    Ok(AnimeDetails {
                                        id,
                                        title,
                                        description: media.description.unwrap_or_default(),
                                        episodes: media.episodes.unwrap_or(0) as i32,
                                        genres: media
                                            .genres
                                            .unwrap_or_default()
                                            .into_iter()
                                            .filter_map(|g| g) // Filter out None values
                                            .collect(),
                                        score: media.average_score.unwrap_or(0) as f32,
                                        image_url,
                                    })
                                } else {
                                    Err("Anime not found".to_string())
                                }
                            }
                            Err(err) => Err(err.to_string()),
                        }
                    },
                    Message::AnimeLoaded,
                )
            }
            Message::AnimeLoaded(result) => {
                match result {
                    Ok(details) => {
                        self.selected_anime = Some(details);
                    }
                    Err(err) => {
                        println!("Error loading anime: {}", err);
                    }
                }
                Command::none()
            }
            Message::Login => {
                // This would trigger the OAuth flow
                // In a real implementation, this would be handled by the auth manager
                Command::none()
            }
            Message::Logout => {
                self.is_logged_in = false;
                Command::none()
            }
            // Message::AuthenticateRequested => Command::perform(
            //     async { Ok(()) },
            //     |_: Result<(), Box<dyn std::error::Error>>| Message::AuthenticationStarted,
            // ),
            // Message::AuthenticationStarted => {
            //     // Show a loading indicator or similar UI feedback
            //     Command::perform(
            //         authenticate(self.auth_config.clone(), self.db.clone()),
            //         |result| match result {
            //             Ok(()) => Message::AuthenticationCompleted(Ok(())),
            //             Err(e) => Message::AuthenticationCompleted(Err(e.to_string())),
            //         },
            //     )
            // }
            // Message::AuthenticationCompleted(result) => {
            //     match result {
            //         Ok(()) => {
            //             self.is_logged_in = true;
            //             // Fetch user data and anime list after login
            //             Command::perform(
            //                 fetch_user_data(self.api_client.clone()),
            //                 |_| Message::Search("".to_string()), // Just a placeholder
            //             )
            //         }
            //         Err(e) => {
            //             eprintln!("Authentication error: {}", e);
            //             // Show error to user
            //             Command::none()
            //         }
            //     }
            // }
            // Message::TokenRefreshed(result) => {
            //     match result {
            //         Ok(()) => {
            //             // Token was successfully refreshed
            //             println!("Token refreshed successfully");
            //             Command::none()
            //         }
            //         Err(e) => {
            //             // Token refresh failed, might need to re-authenticate
            //             eprintln!("Token refresh error: {}", e);
            //             if self.is_logged_in {
            //                 self.is_logged_in = false;
            //                 // Trigger re-authentication
            //                 Command::perform(
            //                     async { Ok(()) },
            //                     |_: Result<(), Box<dyn std::error::Error>>| {
            //                         Message::AuthenticateRequested
            //                     },
            //                 )
            //             } else {
            //                 Command::none()
            //             }
            //         }
            //     }
            // }
            Message::LogoutRequested => {
                Command::perform(logout(self.db.clone()), |_| Message::LogoutCompleted)
            }
            Message::LogoutCompleted => {
                self.is_logged_in = false;
                // Clear user-specific data
                self.anime_list.clear();
                self.selected_anime = None;
                Command::none()
            }
            Message::Error(err) => {
                println!("Error: {}", err);
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        // Top bar with search and login
        let search_bar = row![
            text_input("Search anime or manga...", &self.search_query)
                .padding(10)
                .size(18)
                .on_input(Message::Search),
            button(text("Search"))
                .on_press(Message::SearchSubmitted)
                .padding(10),
        ]
        .padding(20)
        .spacing(10);

        let login_button = if self.is_logged_in {
            button(text("Logout")).on_press(Message::Logout).padding(10)
        } else {
            button(text("Login")).on_press(Message::Login).padding(10)
        };

        let top_bar = row![search_bar, login_button]
            .width(Length::Fill)
            .spacing(20);

        // Main content
        let content = if let Some(anime) = &self.selected_anime {
            // Anime details view
            column![
                text(&anime.title).size(30),
                row![
                    // This would be a real image in production
                    column![text("Image placeholder")]
                        .width(Length::Fixed(200.0))
                        .height(Length::Fixed(300.0)),
                    column![
                        text(&anime.description),
                        text(format!("Episodes: {}", anime.episodes)),
                        text(format!("Score: {:.2}", anime.score)),
                        text(format!("Genres: {}", anime.genres.join(", ")))
                    ]
                    .spacing(10)
                ]
                .spacing(20)
            ]
            .spacing(20)
            .padding(20)
        } else {
            // Anime list view or welcome screen
            let mut list = column![].spacing(10).padding(20);

            if self.anime_list.is_empty() {
                list = list.push(text("Search for anime or manga to get started!").size(20));
            } else {
                for item in &self.anime_list {
                    list = list.push(
                        button(text(&item.title))
                            .on_press(Message::AnimeSelected(item.id))
                            .width(Length::Fill)
                            .padding(10),
                    );
                }
            }

            list
        };

        let scrollable_container =
            container(scrollable(content).width(Length::Fill).height(Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(20);

        // Main layout
        container(column![top_bar, scrollable_container])
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }
}

// async fn authenticate(
//     auth_manager: AuthConfig,
//     mut db: Database,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let _token = auth_manager.ensure_authenticated(&mut db).await?;
//     // Update the client with the new token - this would need to be handled at app level
//     Ok(())
// }

// Function to perform logout
async fn logout(db: Database) -> Result<(), Box<dyn std::error::Error>> {
    // Clear auth data from database
    db.clear_auth()?;
    Ok(())
}

// Function to fetch user data after authentication
async fn _fetch_user_data(_client: AniListClient) -> Result<(), Box<dyn std::error::Error>> {
    // This would use the API client to fetch user data
    // For a simple implementation, we'll just return Ok
    Ok(())
}
