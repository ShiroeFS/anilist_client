use iced::time::every;
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Application, Command, Element, Length, Settings, Subscription, Theme};
use std::sync::{Arc, Mutex};

use crate::api::auth::AuthManager;
use crate::api::client::AniListClient;
use crate::data::database::Database;
use crate::ui::components::auth::{AuthComponent, Message as AuthMessage};
use crate::utils::config::AuthConfig;

// Application screens
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    Home,
    Search,
    Details(i32),    // Anime ID
    Profile(String), // Username
    Settings,
}

// Application state
#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    ChangeScreen(Screen),
    GoBack,

    // Auth-related
    Auth(AuthMessage),
    AuthStatusChanged(bool),

    // Search-related
    SearchQueryChanged(String),
    SearchSubmitted,

    // Data loading
    AnimeLoaded(Result<AnimeDetails, String>),
    UserLoaded(Result<UserProfile, String>),

    // Error handling
    Error(String),

    // Periodic check
    Tick,
}

#[derive(Clone)]
pub struct AniListApp {
    // Core components
    api_client: AniListClient,
    db: Arc<Mutex<Database>>,
    auth_component: AuthComponent,

    // App state
    current_screen: Screen,
    screen_history: Vec<Screen>,
    search_query: String,

    // Data
    selected_anime: Option<AnimeDetails>,
    user_profile: Option<UserProfile>,

    // UI state
    is_loading: bool,
    error: Option<String>,
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

#[derive(Debug, Clone)]
pub struct UserProfile {
    pub id: i32,
    pub name: String,
    pub avatar_url: String,
    pub anime_count: i32,
    pub manga_count: i32,
}

impl AniListApp {
    pub fn new(client: AniListClient, db: Arc<Mutex<Database>>, auth_config: AuthConfig) -> Self {
        // Create auth manager
        let auth_manager = if let Ok(db_guard) = db.lock() {
            // Convert from utils::config::AuthConfig to api::auth::AuthConfig
            AuthManager::new(auth_config, db_guard.clone())
        } else {
            // Fallback in case we can't get the lock
            let db_fallback = Database::new().expect("Failed to create database instance");
            AuthManager::new(auth_config, db_fallback)
        };

        // Create auth component
        let auth_component = AuthComponent::new(auth_manager);

        Self {
            api_client: client,
            db,
            auth_component,
            current_screen: Screen::Home,
            screen_history: Vec::new(),
            search_query: String::new(),
            selected_anime: None,
            user_profile: None,
            is_loading: false,
            error: None,
        }
    }

    pub fn launch() -> iced::Result {
        Self::run(Settings::default())
    }

    fn navigate_to(&mut self, screen: Screen) {
        // Don't add duplicate screens to history
        if self.current_screen != screen {
            self.screen_history.push(self.current_screen.clone());
            self.current_screen = screen;
        }
    }

    fn go_back(&mut self) {
        if let Some(previous_screen) = self.screen_history.pop() {
            self.current_screen = previous_screen;
        }
    }

    fn check_auth_status(&self) -> Command<Message> {
        let client = self.api_client.clone();
        Command::perform(
            async move { client.is_authenticated().await },
            Message::AuthStatusChanged,
        )
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
        let default_db =
            Database::new().unwrap_or_else(|_| panic!("Failed to initialize database"));
        let db_arc = Arc::new(Mutex::new(default_db.clone()));

        let default_app = Self {
            api_client: AniListClient::new(),
            db: db_arc,
            auth_component: AuthComponent::new(AuthManager::new(
                AuthConfig {
                    client_id: "default".to_string(),
                    client_secret: "default".to_string(),
                    redirect_uri: "http://localhost:8080/callback".to_string(),
                },
                default_db,
            )),
            current_screen: Screen::Home,
            screen_history: Vec::new(),
            search_query: String::new(),
            selected_anime: None,
            user_profile: None,
            is_loading: false,
            error: None,
        };

        (default_app, Command::none())
    }

    fn title(&self) -> String {
        match &self.current_screen {
            Screen::Home => String::from("AniList Desktop - Home"),
            Screen::Search => String::from("AniList Desktop - Search"),
            Screen::Details(_) => {
                if let Some(anime) = &self.selected_anime {
                    format!("AniList Desktop - {}", anime.title)
                } else {
                    String::from("AniList Desktop - Anime Details")
                }
            }
            Screen::Profile(username) => format!("AniList Desktop - {}'s Profile", username),
            Screen::Settings => String::from("AniList Desktop - Settings"),
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ChangeScreen(screen) => {
                self.navigate_to(screen);

                match &self.current_screen {
                    Screen::Details(id) => {
                        // Load anime details
                        self.is_loading = true;
                        let anime_id = *id;
                        let client = self.api_client.clone();

                        Command::perform(
                            async move {
                                match client.get_anime_details(anime_id).await {
                                    Ok(data) => {
                                        if let Some(media) = data.media {
                                            let title = media.title.map_or_else(
                                                || "Unknown Title".to_string(),
                                                |t| {
                                                    t.romaji.or(t.english).unwrap_or_else(|| {
                                                        "Unknown Title".to_string()
                                                    })
                                                },
                                            );

                                            let image_url = media.cover_image.map_or_else(
                                                || "".to_string(),
                                                |img| img.large.unwrap_or_default(),
                                            );

                                            Ok(AnimeDetails {
                                                id: anime_id,
                                                title,
                                                description: media.description.unwrap_or_default(),
                                                episodes: media.episodes.unwrap_or(0) as i32,
                                                genres: media
                                                    .genres
                                                    .unwrap_or_default()
                                                    .into_iter()
                                                    .filter_map(|g| g) // Filter out None values
                                                    .collect(),
                                                score: media.average_score.unwrap_or(0) as f32
                                                    / 10.0,
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
                    Screen::Profile(username) => {
                        // Load user profile
                        self.is_loading = true;
                        let username = username.clone();
                        let client = self.api_client.clone();

                        Command::perform(
                            async move {
                                match client.get_user_profile(username).await {
                                    Ok(data) => {
                                        if let Some(user) = data.user {
                                            let avatar_url = user
                                                .avatar
                                                .as_ref()
                                                .and_then(|a| a.large.clone().or(a.medium.clone()))
                                                .unwrap_or_default();

                                            // Clone the statistics before using it multiple times
                                            let statistics = user.statistics.clone();

                                            let anime_count = statistics
                                                .as_ref()
                                                .and_then(|s| s.anime.as_ref())
                                                .map(|a| a.count)
                                                .unwrap_or(0)
                                                as i32;

                                            let manga_count = statistics
                                                .as_ref()
                                                .and_then(|s| s.manga.as_ref())
                                                .map(|m| m.count)
                                                .unwrap_or(0)
                                                as i32;

                                            Ok(UserProfile {
                                                id: user.id as i32,
                                                name: user.name,
                                                avatar_url,
                                                anime_count,
                                                manga_count,
                                            })
                                        } else {
                                            Err("User not found".to_string())
                                        }
                                    }
                                    Err(err) => Err(err.to_string()),
                                }
                            },
                            Message::UserLoaded,
                        )
                    }
                    _ => Command::none(),
                }
            }
            Message::GoBack => {
                self.go_back();
                Command::none()
            }
            Message::Auth(auth_msg) => {
                let auth_cmd = self.auth_component.update(auth_msg);

                // After authentication state changes, check auth status and refresh UI
                Command::batch(vec![auth_cmd.map(Message::Auth), self.check_auth_status()])
            }
            Message::AuthStatusChanged(is_authenticated) => {
                // Refresh the home screen if authentication status changed
                if is_authenticated && self.current_screen == Screen::Home {
                    // Reload the home screen data
                    Command::none() // Would trigger home screen data loading
                } else {
                    Command::none()
                }
            }
            Message::SearchQueryChanged(query) => {
                self.search_query = query;
                Command::none()
            }
            Message::SearchSubmitted => {
                // Navigate to search screen with the current query
                self.navigate_to(Screen::Search);

                // In a real implementation, this would actually perform the search
                // but we're just navigating for now
                Command::none()
            }
            Message::AnimeLoaded(result) => {
                self.is_loading = false;

                match result {
                    Ok(details) => {
                        self.selected_anime = Some(details);
                        self.error = None;
                    }
                    Err(err) => {
                        self.error = Some(format!("Failed to load anime details: {}", err));
                    }
                }

                Command::none()
            }
            Message::UserLoaded(result) => {
                self.is_loading = false;

                match result {
                    Ok(profile) => {
                        self.user_profile = Some(profile);
                        self.error = None;
                    }
                    Err(err) => {
                        self.error = Some(format!("Failed to load user profile: {}", err));
                    }
                }

                Command::none()
            }
            Message::Error(error) => {
                self.error = Some(error);
                Command::none()
            }
            Message::Tick => {
                // Check auth status periodically
                self.check_auth_status()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        // Top navigation bar
        let nav_bar = row![
            button(text("Home"))
                .on_press(Message::ChangeScreen(Screen::Home))
                .padding(10)
                .style(if self.current_screen == Screen::Home {
                    iced::theme::Button::Primary
                } else {
                    iced::theme::Button::Secondary
                }),
            button(text("Search"))
                .on_press(Message::ChangeScreen(Screen::Search))
                .padding(10)
                .style(if matches!(self.current_screen, Screen::Search) {
                    iced::theme::Button::Primary
                } else {
                    iced::theme::Button::Secondary
                }),
            button(text("Settings"))
                .on_press(Message::ChangeScreen(Screen::Settings))
                .padding(10)
                .style(if self.current_screen == Screen::Settings {
                    iced::theme::Button::Primary
                } else {
                    iced::theme::Button::Secondary
                }),
        ]
        .spacing(10)
        .padding(10);

        // Search bar
        let search_bar = row![
            text_input("Search anime...", &self.search_query)
                .padding(10)
                .on_input(Message::SearchQueryChanged),
            button(text("Search"))
                .on_press(Message::SearchSubmitted)
                .padding(10),
        ]
        .spacing(10)
        .padding(10);

        // Auth component
        let auth_view = self.auth_component.view().map(Message::Auth);

        // Top area with navigation and search
        let top_area = row![nav_bar, search_bar, auth_view,]
            .spacing(20)
            .padding(10)
            .width(Length::Fill);

        // Error message if any
        let error_view = if let Some(error) = &self.error {
            let error_container: Element<Message> = container(text(error).style(
                iced::theme::Text::Color(iced::Color::from_rgb(0.8, 0.2, 0.2)),
            ))
            .padding(10)
            .width(Length::Fill)
            .into();

            error_container
        } else {
            container(text(""))
                .width(Length::Fill)
                .height(Length::Shrink)
                .into()
        };

        // Main content area based on current screen
        let content = match &self.current_screen {
            Screen::Home => {
                // For now, just a placeholder
                let home_content = column![
                    text("Welcome to AniList Desktop").size(30),
                    text("Browse and track your anime and manga in one place.").size(18),
                ]
                .spacing(20)
                .padding(40)
                .width(Length::Fill);

                scrollable(home_content).height(Length::Fill).into()
            }
            Screen::Search => {
                // For now, just a placeholder
                let search_content = if self.is_loading {
                    column![text("Searching...").size(24)]
                } else {
                    column![text("Search Results").size(24)]
                };

                scrollable(search_content.spacing(20).padding(40).width(Length::Fill))
                    .height(Length::Fill)
                    .into()
            }
            Screen::Details(id) => {
                // Show anime details or loading indicator
                let details_content = if self.is_loading {
                    column![text("Loading anime details...").size(24)]
                } else if let Some(anime) = &self.selected_anime {
                    column![
                        text(&anime.title).size(30),
                        row![
                            // This would be an actual image in a real implementation
                            column![text("Image placeholder")].width(Length::Fixed(200.0)),
                            column![
                                text(&anime.description),
                                text(format!("Episodes: {}", anime.episodes)),
                                text(format!("Score: {:.1}", anime.score)),
                                text(format!("Genres: {}", anime.genres.join(", ")))
                            ]
                            .spacing(10)
                        ]
                        .spacing(20)
                    ]
                } else {
                    column![text("Anime not found").size(24)]
                };

                scrollable(details_content.spacing(20).padding(40).width(Length::Fill))
                    .height(Length::Fill)
                    .into()
            }
            Screen::Profile(username) => {
                // Show profile or loading indicator
                let profile_content = if self.is_loading {
                    column![text(format!("Loading {}'s profile...", username)).size(24)]
                } else if let Some(profile) = &self.user_profile {
                    column![
                        text(&profile.name).size(30),
                        row![
                            // This would be an actual avatar image in a real implementation
                            column![text("Avatar placeholder")].width(Length::Fixed(100.0)),
                            column![
                                text(format!("Anime: {}", profile.anime_count)),
                                text(format!("Manga: {}", profile.manga_count)),
                            ]
                            .spacing(10)
                        ]
                        .spacing(20)
                    ]
                } else {
                    column![text(format!("User {} not found", username)).size(24)]
                };

                scrollable(profile_content.spacing(20).padding(40).width(Length::Fill))
                    .height(Length::Fill)
                    .into()
            }
            Screen::Settings => {
                // Settings screen
                let settings_content = column![
                    text("Settings").size(30),
                    // Theme selection
                    text("Theme").size(18),
                    row![
                        button(text("Light")).padding(10),
                        button(text("Dark")).padding(10),
                        button(text("System")).padding(10),
                    ]
                    .spacing(10),
                    // API settings
                    text("API Configuration").size(18),
                    text("Client ID").size(14),
                    text_input("Client ID", "").padding(10),
                    text("Client Secret").size(14),
                    text_input("Client Secret", "").padding(10),
                    // Cache settings
                    text("Cache").size(18),
                    row![
                        button(text("Clear Cache")).padding(10),
                        button(text("Clear Auth Data")).padding(10),
                    ]
                    .spacing(10),
                ]
                .spacing(15)
                .padding(40);

                scrollable(settings_content).height(Length::Fill).into()
            }
        };

        // Back button if we're not on the home screen
        let back_button = if self.current_screen != Screen::Home && !self.screen_history.is_empty()
        {
            container(button(text("← Back")).on_press(Message::GoBack).padding(10))
                .padding(10)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Left)
                .into()
        } else {
            container(text("")).width(Length::Fill).into()
        };

        // Main layout
        container(column![top_area, error_view, back_button, content,].spacing(5))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // Create a periodic timer to check auth status
        every(std::time::Duration::from_secs(300)).map(|_| Message::Tick)
    }
}
