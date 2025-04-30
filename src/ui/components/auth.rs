use iced::widget::{button, column, container, row, text};
use iced::{Command, Element, Length};

use crate::api::auth::AuthManager;

#[derive(Debug, Clone)]
pub enum State {
    Idle,
    Authenticating,
    Authenticated { username: String },
    Failed { error: String },
    LoggingOut,
}

#[derive(Debug, Clone)]
pub enum Message {
    LoginPressed,
    LoginCompleted(Result<String, String>),
    LogoutPressed,
    LogoutCompleted(Result<(), String>),
}

#[derive(Clone)]
pub struct AuthComponent {
    state: State,
    auth_manager: AuthManager,
}

impl AuthComponent {
    pub fn new(auth_manager: AuthManager) -> Self {
        Self {
            state: State::Idle,
            auth_manager,
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::LoginPressed => {
                self.state = State::Authenticating;

                let auth_manager = self.auth_manager.clone();
                Command::perform(
                    async move {
                        match auth_manager.authenticate().await {
                            Ok(token) => {
                                // Use the token to get the user's name
                                let client = reqwest::Client::new();
                                let response = client
                                    .post("https://graphql.anilist.co")
                                    .header(
                                        "Authorization",
                                        format!("Bearer {}", token.access_token),
                                    )
                                    .json(&serde_json::json!({
                                        "query": "query { Viewer { name } }"
                                    }))
                                    .send()
                                    .await;

                                match response {
                                    Ok(res) => {
                                        if let Ok(json) = res.json::<serde_json::Value>().await {
                                            if let Some(name) = json
                                                .get("data")
                                                .and_then(|data| data.get("Viewer"))
                                                .and_then(|viewer| viewer.get("name"))
                                                .and_then(|name| name.as_str())
                                            {
                                                Ok(name.to_string())
                                            } else {
                                                Ok("User".to_string())
                                            }
                                        } else {
                                            Ok("User".to_string())
                                        }
                                    }
                                    Err(_) => Ok("User".to_string()),
                                }
                            }
                            Err(e) => Err(format!("Authentication failed: {}", e)),
                        }
                    },
                    Message::LoginCompleted,
                )
            }
            Message::LoginCompleted(result) => {
                match result {
                    Ok(username) => {
                        self.state = State::Authenticated { username };
                    }
                    Err(error) => {
                        self.state = State::Failed { error };
                    }
                }
                Command::none()
            }
            Message::LogoutPressed => {
                self.state = State::LoggingOut;

                let auth_manager = self.auth_manager.clone();
                Command::perform(
                    async move {
                        auth_manager
                            .logout()
                            .await
                            .map_err(|e| format!("Logout failed: {}", e))
                    },
                    Message::LogoutCompleted,
                )
            }
            Message::LogoutCompleted(result) => {
                match result {
                    Ok(()) => {
                        self.state = State::Idle;
                    }
                    Err(error) => {
                        self.state = State::Failed { error };
                    }
                }
                Command::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        match &self.state {
            State::Idle => container(
                column![
                    text("You are not logged in").size(16),
                    button(text("Login to AniList"))
                        .on_press(Message::LoginPressed)
                        .padding(10)
                ]
                .spacing(10)
                .padding(20),
            )
            .width(Length::Fill)
            .into(),
            State::Authenticating => container(
                column![
                    text("Authenticating...").size(16),
                    text("A browser window should open. Please log in to AniList.").size(14)
                ]
                .spacing(10)
                .padding(20),
            )
            .width(Length::Fill)
            .into(),
            State::Authenticated { username } => container(
                column![
                    row![
                        text("Logged in as ").size(16),
                        text(username).size(16).style(iced::theme::Text::Color(
                            iced::Color::from_rgb(0.2, 0.5, 0.8)
                        ))
                    ],
                    button(text("Logout"))
                        .on_press(Message::LogoutPressed)
                        .padding(10)
                ]
                .spacing(10)
                .padding(20),
            )
            .width(Length::Fill)
            .into(),
            State::Failed { error } => {
                container(
                    column![
                        text("Authentication Error").size(16),
                        text(error).size(14).style(iced::theme::Text::Color(
                            iced::Color::from_rgb(0.8, 0.2, 0.2)
                        )),
                        button(text("Try Again"))
                            .on_press(Message::LoginPressed)
                            .padding(10)
                    ]
                    .spacing(10)
                    .padding(20),
                )
                .width(Length::Fill)
                .into()
            }
            State::LoggingOut => container(column![text("Logging out...").size(16)].padding(20))
                .width(Length::Fill)
                .into(),
        }
    }

    pub fn is_authenticated(&self) -> bool {
        matches!(self.state, State::Authenticated { .. })
    }
}
