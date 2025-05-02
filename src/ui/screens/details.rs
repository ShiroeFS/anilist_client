use iced::widget::{button, column, container, row, scrollable, slider, text};
use iced::{Alignment, Command, Element, Length};

use crate::api::client::{update_media_list, AniListClient};

// Anime details
#[derive(Debug, Clone)]
pub struct AnimeDetails {
    pub id: i32,
    pub title: String,
    pub english_title: Option<String>,
    pub native_title: Option<String>,
    pub description: String,
    pub episodes: Option<i32>,
    pub duration: Option<i32>,
    pub genres: Vec<String>,
    pub score: f32,
    pub status: String,
    pub format: String,
    pub season: Option<String>,
    pub year: Option<i32>,
    pub cover_image: String,
    pub banner_image: Option<String>,
    pub studios: Vec<String>,
    pub character_previews: Vec<CharacterPreview>,
}

#[derive(Debug, Clone)]
pub struct CharacterPreview {
    pub id: i32,
    pub name: String,
    pub image_url: String,
    pub role: String,
}

#[derive(Debug, Clone)]
pub struct UserProgress {
    pub list_entry_id: Option<i32>,
    pub status: String,
    pub score: f32,
    pub progress: i32,
    pub max_progress: Option<i32>,
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadAnimeDetails(i32),
    AnimeDetailsLoaded(Result<AnimeDetails, String>),
    UserProgressLoaded(Result<UserProgress, String>),
    StatusChanged(String),
    ScoreChanged(f32),
    ProgressChanged(i32),
    SaveProgress,
    ProgressSaved(Result<(), String>),
    Error(String),
}

pub struct DetailsScreen {
    client: AniListClient,
    anime_id: Option<i32>,
    anime: Option<AnimeDetails>,
    user_progress: Option<UserProgress>,
    is_authenticated: bool,
    is_loading: bool,
    is_saving: bool,
    error: Option<String>,
    temp_status: Option<String>,
    temp_score: Option<f32>,
    temp_progress: Option<i32>,
}

impl DetailsScreen {
    pub fn new(client: AniListClient) -> Self {
        Self {
            client,
            anime_id: None,
            anime: None,
            user_progress: None,
            is_authenticated: false,
            is_loading: false,
            is_saving: false,
            error: None,
            temp_status: None,
            temp_score: None,
            temp_progress: None,
        }
    }

    pub fn load(&mut self, anime_id: i32) -> Command<Message> {
        self.anime_id = Some(anime_id);
        self.is_loading = true;
        self.error = None;
        self.anime = None;

        Command::perform(async move { anime_id }, Message::LoadAnimeDetails)
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::LoadAnimeDetails(id) => {
                // Create two separate client instances for each async operation
                let client1 = self.client.clone();
                let client2 = self.client.clone();

                Command::batch(vec![
                    // Load anime details
                    Command::perform(
                        async move {
                            match client1.get_anime_details(id).await {
                                Ok(data) => {
                                    if let Some(media) = data.media {
                                        // Convert to our model
                                        let title = media
                                            .title
                                            .as_ref()
                                            .and_then(|t| t.romaji.as_ref())
                                            .unwrap_or(&"Unknown Title".to_string())
                                            .clone();

                                        let english_title =
                                            media.title.as_ref().and_then(|t| t.english.clone());

                                        let native_title =
                                            media.title.as_ref().and_then(|t| t.native.clone());

                                        let description = media
                                            .description
                                            .unwrap_or_default()
                                            .replace("<br>", "\n")
                                            .replace("<i>", "")
                                            .replace("</i>", "");

                                        let cover_image = media
                                            .cover_image
                                            .as_ref()
                                            .and_then(|img| img.large.clone())
                                            .unwrap_or_default();

                                        let genres = media
                                            .genres
                                            .unwrap_or_default()
                                            .into_iter()
                                            .filter_map(|g| g)
                                            .collect();

                                        // Extract studio names
                                        let studios = if let Some(studio_conn) = media.studios {
                                            studio_conn
                                                .edges
                                                .unwrap_or_default()
                                                .into_iter()
                                                .filter_map(|edge| {
                                                    edge?
                                                        .node
                                                        .as_ref()
                                                        .map(|node| node.name.clone())
                                                })
                                                .collect()
                                        } else {
                                            Vec::new()
                                        };

                                        // Extract character previews
                                        let character_previews =
                                            if let Some(char_conn) = media.characters {
                                                char_conn
                                                    .edges
                                                    .unwrap_or_default()
                                                    .into_iter()
                                                    .filter_map(|edge| {
                                                        let node = edge.as_ref()?.node.as_ref()?;
                                                        let role = edge.as_ref()?.role.as_ref()?;

                                                        Some(CharacterPreview {
                                                            id: node.id as i32,
                                                            name: node
                                                                .name
                                                                .as_ref()
                                                                .and_then(|n| n.full.clone())
                                                                .unwrap_or_default(),
                                                            image_url: node
                                                                .image
                                                                .as_ref()
                                                                .and_then(|img| img.medium.clone())
                                                                .unwrap_or_default(),
                                                            role: format!("{:?}", role), // Convert enum to string
                                                        })
                                                    })
                                                    .collect()
                                            } else {
                                                Vec::new()
                                            };

                                        // Convert status and format enums to strings
                                        let status_str = media.status.map_or_else(
                                            || "Unknown".to_string(),
                                            |s| format!("{:?}", s),
                                        );

                                        let format_str = media.format.map_or_else(
                                            || "Unknown".to_string(),
                                            |f| format!("{:?}", f),
                                        );

                                        // Convert season enum to string
                                        let season_str = media.season.map(|s| format!("{:?}", s));

                                        let details = AnimeDetails {
                                            id: media.id as i32,
                                            title,
                                            english_title,
                                            native_title,
                                            description,
                                            episodes: media.episodes.map(|e| e as i32),
                                            duration: media.duration.map(|d| d as i32),
                                            genres,
                                            score: media.average_score.unwrap_or(0) as f32 / 10.0,
                                            status: status_str,
                                            format: format_str,
                                            season: season_str,
                                            year: media.season_year.map(|y| y as i32),
                                            cover_image,
                                            banner_image: media.banner_image,
                                            studios,
                                            character_previews,
                                        };

                                        Ok(details)
                                    } else {
                                        Err("Anime not found".to_string())
                                    }
                                }
                                Err(e) => Err(e.to_string()),
                            }
                        },
                        Message::AnimeDetailsLoaded,
                    ),
                    // Check authentication and load user progress if authenticated
                    Command::perform(
                        async move {
                            let client_clone = client2.clone();

                            // Check if user is authenticated
                            if client2.is_authenticated().await {
                                // Get viewer information to get user ID
                                match client2.get_viewer().await {
                                    Ok(viewer_data) => {
                                        if let Some(viewer) = viewer_data.viewer {
                                            let user_id = viewer.id as i32;

                                            // Get user's anime list to find this specific entry
                                            match client_clone
                                                .get_user_anime_list(user_id, None)
                                                .await
                                            {
                                                Ok(list_data) => {
                                                    if let Some(collection) =
                                                        list_data.media_list_collection
                                                    {
                                                        for list_opt in
                                                            collection.lists.unwrap_or_default()
                                                        {
                                                            if let Some(list) = list_opt {
                                                                if let Some(entries) = list.entries
                                                                {
                                                                    for entry in entries {
                                                                        if let Some(entry) = entry {
                                                                            if entry.media_id as i32
                                                                                == id
                                                                            {
                                                                                // Found the entry
                                                                                let progress = UserProgress {
                                                                                    list_entry_id: Some(entry.id as i32),
                                                                                    status: entry.status.map_or("PLANNING".to_string(), |s| format!("{:?}", s)),
                                                                                    score: entry.score.unwrap_or(0.0) as f32,
                                                                                    progress: entry.progress.unwrap_or(0) as i32,
                                                                                    max_progress: entry.media.as_ref()
                                                                                        .and_then(|m| m.episodes)
                                                                                        .map(|e| e as i32),
                                                                                };

                                                                                return Ok(
                                                                                    progress,
                                                                                );
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }

                                                        // Entry not found, return default
                                                        return Ok(UserProgress {
                                                            list_entry_id: None,
                                                            status: "PLANNING".to_string(),
                                                            score: 0.0,
                                                            progress: 0,
                                                            max_progress: None,
                                                        });
                                                    }
                                                }
                                                Err(e) => {
                                                    return Err(format!(
                                                        "Failed to load anime list: {}",
                                                        e
                                                    ))
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        return Err(format!("Failed to get user info: {}", e))
                                    }
                                }
                            }

                            // Not authenticated or user info not found
                            Err("Not authenticated".to_string())
                        },
                        Message::UserProgressLoaded,
                    ),
                ])
            }
            Message::AnimeDetailsLoaded(result) => {
                self.is_loading = false;

                match result {
                    Ok(details) => {
                        self.anime = Some(details);
                        self.error = None;
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to load anime details: {}", e));
                    }
                }

                Command::none()
            }
            Message::UserProgressLoaded(result) => {
                match result {
                    Ok(progress) => {
                        let progress_clone = progress.clone();
                        self.user_progress = Some(progress);
                        self.is_authenticated = true;
                        // Initialize temporary values for editing
                        self.temp_status = Some(progress_clone.status.clone());
                        self.temp_score = Some(progress_clone.score);
                        self.temp_progress = Some(progress_clone.progress);
                    }
                    Err(e) => {
                        if e == "Not authenticated" {
                            self.is_authenticated = false;
                        } else {
                            self.error = Some(format!("Failed to load user progress: {}", e));
                        }
                    }
                }

                Command::none()
            }
            Message::StatusChanged(status) => {
                self.temp_status = Some(status);
                Command::none()
            }
            Message::ScoreChanged(score) => {
                self.temp_score = Some(score);
                Command::none()
            }
            Message::ProgressChanged(progress) => {
                self.temp_progress = Some(progress);
                Command::none()
            }
            Message::SaveProgress => {
                if !self.is_authenticated {
                    self.error = Some("You must be logged in to save progress".to_string());
                    return Command::none();
                }

                if let (
                    Some(anime),
                    Some(progress),
                    Some(status),
                    Some(score),
                    Some(progress_val),
                ) = (
                    &self.anime,
                    &self.user_progress,
                    &self.temp_status,
                    &self.temp_score,
                    &self.temp_progress,
                ) {
                    self.is_saving = true;

                    // Extract and clone the required values for use in the async closure
                    let entry_id = progress.list_entry_id;
                    let media_id = Some(anime.id);
                    let status_str = status.clone();
                    let score_val = *score;
                    let progress_value = *progress_val;
                    let client = self.client.clone();

                    Command::perform(
                        async move {
                            // Convert status string to enum
                            let status_enum = match status_str.as_str() {
                                "CURRENT" => Some(update_media_list::MediaListStatus::CURRENT),
                                "PLANNING" => Some(update_media_list::MediaListStatus::PLANNING),
                                "COMPLETED" => Some(update_media_list::MediaListStatus::COMPLETED),
                                "DROPPED" => Some(update_media_list::MediaListStatus::DROPPED),
                                "PAUSED" => Some(update_media_list::MediaListStatus::PAUSED),
                                "REPEATING" => Some(update_media_list::MediaListStatus::REPEATING),
                                _ => None,
                            };

                            match client
                                .update_media_list(
                                    entry_id,
                                    media_id,
                                    status_enum,
                                    Some(score_val as f64),
                                    Some(progress_value),
                                )
                                .await
                            {
                                Ok(_) => Ok(()),
                                Err(e) => Err(e.to_string()),
                            }
                        },
                        Message::ProgressSaved,
                    )
                } else {
                    self.error = Some("Missing required data to save progress".to_string());
                    Command::none()
                }
            }
            Message::ProgressSaved(result) => {
                self.is_saving = false;

                match result {
                    Ok(_) => {
                        // Update user_progress with temporary values
                        if let Some(progress) = &mut self.user_progress {
                            if let (Some(status), Some(score), Some(progress_val)) =
                                (&self.temp_status, &self.temp_score, &self.temp_progress)
                            {
                                progress.status = status.clone();
                                progress.score = *score;
                                progress.progress = *progress_val;
                            }
                        }
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to save progress: {}", e));
                    }
                }

                Command::none()
            }
            Message::Error(e) => {
                self.error = Some(e);
                Command::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        if self.is_loading {
            return container(
                text("Loading anime details...")
                    .size(20)
                    .width(Length::Fill)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
            )
            .width(Length::Fill)
            .padding(40)
            .into();
        }

        if let Some(anime) = &self.anime {
            let mut content = column![].spacing(20).padding(30);

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

            // Title and banner area
            let title_section = column![
                text(&anime.title).size(30),
                if let Some(english) = &anime.english_title {
                    let english_text: Element<Message> = container(text(english).size(20)).into();
                    english_text
                } else {
                    container(text("")).into()
                },
                row![
                    text(&anime.format).size(16),
                    text(&anime.status).size(16),
                    if let Some(season) = &anime.season {
                        if let Some(year) = anime.year {
                            text(format!("{} {}", season, year)).size(16)
                        } else {
                            text(season).size(16)
                        }
                    } else if let Some(year) = anime.year {
                        text(format!("{}", year)).size(16)
                    } else {
                        text("")
                    }
                ]
                .spacing(10),
                text(format!("⭐ {:.1} / 10", anime.score * 10.0))
                    .size(16)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(
                        1.0, 0.8, 0.0
                    ))),
            ]
            .spacing(5);

            content = content.push(title_section);

            // Main info section with cover image and details
            let cover_and_details = row![
            // Cover image (placeholder for now)
            container(text("Cover Image"))
            .width(Length::Fixed(220.0))
            .height(Length::Fixed(320.0))
            .center_x()
            .center_y()
            .style(iced::theme::Container::Box),

            // Details section
            column![
                // User progress tracking if authenticated
                if self.is_authenticated {
                    if let (Some(progress), Some(status), Some(score), Some(progress_val)) =
                        (&self.user_progress, &self.temp_status, &self.temp_score, &self.temp_progress) {

                            let max_progress = progress.max_progress.unwrap_or(anime.episodes.unwrap_or(0));

                            container(
                                column![
                                    text("Your Progress").size(18),

                                    // Status selection
                                    text("Status").size(14),
                                    row![
                                        button(text("Watching"))
                                            .on_press(Message::StatusChanged("CURRENT".to_string()))
                                            .style(if status == "CURRENT" {
                                                iced::theme::Button::Primary
                                            } else {
                                                iced::theme::Button::Secondary
                                            })
                                            .padding(5),

                                        button(text("Completed"))
                                            .on_press(Message::StatusChanged("COMPLETED".to_string()))
                                            .style(if status == "COMPLETED" {
                                                iced::theme::Button::Primary
                                            } else {
                                                iced::theme::Button::Secondary
                                            })
                                            .padding(5),

                                        button(text("Plan to Watch"))
                                            .on_press(Message::StatusChanged("PLANNING".to_string()))
                                            .style(if status == "PLANNING" {
                                                iced::theme::Button::Primary
                                            } else {
                                                iced::theme::Button::Secondary
                                            })
                                            .padding(5),
                                    ]
                                    .spacing(5),

                                    row![
                                        button(text("Dropped"))
                                            .on_press(Message::StatusChanged("DROPPED".to_string()))
                                            .style(if status == "DROPPED" {
                                                iced::theme::Button::Primary
                                            } else {
                                                iced::theme::Button::Secondary
                                            })
                                            .padding(5),

                                        button(text("Paused"))
                                            .on_press(Message::StatusChanged("PAUSED".to_string()))
                                            .style(if status == "PAUSED" {
                                                iced::theme::Button::Primary
                                            } else {
                                                iced::theme::Button::Secondary
                                            })
                                            .padding(5),

                                        button(text("Rewatching"))
                                            .on_press(Message::StatusChanged("REPEATING".to_string()))
                                            .style(if status == "REPEATING" {
                                                iced::theme::Button::Primary
                                            } else {
                                                iced::theme::Button::Secondary
                                            })
                                            .padding(5),
                                    ]
                                    .spacing(5),

                                    // Score slider
                                    text(format!("Score: {:.1}", score)).size(14),
                                    slider(0.0..=10.0, *score, Message::ScoreChanged)
                                            .step(0.5),

                                            // Episode progress
                                            row![
                                                text(format!("Progress: {}", progress_val)).size(14),
                                                if max_progress > 0 {
                                                    container(text(format!("/ {}", max_progress))).into()
                                                } else {
                                                    container(text("")).into()
                                                }
                                            ],

                                            slider(0.0..=(max_progress as f32), *progress_val as f32, |v| {
                                                Message::ProgressChanged(v as i32)
                                            })
                                            .step(1.0),

                                            // Save button
                                            button(
                                                text(if self.is_saving { "Saving..." } else { "Save Progress" })
                                            )
                                            .on_press(Message::SaveProgress)
                                            .padding(10)
                                            .width(Length::Fill),
                                ]
                                .spacing(10)
                                .width(Length::Fill)
                            )
                            .padding(15)
                            .style(iced::theme::Container::Box)
                            .into()
                        } else {
                            container(text("")).into()
                        }
                } else {
                    container(
                        column![
                            text("Login to track your progress").size(18),
                            text("You can add this anime to your list and track your progress after logging in.")
                                .size(14)
                        ]
                        .spacing(5)
                        .padding(10)
                        .width(Length::Fill)
                    )
                    .padding(15)
                    .style(iced::theme::Container::Box)
                    .into()
                },

                // Basic information
                if let Some(episodes) = anime.episodes {
                    container(text(format!("Episodes: {}", episodes)).size(14)).into()
                } else {
                    container(text("Episodes: Unknown").size(14)).into()
                },

                if let Some(duration) = anime.duration {
                    container(text(format!("Duration: {} minutes", duration)).size(14)).into()
                } else {
                    container(text("")).into()
                },

                if !anime.genres.is_empty() {
                    container(text(format!("Genres: {}", anime.genres.join(", "))).size(14)).into()
                } else {
                    container(text("")).into()
                },

                if !anime.studios.is_empty() {
                    container(text(format!("Studios: {}", anime.studios.join(", "))).size(14)).into()
                } else {
                    container(text("")).into()
                },
            ]
            .spacing(10)
            .width(Length::Fill)
        ]
        .spacing(20);

            content = content.push(cover_and_details);

            // Synopsis
            content = content.push(
                column![text("Synopsis").size(20), text(&anime.description).size(14),].spacing(10),
            );

            // Characters preview
            if !anime.character_previews.is_empty() {
                let mut character_section = column![text("Characters").size(20),].spacing(15);

                // Create rows of 6 characters
                for chunk in anime.character_previews.chunks(6) {
                    let mut row_content = row![].spacing(15);

                    for character in chunk {
                        let character_card = column![
                            // Character image placeholder
                            container(text(""))
                                .width(Length::Fixed(80.0))
                                .height(Length::Fixed(120.0))
                                .center_x()
                                .style(iced::theme::Container::Box),
                            text(&character.name)
                                .size(14)
                                .width(Length::Fill)
                                .horizontal_alignment(iced::alignment::Horizontal::Center),
                            text(&character.role)
                                .size(12)
                                .width(Length::Fill)
                                .horizontal_alignment(iced::alignment::Horizontal::Center),
                        ]
                        .spacing(5)
                        .width(Length::Fixed(100.0))
                        .align_items(Alignment::Center);

                        row_content = row_content.push(character_card);
                    }

                    character_section = character_section.push(row_content);
                }

                content = content.push(character_section);
            }

            scrollable(content).height(Length::Fill).into()
        } else if let Some(error) = &self.error {
            container(
                text(error)
                    .size(18)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(
                        0.8, 0.2, 0.2,
                    ))),
            )
            .width(Length::Fill)
            .padding(40)
            .into()
        } else {
            container(
                text("Select an anime to view details")
                    .size(20)
                    .width(Length::Fill)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
            )
            .width(Length::Fill)
            .padding(40)
            .into()
        }
    }
}
