use iced::time::every;
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{settings, Application, Command, Element, Length, Settings, Subscription, Theme};
use std::sync::{Arc, Mutex};

use crate::api::auth::AuthManager;
use crate::api::client::{self, AniListClient};
use crate::data::database::Database;
use crate::ui::components::auth::{AuthComponent, Message as AuthMessage};
use crate::ui::screens::details::{DetailsScreen, Message as DetailsMessage};
use crate::ui::screens::home::{HomeScreen, Message as HomeMessage};
use crate::ui::screens::profile::{Message as ProfileMessage, ProfileScreen};
use crate::ui::screens::search::{Message as SearchMessage, SearchScreen};
use crate::ui::screens::settings::{Message as SettingsMessage, SettingsScreen};
use crate::utils::config::AuthConfig;

use super::screens::home;

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

    // Screen-specific messages
    Home(HomeMessage),
    Search(SearchMessage),
    Details(DetailsMessage),
    Profile(ProfileMessage),
    Settings(SettingsMessage),

    // Search-related
    SearchQueryChanged(String),
    SearchSubmitted,

    // Error handling
    Error(String),

    // Periodic check
    Tick,
}

pub struct AniListApp {
    // Core components
    api_client: AniListClient,
    db: Arc<Mutex<Database>>,
    auth_component: AuthComponent,

    // App state
    current_screen: Screen,
    screen_history: Vec<Screen>,
    search_query: String,

    // Screen modules
    home_screen: HomeScreen,
    search_screen: SearchScreen,
    details_screen: DetailsScreen,
    profile_screen: ProfileScreen,
    settings_screen: SettingsScreen,

    // UI state
    is_loading: bool,
    error: Option<String>,
}

impl AniListApp {
    pub fn new(client: AniListClient, db: Database, auth_config: AuthConfig) -> Self {
        // Wrap database in Arc<Mutex>
        let db_arc = Arc::new(Mutex::new(db.clone()));

        // Create auth manager
        let auth_manager = AuthManager::new(auth_config, db.clone());

        // Create auth component
        let auth_component = AuthComponent::new(auth_manager);

        // Create screen modules
        let home_screen = HomeScreen::new(client.clone());
        let search_screen = SearchScreen::new(client.clone());
        let details_screen = DetailsScreen::new(client.clone());
        let profile_screen = ProfileScreen::new(client.clone());
        let settings_screen = SettingsScreen::new(db_arc.clone());

        Self {
            api_client: client,
            db: db_arc,
            auth_component,
            current_screen: Screen::Home,
            screen_history: Vec::new(),
            search_query: String::new(),
            home_screen,
            search_screen,
            details_screen,
            profile_screen,
            settings_screen,
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

        let client = AniListClient::new();

        let home_screen = HomeScreen::new(client.clone());
        let search_screen = SearchScreen::new(client.clone());
        let details_screen = DetailsScreen::new(client.clone());
        let profile_screen = ProfileScreen::new(client.clone());
        let settings_screen = SettingsScreen::new(db_arc.clone());

        let default_app = Self {
            api_client: client,
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
            home_screen,
            search_screen,
            details_screen,
            profile_screen,
            settings_screen,
            is_loading: false,
            error: None,
        };

        (default_app, Command::none())
    }

    fn title(&self) -> String {
        match &self.current_screen {
            Screen::Home => String::from("AniList Desktop - Home"),
            Screen::Search => String::from("AniList Desktop - Search"),
            Screen::Details(_) => String::from("AniList Desktop - Anime Details"),
            Screen::Profile(username) => format!("AniList Desktop - {}'s Profile", username),
            Screen::Settings => String::from("AniList Desktop - Settings"),
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ChangeScreen(screen) => {
                self.navigate_to(screen.clone());

                match &screen {
                    Screen::Home => self.home_screen.init().map(Message::Home),
                    Screen::Search => Command::none(),
                    Screen::Details(id) => self.details_screen.load(*id).map(Message::Details),
                    Screen::Profile(username) => self
                        .profile_screen
                        .load(username.clone())
                        .map(Message::Profile),
                    Screen::Settings => Command::none(),
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
            Message::Home(home_msg) => {
                let cmd = self.home_screen.update(home_msg.clone());

                match home_msg {
                    HomeMessage::AnimeSelected(id) => {
                        // Navigate to details screen
                        self.navigate_to(Screen::Details(id));
                        self.details_screen.load(id).map(Message::Details)
                    }
                    _ => cmd.map(Message::Home),
                }
            }
            Message::Search(search_msg) => {
                let cmd = self.search_screen.update(search_msg.clone());

                match search_msg {
                    SearchMessage::AnimeSelected(id) => {
                        // Navigate to details screen
                        self.navigate_to(Screen::Details(id));
                        self.details_screen.load(id).map(Message::Details)
                    }
                    _ => cmd.map(Message::Search),
                }
            }
            Message::Details(details_msg) => self
                .details_screen
                .update(details_msg)
                .map(Message::Details),
            Message::Profile(profile_msg) => self
                .profile_screen
                .update(profile_msg)
                .map(Message::Profile),
            Message::Settings(settings_msg) => self
                .settings_screen
                .update(settings_msg)
                .map(Message::Settings),
            Message::AuthStatusChanged(is_authenticated) => {
                // Refresh the home screen if authentication status changed
                if is_authenticated && self.current_screen == Screen::Home {
                    self.home_screen.init().map(Message::Home)
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
                self.search_screen
                    .update(SearchMessage::QueryChanged(self.search_query.clone()))
                    .map(Message::Search)
            }
            Message::Error(e) => {
                self.error = Some(e);
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
        let top_area = row![nav_bar, search_bar, auth_view]
            .spacing(20)
            .padding(10)
            .width(Length::Fill);

        // Error message if any
        let error_view = if let Some(error) = &self.error {
            container(
                text(error).style(iced::theme::Text::Color(iced::Color::from_rgb(
                    0.8, 0.2, 0.2,
                ))),
            )
            .padding(10)
            .width(Length::Fill)
            .into()
        } else {
            container(text(""))
                .width(Length::Fill)
                .height(Length::Shrink)
                .into()
        };

        // Main content area based on current screen
        let content: Element<Message> = match &self.current_screen {
            Screen::Home => self.home_screen.view().map(Message::Home),
            Screen::Search => self.search_screen.view().map(Message::Search),
            Screen::Details(_) => self.details_screen.view().map(Message::Details),
            Screen::Profile(_) => self.profile_screen.view().map(Message::Profile),
            Screen::Settings => self.settings_screen.view().map(Message::Settings),
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
        container(column![top_area, error_view, back_button, content].spacing(5))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // Create a periodic timer to check auth status
        every(std::time::Duration::from_secs(300)).map(|_| Message::Tick)
    }
}
