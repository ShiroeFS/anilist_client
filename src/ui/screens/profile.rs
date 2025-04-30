use iced::widget::{column, container, row, scrollable, text};
use iced::{Command, Element, Length};

use crate::api::client::AniListClient;

#[derive(Debug, Clone)]
pub struct UserProfile {
    pub id: i32,
    pub name: String,
    pub about: Option<String>,
    pub avatar_url: String,
    pub banner_url: Option<String>,
    pub anime_count: i32,
    pub anime_mean_score: f32,
    pub anime_minutes_watched: i32,
    pub manga_count: i32,
    pub manga_mean_score: f32,
    pub manga_chapters_read: i32,
    pub favorite_anime: Vec<FavoriteItem>,
    pub favorite_manga: Vec<FavoriteItem>,
    pub favorite_characters: Vec<FavoriteCharacter>,
}

#[derive(Debug, Clone)]
pub struct FavoriteItem {
    pub id: i32,
    pub title: String,
    pub image_url: String,
}

#[derive(Debug, Clone)]
pub struct FavoriteCharacter {
    pub id: i32,
    pub name: String,
    pub image_url: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadProfile(String),
    ProfileLoaded(Result<UserProfile, String>),
    FavoriteAnimeSelected(i32),
    FavoriteMangaSelected(i32),
    Error(String),
}

pub struct ProfileScreen {
    client: AniListClient,
    username: Option<String>,
    profile: Option<UserProfile>,
    is_loading: bool,
    error: Option<String>,
}

impl ProfileScreen {
    pub fn new(client: AniListClient) -> Self {
        Self {
            client,
            username: None,
            profile: None,
            is_loading: false,
            error: None,
        }
    }

    pub fn load(&mut self, username: String) -> Command<Message> {
        self.username = Some(username.clone());
        self.is_loading = true;
        self.error = None;

        Command::perform(async move { username }, Message::LoadProfile)
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::LoadProfile(username) => {
                let client = self.client.clone();

                Command::perform(
                    async move {
                        match client.get_user_profile(username).await {
                            Ok(data) => {
                                if let Some(user) = data.user {
                                    // Extract user data
                                    let avatar_url = user
                                        .avatar
                                        .as_ref()
                                        .and_then(|a| a.large.clone().or(a.medium.clone()))
                                        .unwrap_or_default();

                                    // Clone statistics for multiple uses
                                    let statistics = user.statistics.clone();

                                    // Extract anime statistics
                                    let (anime_count, anime_mean_score, anime_minutes_watched) =
                                        if let Some(stats) =
                                            statistics.as_ref().and_then(|s| s.anime.as_ref())
                                        {
                                            (
                                                stats.count as i32,
                                                stats.mean_score as f32,
                                                stats.minutes_watched as i32,
                                            )
                                        } else {
                                            (0, 0.0, 0)
                                        };

                                    // Extract manga statistics
                                    let (manga_count, manga_mean_score, manga_chapters_read) =
                                        if let Some(stats) =
                                            statistics.as_ref().and_then(|s| s.manga.as_ref())
                                        {
                                            (
                                                stats.count as i32,
                                                stats.mean_score as f32,
                                                stats.chapters_read as i32,
                                            )
                                        } else {
                                            (0, 0.0, 0)
                                        };

                                    // Extract favorite anime
                                    let favorite_anime = if let Some(favorites) =
                                        user.favourites.as_ref()
                                    {
                                        if let Some(anime) = favorites.anime.as_ref() {
                                            anime
                                                .nodes
                                                .as_ref()
                                                .map(|nodes| {
                                                    nodes
                                                        .iter()
                                                        .filter_map(|node| {
                                                            let node = node.as_ref()?;
                                                            let title = node
                                                                .title
                                                                .as_ref()
                                                                .and_then(|t| {
                                                                    t.romaji
                                                                        .clone()
                                                                        .or(t.english.clone())
                                                                })
                                                                .unwrap_or_default();

                                                            let image_url = node
                                                                .cover_image
                                                                .as_ref()
                                                                .and_then(|img| img.medium.clone())
                                                                .unwrap_or_default();

                                                            Some(FavoriteItem {
                                                                id: node.id as i32,
                                                                title,
                                                                image_url,
                                                            })
                                                        })
                                                        .collect()
                                                })
                                                .unwrap_or_default()
                                        } else {
                                            Vec::new()
                                        }
                                    } else {
                                        Vec::new()
                                    };

                                    // Extract favorite manga
                                    let favorite_manga = if let Some(favorites) =
                                        user.favourites.as_ref()
                                    {
                                        if let Some(manga) = favorites.manga.as_ref() {
                                            manga
                                                .nodes
                                                .as_ref()
                                                .map(|nodes| {
                                                    nodes
                                                        .iter()
                                                        .filter_map(|node| {
                                                            let node = node.as_ref()?;
                                                            let title = node
                                                                .title
                                                                .as_ref()
                                                                .and_then(|t| {
                                                                    t.romaji
                                                                        .clone()
                                                                        .or(t.english.clone())
                                                                })
                                                                .unwrap_or_default();

                                                            let image_url = node
                                                                .cover_image
                                                                .as_ref()
                                                                .and_then(|img| img.medium.clone())
                                                                .unwrap_or_default();

                                                            Some(FavoriteItem {
                                                                id: node.id as i32,
                                                                title,
                                                                image_url,
                                                            })
                                                        })
                                                        .collect()
                                                })
                                                .unwrap_or_default()
                                        } else {
                                            Vec::new()
                                        }
                                    } else {
                                        Vec::new()
                                    };

                                    // Extract favorite characters
                                    let favorite_characters = if let Some(favorites) =
                                        user.favourites.as_ref()
                                    {
                                        if let Some(characters) = favorites.characters.as_ref() {
                                            characters
                                                .nodes
                                                .as_ref()
                                                .map(|nodes| {
                                                    nodes
                                                        .iter()
                                                        .filter_map(|node| {
                                                            let node = node.as_ref()?;
                                                            let name = node
                                                                .name
                                                                .as_ref()
                                                                .and_then(|n| n.full.clone())
                                                                .unwrap_or_default();

                                                            let image_url = node
                                                                .image
                                                                .as_ref()
                                                                .and_then(|img| img.medium.clone())
                                                                .unwrap_or_default();

                                                            Some(FavoriteCharacter {
                                                                id: node.id as i32,
                                                                name,
                                                                image_url,
                                                            })
                                                        })
                                                        .collect()
                                                })
                                                .unwrap_or_default()
                                        } else {
                                            Vec::new()
                                        }
                                    } else {
                                        Vec::new()
                                    };

                                    let profile = UserProfile {
                                        id: user.id as i32,
                                        name: user.name,
                                        about: user.about,
                                        avatar_url,
                                        banner_url: user.banner_image,
                                        anime_count,
                                        anime_mean_score,
                                        anime_minutes_watched,
                                        manga_count,
                                        manga_mean_score,
                                        manga_chapters_read,
                                        favorite_anime,
                                        favorite_manga,
                                        favorite_characters,
                                    };

                                    Ok(profile)
                                } else {
                                    Err("User not found".to_string())
                                }
                            }
                            Err(e) => Err(e.to_string()),
                        }
                    },
                    Message::ProfileLoaded,
                )
            }
            Message::ProfileLoaded(result) => {
                self.is_loading = false;

                match result {
                    Ok(profile) => {
                        self.profile = Some(profile);
                        self.error = None;
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to load profile: {}", e));
                    }
                }

                Command::none()
            }
            Message::FavoriteAnimeSelected(_) | Message::FavoriteMangaSelected(_) => {
                // These would be handled by the parent component
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
            let username = self.username.as_deref().unwrap_or("user");

            return container(
                text(format!("Loading {}'s profile...", username))
                    .size(20)
                    .width(Length::Fill)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
            )
            .width(Length::Fill)
            .padding(40)
            .into();
        }

        if let Some(profile) = &self.profile {
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

            // User header
            let header = row![
                // Avatar image (placeholder for now)
                container(text("Avatar"))
                    .width(Length::Fixed(100.0))
                    .height(Length::Fixed(100.0))
                    .center_x()
                    .center_y()
                    .style(iced::theme::Container::Box),
                // User info
                column![
                    text(&profile.name).size(30),
                    if let Some(about) = &profile.about {
                        container(text(about).size(14)).into()
                    } else {
                        container(text("")).into()
                    },
                ]
                .spacing(10)
                .width(Length::Fill),
            ]
            .spacing(20);

            content = content.push(header);

            // Statistics section
            let stats_section = column![
                text("Statistics").size(24),
                row![
                    // Anime stats
                    column![
                        text("Anime").size(18),
                        text(format!("Count: {}", profile.anime_count)),
                        text(format!("Mean Score: {:.1}", profile.anime_mean_score)),
                        text(format!(
                            "Minutes Watched: {}",
                            profile.anime_minutes_watched
                        )),
                    ]
                    .spacing(5)
                    .padding(10)
                    .width(Length::FillPortion(1)),
                    // Manga stats
                    column![
                        text("Manga").size(18),
                        text(format!("Count: {}", profile.manga_count)),
                        text(format!("Mean Score: {:.1}", profile.manga_mean_score)),
                        text(format!("Chapters Read: {}", profile.manga_chapters_read)),
                    ]
                    .spacing(5)
                    .padding(10)
                    .width(Length::FillPortion(1)),
                ]
                .spacing(20),
            ]
            .spacing(10);

            content = content.push(stats_section);

            // Favorites section
            if !profile.favorite_anime.is_empty() {
                let mut favorites_section = column![text("Favorite Anime").size(20),].spacing(10);

                // Create a row of favorites
                let mut row_content = row![].spacing(15);

                for (i, favorite) in profile.favorite_anime.iter().enumerate().take(6) {
                    let favorite_card = column![
                        // Image placeholder
                        container(text(""))
                            .width(Length::Fixed(80.0))
                            .height(Length::Fixed(120.0))
                            .center_x()
                            .style(iced::theme::Container::Box),
                        text(&favorite.title)
                            .size(14)
                            .width(Length::Fill)
                            .horizontal_alignment(iced::alignment::Horizontal::Center),
                    ]
                    .spacing(5)
                    .width(Length::Fixed(100.0))
                    .align_items(iced::Alignment::Center);

                    row_content = row_content.push(favorite_card);

                    // Create a new row after every 6 items
                    if (i + 1) % 6 == 0 && i > 0 {
                        favorites_section = favorites_section.push(row_content);
                        row_content = row![].spacing(15);
                    }
                }

                // Add any remaining items if row has content
                let has_remaining_content = row_content.children().count() > 0;
                if has_remaining_content {
                    favorites_section = favorites_section.push(row_content);
                }

                content = content.push(favorites_section);
            }

            // Favorite Manga section (similar to anime)
            if !profile.favorite_manga.is_empty() {
                let mut favorites_section = column![text("Favorite Manga").size(20),].spacing(10);

                // Create a row of favorites
                let mut row_content = row![].spacing(15);

                for (i, favorite) in profile.favorite_manga.iter().enumerate().take(6) {
                    let favorite_card = column![
                        // Image placeholder
                        container(text(""))
                            .width(Length::Fixed(80.0))
                            .height(Length::Fixed(120.0))
                            .center_x()
                            .style(iced::theme::Container::Box),
                        text(&favorite.title)
                            .size(14)
                            .width(Length::Fill)
                            .horizontal_alignment(iced::alignment::Horizontal::Center),
                    ]
                    .spacing(5)
                    .width(Length::Fixed(100.0))
                    .align_items(iced::Alignment::Center);

                    row_content = row_content.push(favorite_card);

                    // Create a new row after every 6 items
                    if (i + 1) % 6 == 0 && i > 0 {
                        favorites_section = favorites_section.push(row_content);
                        row_content = row![].spacing(15);
                    }
                }

                // Add any remaining items if row has content
                let has_remaining_content = row_content.children().count() > 0;
                if has_remaining_content {
                    favorites_section = favorites_section.push(row_content);
                }

                content = content.push(favorites_section);
            }

            // Favorite Characters section
            if !profile.favorite_characters.is_empty() {
                let mut favorites_section =
                    column![text("Favorite Characters").size(20),].spacing(10);

                // Create a row of favorites
                let mut row_content = row![].spacing(15);

                for (i, character) in profile.favorite_characters.iter().enumerate().take(6) {
                    let favorite_card = column![
                        // Image placeholder
                        container(text(""))
                            .width(Length::Fixed(80.0))
                            .height(Length::Fixed(120.0))
                            .center_x()
                            .style(iced::theme::Container::Box),
                        text(&character.name)
                            .size(14)
                            .width(Length::Fill)
                            .horizontal_alignment(iced::alignment::Horizontal::Center),
                    ]
                    .spacing(5)
                    .width(Length::Fixed(100.0))
                    .align_items(iced::Alignment::Center);

                    row_content = row_content.push(favorite_card);

                    // Create a new row after every 6 items
                    if (i + 1) % 6 == 0 && i > 0 {
                        favorites_section = favorites_section.push(row_content);
                        row_content = row![].spacing(15);
                    }
                }

                // Add any remaining items if row has content
                let has_remaining_content = row_content.children().count() > 0;
                if has_remaining_content {
                    favorites_section = favorites_section.push(row_content);
                }

                content = content.push(favorites_section);
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
                text("Select a user to view their profile")
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
