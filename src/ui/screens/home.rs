use iced::widget::{column, container, scrollable, text};
use iced::{Command, Element, Length};
use iced_native::Element;

use crate::api::client::AniListClient;
use crate::api::models::{Media, MediaListEntry};
use crate::ui::components::media_list::{MediaList, Message as MediaListMessage};

#[derive(Debug, Clone)]
pub enum Message {
    LoadUserData,
    UserDataLoaded(Result<Vec<MediaListEntry>, String>),
    MediaListMessage(MediaListMessage),
    AnimeSelected(i32),
    Error(String),
}

pub struct HomeScreen {
    client: AniListClient,
    user_id: Option<i32>,
    username: Option<String>,
    currently_watching: Vec<MediaListEntry>,
    is_authenticated: bool,
    is_loading: bool,
    error: Option<String>,
}

impl HomeScreen {
    pub fn new(client: AniListClient) -> Self {
        Self {
            client,
            user_id: None,
            username: None,
            currently_watching: Vec::new(),
            is_authenticated: false,
            is_loading: false,
            error: None,
        }
    }

    pub fn init(&mut self) -> Command<Message> {
        Command::perform(async { () }, |_| Message::LoadUserData)
    }

    pub fn view(&self) -> Element<Message> {
        // Create content based on state
        let content = if self.is_loading {
            column![text("Loading your anime list...").size(20)]
                .spacing(20)
                .padding(40)
        } else if !self.is_authenticated {
            column![
                text("Welcome to AniList Desktop").size(30),
                text("Please log in to see your anime list").size(18),
            ]
            .spacing(20)
            .padding(40)
        } else if self.currently_watching.is_empty() {
            column![
                text("Welcome to AniList Desktop").size(30),
                text("You don't have any anime in your 'Currently Watching' list").size(18),
            ]
            .spacing(20)
            .padding(40)
        } else {
            // Create a MediaList widget with our data - create owned clone to avoid borrowing
            let entries_clone = self.currently_watching.clone();
            column![
                text("Currently Watching").size(30),
                MediaList::new(entries_clone)
                    .on_select(|id| MediaListMessage::Selected(id))
                    .view()
                    .map(Message::MediaListMessage)
            ]
            .spacing(20)
            .padding(40)
        };

        // Error message if any
        let content_with_error = if let Some(error) = &self.error {
            column![
                text(error)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(
                        0.8, 0.2, 0.2
                    )))
                    .size(16),
                content
            ]
            .spacing(20)
        } else {
            content
        };

        // Create a container
        let container_element = container(content_with_error)
            .width(Length::Fill)
            .padding(20);

        // Create scrollable element with a separately owned container to avoid reference issues
        let scrollable_element = scrollable(container_element).height(Length::Fill);

        // Return element without borrowing
        Element::from(scrollable_element)
    }

    pub fn is_authenticated(&self) -> bool {
        self.is_authenticated
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::LoadUserData => {
                // First, check if we're authenticated
                let client = self.client.clone();
                self.is_loading = true;

                Command::perform(
                    async move {
                        // Try to get the current user (Viewer)
                        match client.get_viewer().await {
                            Ok(viewer_data) => {
                                if let Some(viewer) = viewer_data.viewer {
                                    let user_id = viewer.id as i32;

                                    // Now fetch the user's anime list
                                    match client.get_user_anime_list(
                                        user_id,
                                    Some(crate::api::client::user_anime_list::MediaListStatus::CURRENT)
                                ).await {
                                    Ok(list_data) => {
                                        // Convert to MediaListEntry objects
                                        let mut entries = Vec::new();

                                        if let Some(collection) = list_data.media_list_collection {
                                            for list_opt in collection.lists.unwrap_or_default() {
                                                if let Some(list) = list_opt {
                                                    if let Some(list_entries) = list.entries {
                                                        for entry in list_entries {
                                                            if let Some(entry) = entry {
                                                                // Convert to our internal model
                                                                let media_list_entry = MediaListEntry {
                                                                    id: entry.id as i32,
                                                                    media_id: entry.media_id as i32,
                                                                    status: entry.status.map_or("UNKNOWN".to_string(), |s| format!("{:?}", s)),
                                                                    score: entry.score,
                                                                    progress: entry.progress.map(|p| p as i32),
                                                                    updated_at: entry.updated_at.unwrap_or(0),
                                                                    media: entry.media.map(|m| {
                                                                        crate::api::models::Media {
                                                                            id: m.id as i32,
                                                                            title: crate::api::models::MediaTitle {
                                                                                romaji: m.title.as_ref().and_then(|t| t.romaji.clone()),
                                                                                english: m.title.as_ref().and_then(|t| t.english.clone()),
                                                                                native: m.title.as_ref().and_then(|t| t.native.clone()),
                                                                            },
                                                                            description: None,
                                                                            episodes: m.episodes.map(|e| e as i32),
                                                                            duration: None,
                                                                            genres: None,
                                                                            average_score: None,
                                                                            cover_image: m.cover_image.map(|img| {
                                                                                crate::api::models::MediaCoverImage {
                                                                                    large: None,
                                                                                    medium: img.medium,
                                                                                }
                                                                            }),
                                                                            banner_image: None,
                                                                            status: m.status.map(|s| format!("{:?}", s)),
                                                                            format: m.format.map(|f| format!("{:?}", f)),
                                                                        }
                                                                    }),
                                                                };

                                                                entries.push(media_list_entry);
                                                }
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        Ok((user_id, viewer.name, entries))
                                    },
                                    Err(e) => Err(format!("Failed to load anime list: {}", e)),
                                }
                                } else {
                                    Err("Could not get user information".to_string())
                                }
                            }
                            Err(e) => {
                                // Check if it's an authentication error
                                if e.to_string().contains("Authentication required") {
                                    Err("Not authenticated".to_string())
                                } else {
                                    Err(format!("Failed to get user: {}", e))
                                }
                            }
                        }
                    },
                    |result| match result {
                        Ok((_, _, entries)) => Message::UserDataLoaded(Ok(entries)),
                        Err(e) => {
                            if e == "Not authenticated" {
                                Message::UserDataLoaded(Err(e))
                            } else {
                                Message::Error(e)
                            }
                        }
                    },
                )
            }
            Message::UserDataLoaded(result) => {
                self.is_loading = false;

                match result {
                    Ok(entries) => {
                        self.currently_watching = entries;
                        self.is_authenticated = true;
                        self.error = None;
                    }
                    Err(e) => {
                        if e == "Not authenticated" {
                            self.is_authenticated = false;
                        } else {
                            self.error = Some(e);
                        }
                    }
                }

                Command::none()
            }
            Message::MediaListMessage(media_list_msg) => {
                match media_list_msg {
                    MediaListMessage::Selected(id) => {
                        // The user selected an anime, propagate the message up
                        Command::perform(async move { id }, Message::AnimeSelected)
                    }
                    MediaListMessage::CardClicked(id) => {
                        // The user clicked a card, propagate the message up
                        Command::perform(async move { id }, Message::AnimeSelected)
                    }
                }
            }
            Message::AnimeSelected(_) => {
                // This will be handled by the parent component
                Command::none()
            }
            Message::Error(e) => {
                self.error = Some(e);
                self.is_loading = false;
                Command::none()
            }
        }
    }
}
