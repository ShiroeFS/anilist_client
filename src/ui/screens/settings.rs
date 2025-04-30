use iced::widget::{
    button, column, container, pick_list, row, scrollable, text, text_input, toggler,
};
use iced::{Command, Element, Length};
use std::sync::{Arc, Mutex};

use crate::data::database::Database;
use crate::utils::config::{load_config, save_config, Config};

#[derive(Debug, Clone)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Theme::Light => write!(f, "Light"),
            Theme::Dark => write!(f, "Dark"),
            Theme::System => write!(f, "System"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    ThemeSelected(Theme),
    ClientIdChanged(String),
    ClientSecretChanged(String),
    RedirectUriChanged(String),
    OfflineModeToggled(bool),
    LanguageChanged(String),
    SaveConfig,
    ConfigSaved(Result<(), String>),
    ClearCache,
    ClearCacheCompleted(Result<(), String>),
    ClearAuth,
    ClearAuthCompleted(Result<(), String>),
    Error(String),
}

pub struct SettingsScreen {
    db: Arc<Mutex<Database>>,
    config: Config,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    offline_mode: bool,
    language: String,
    theme: Theme,
    is_saving: bool,
    is_clearing_cache: bool,
    is_clearing_auth: bool,
    error: Option<String>,
    success_message: Option<String>,
}

impl SettingsScreen {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        // Load current config
        let config = load_config().unwrap_or_else(|_| {
            println!("Failed to load config, using defaults");
            Config::default()
        });

        // Determine theme
        let theme = match config.theme.as_str() {
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            _ => Theme::System,
        };

        Self {
            db,
            config: config.clone(),
            client_id: config.auth_config.client_id.clone(),
            client_secret: config.auth_config.client_secret.clone(),
            redirect_uri: config.auth_config.redirect_uri.clone(),
            offline_mode: config.offline_mode,
            language: config.language.clone(),
            theme,
            is_saving: false,
            is_clearing_cache: false,
            is_clearing_auth: false,
            error: None,
            success_message: None,
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ThemeSelected(theme) => {
                self.theme = theme;
                Command::none()
            }
            Message::ClientIdChanged(client_id) => {
                self.client_id = client_id;
                Command::none()
            }
            Message::ClientSecretChanged(client_secret) => {
                self.client_secret = client_secret;
                Command::none()
            }
            Message::RedirectUriChanged(redirect_uri) => {
                self.redirect_uri = redirect_uri;
                Command::none()
            }
            Message::OfflineModeToggled(enabled) => {
                self.offline_mode = enabled;
                Command::none()
            }
            Message::LanguageChanged(language) => {
                self.language = language;
                Command::none()
            }
            Message::SaveConfig => {
                self.is_saving = false;

                match result {
                    Ok(()) => {
                        // Update the stored config
                        self.config.auth_config.client_id = self.client_id.clone();
                        self.config.auth_config.client_secret = self.client_secret.clone();
                        self.config.auth_config.redirect_uri = self.redirect_uri.clone();
                        self.config.offline_mode = self.offline_mode;
                        self.config.language = self.language.clone();
                        self.config.theme = match self.theme {
                            Theme::Light => "light".to_string(),
                            Theme::Dark => "dark".to_string(),
                            Theme::System => "system".to_string(),
                        };

                        self.success_message = Some("Settings saved successfully".to_string());
                    }
                    Err(e) => {
                        self.error = Some(e);
                    }
                }

                Command::none()
            }
            Message::ClearCache => {
                self.is_clearing_cache = true;
                self.error = None;
                self.success_message = None;

                let db = self.db.clone();

                Command::perform(
                    async move {
                        if let Ok(db_guard) = db.lock() {
                            match db_guard.clear_cache() {
                                Ok(()) => Ok(()),
                                Err(e) => Err(format!("Failed to clear cache: {}", e)),
                            }
                        } else {
                            Err("Failed to access database".to_string())
                        }
                    },
                    Message::ClearCacheCompleted,
                )
            }
            Message::ClearCacheCompleted(result) => {
                self.is_clearing_cache = false;

                match result {
                    Ok(()) => {
                        self.success_message = Some("Cache cleared successfully".to_string());
                    }
                    Err(e) => {
                        self.error = Some(e);
                    }
                }

                Command::none()
            }
            Message::ClearAuth => {
                self.is_clearing_auth = true;
                self.error = None;
                self.success_message = None;

                let db = self.db.clone();

                Command::perform(
                    async move {
                        if let Ok(db_guard) = db.lock() {
                            match db_guard.clear_auth() {
                                Ok(()) => Ok(()),
                                Err(e) => {
                                    Err(format!("Failed to clear authentication data: {}", e))
                                }
                            }
                        } else {
                            Err("Failed to access database".to_string())
                        }
                    },
                    Message::ClearAuthCompleted,
                )
            }
            Message::ClearAuthCompleted(result) => {
                self.is_clearing_auth = false;

                match result {
                    Ok(()) => {
                        self.success_message =
                            Some("Authentication data cleared successfully".to_string());
                    }
                    Err(e) => {
                        self.error = Some(e);
                    }
                }

                Command::none()
            }
            Message::Error(e) => {
                self.error = Some(e);
                self.success_message = None;
                Command::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let mut content = column![].spacing(20).padding(30);

        // Page title
        content = content.push(
            text("Settings")
                .size(30)
                .width(Length::Fill)
                .horizontal_alignment(iced::alignment::Horizontal::Center),
        );

        // Error message if any
        if let Some(error) = &self.error {
            content = content.push(
                container(
                    text(error)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(
                            0.8, 0.2, 0.2,
                        )))
                        .size(16),
                )
                .width(Length::Fill)
                .padding(10),
            );
        }

        // Success message if any
        if let Some(success) = &self.success_message {
            content = content.push(
                container(
                    text(success)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(
                            0.2, 0.7, 0.3,
                        )))
                        .size(16),
                )
                .width(Length::Fill)
                .padding(10),
            );
        }

        // Theme settings
        content = content.push(
            column![
                text("Theme").size(18),
                row![
                    button(text("Light"))
                        .on_press(Message::ThemeSelected(Theme::Light))
                        .padding(10)
                        .style(if matches!(self.theme, Theme::Light) {
                            iced::theme::Button::Primary
                        } else {
                            iced::theme::Button::Secondary
                        }),
                    button(text("Dark"))
                        .on_press(Message::ThemeSelected(Theme::Dark))
                        .padding(10)
                        .style(if matches!(self.theme, Theme::Dark) {
                            iced::theme::Button::Primary
                        } else {
                            iced::theme::Button::Secondary
                        }),
                    button(text("System"))
                        .on_press(Message::ThemeSelected(Theme::System))
                        .padding(10)
                        .style(if matches!(self.theme, Theme::System) {
                            iced::theme::Button::Primary
                        } else {
                            iced::theme::Button::Secondary
                        }),
                ]
                .spacing(10)
            ]
            .spacing(10),
        );

        // Language settings
        let languages = vec!["en", "ja", "fr", "de", "es"];
        content = content.push(
            column![
                text("Language").size(18),
                pick_list(languages, Some(&self.language), |lang: &str| {
                    Message::LanguageChanged(lang.to_string())
                })
                .width(Length::Fixed(200.0))
            ]
            .spacing(10),
        );

        // Offline mode
        content = content.push(
            column![
                text("Offline Mode").size(18),
                toggler(
                    String::from("Enable offline mode"),
                    self.offline_mode,
                    Message::OfflineModeToggled
                )
                .width(Length::Fixed(200.0))
            ]
            .spacing(10),
        );

        // API settings
        content = content.push(
            column![
                text("API Configuration").size(18),
                text("Client ID").size(14),
                text_input("Client ID", &self.client_id)
                    .padding(10)
                    .on_input(Message::ClientIdChanged),
                text("Client Secret").size(14),
                text_input("Client Secret", &self.client_secret)
                    .padding(10)
                    .on_input(Message::ClientSecretChanged),
                text("Redirect URI").size(14),
                text_input("Redirect URI", &self.redirect_uri)
                    .padding(10)
                    .on_input(Message::RedirectUriChanged),
                button(text(if self.is_saving {
                    "Saving..."
                } else {
                    "Save Settings"
                }))
                .on_press(Message::SaveConfig)
                .padding(10)
                .width(Length::Fixed(200.0))
            ]
            .spacing(10),
        );

        // Cache and auth settings
        content = content.push(
            column![
                text("Data Management").size(18),
                row![
                    button(text(if self.is_clearing_cache {
                        "Clearing..."
                    } else {
                        "Clear Cache"
                    }))
                    .on_press(Message::ClearCache)
                    .padding(10),
                    button(text(if self.is_clearing_auth {
                        "Clearing..."
                    } else {
                        "Clear Auth Data"
                    }))
                    .on_press(Message::ClearAuth)
                    .padding(10),
                ]
                .spacing(10)
            ]
            .spacing(10),
        );

        // About section
        content = content.push(
            column![
                text("About").size(18),
                text("AniList Desktop Client v0.1.0"),
                text("Created with Rust and Iced"),
                text("© 2025"),
            ]
            .spacing(5),
        );

        scrollable(content).height(Length::Fill).into()
    }
}
