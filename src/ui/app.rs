use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Application, Command, Element, Length, Subscription, Theme};

use crate::api::auth::AuthManager;
use crate::api::client::AniListClient;
use crate::data::database::Database;

// Application state
#[derive(Debug, Clone)]
pub enum Message {
    Search(String),
    SearchSubmitted,
    AnimeSelected(i32),
    AnimeLoaded(Result<AnimeDetails, String>),
    Login,
    Logout,
}

pub struct AniListApp {
    // App state
    search_query: String,
    anime_list: Vec<AnimeListItem>,
    selected_anime: Option<AnimeDetails>,
    is_logged_in: bool,
    api_client: AniListClient,
    db: Database,
    auth_manager: AuthManager,
}

#[derive(Debug, Clone)]
struct AnimeListItem {
    id: i32,
    title: String,
    image_url: String,
}

#[derive(Debug, Clone)]
struct AnimeDetails {
    id: i32,
    title: String,
    description: String,
    episodes: i32,
    genres: Vec<String>,
    score: f32,
    image_url: String,
}

impl AniListApp {
    pub fn new(client: AniListClient, db: Database, auth_manager: AuthManager) -> Self {
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
            auth_manager,
        }
    }
}

impl Application for AniListApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = (AniListClient, Database, AuthManager);

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        // Use the constructor with our flag values
        let (client, db, auth_manager) = flags;
        (Self::new(client, db, auth_manager), Command::none())
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
                // Simulate search results for now
                let query = self.search_query.clone();
                Command::perform(
                    async move {
                        // This would be an API call in a real app
                        // For now, just return some dummy data
                        if query.is_empty() {
                            return Ok(vec![]);
                        }

                        Ok(vec![
                            AnimeListItem {
                                id: 1,
                                title: "Cowboy Bebop".to_string(),
                                image_url: "http://example.com/image1.jpg".to_string(),
                            },
                            AnimeListItem {
                                id: 2,
                                title: "Steins;Gate".to_string(),
                                image_url: "https://example.com/image2.jpg".to_string(),
                            },
                        ])
                    },
                    |result: Result<Vec<AnimeListItem>, String>| {
                        match result {
                            Ok(_items) => {
                                // This would set self.anime_list in the real implementation
                                Message::AnimeSelected(1) // Just for demo
                            }
                            Err(_) => {
                                // Handle error
                                Message::Search("".to_string())
                            }
                        }
                    },
                )
            }
            Message::AnimeSelected(id) => {
                // Fetch anime details
                Command::perform(
                    async move {
                        // this would be an API call in real app
                        Ok(AnimeDetails {
                            id,
                            title: "Cowboy Bebop".to_string(),
                            description: "In the year 2071, humanity has colonized several of the planets and moons...".to_string(),
                            episodes: 26,
                            genres: vec!["Action".to_string(), "Adventure".to_string(), "Drama".to_string()],
                            score: 8.78,
                            image_url: "https://example.com/image1.jpg".to_string(),
                        })
                    },
                    Message::AnimeLoaded,
                )
            }
            Message::AnimeLoaded(result) => {
                match result {
                    Ok(details) => {
                        self.selected_anime = Some(details);
                    }
                    Err(_) => {
                        // Handle error
                    }
                }
                Command::none()
            }
            Message::Login => {
                // This would trigger the OAuth flow
                Command::none()
            }
            Message::Logout => {
                self.is_logged_in = false;
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
