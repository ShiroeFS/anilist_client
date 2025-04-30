use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Command, Element, Length};

use crate::api::client::AniListClient;

// Search result item
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: i32,
    pub title: String,
    pub image_url: String,
    pub format: String,
    pub episodes: Option<i32>,
    pub year: Option<i32>,
    pub score: Option<f64>,
}

// Screen state
#[derive(Debug, Clone)]
pub struct SearchScreen {
    query: String,
    results: Vec<SearchResult>,
    page: i32,
    has_next_page: bool,
    is_loading: bool,
    error: Option<String>,
    client: AniListClient,
}

#[derive(Debug, Clone)]
pub enum Message {
    QueryChanged(String),
    Search,
    LoadMore,
    ResultsReceived(Result<(Vec<SearchResult>, bool), String>),
    AnimeSelected(i32),
    Error(String),
}

impl SearchScreen {
    pub fn new(client: AniListClient) -> Self {
        Self {
            query: String::new(),
            results: Vec::new(),
            page: 1,
            has_next_page: false,
            is_loading: false,
            error: None,
            client,
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::QueryChanged(query) => {
                self.query = query;
                Command::none()
            }
            Message::Search => {
                if self.query.trim().is_empty() {
                    self.error = Some("Please enter a search query".to_string());
                    return Command::none();
                }

                // Reset results and page for a new search
                self.results.clear();
                self.page = 1;
                self.is_loading = true;
                self.error = None;

                self.execute_search()
            }
            Message::LoadMore => {
                if self.is_loading || !self.has_next_page {
                    return Command::none();
                }

                self.page += 1;
                self.is_loading = true;

                self.execute_search()
            }
            Message::ResultsReceived(result) => {
                self.is_loading = false;

                match result {
                    Ok((new_results, has_next_page)) => {
                        // Append new results to existing ones
                        self.results.extend(new_results);
                        self.has_next_page = has_next_page;
                        self.error = None;
                    }
                    Err(e) => {
                        self.error = Some(format!("Search failed: {}", e));
                    }
                }

                Command::none()
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

    fn execute_search(&self) -> Command<Message> {
        let query = self.query.clone();
        let page = self.page;
        let client = self.client.clone();

        Command::perform(
            async move {
                // Execute the search with pagination
                match client.search_anime(query, Some(page), Some(25)).await {
                    Ok(data) => {
                        if let Some(page_data) = data.page {
                            // Extract pagination info
                            let has_next_page = page_data
                                .page_info
                                .as_ref()
                                .and_then(|info| info.has_next_page)
                                .unwrap_or(false);

                            // Transform media items to our model
                            let results = page_data
                                .media
                                .unwrap_or_default()
                                .into_iter()
                                .filter_map(|media_option| {
                                    let media = media_option?;

                                    // Extract title
                                    let title = media
                                        .title
                                        .and_then(|t| t.romaji.or(t.english))
                                        .unwrap_or_else(|| "Unknown Title".to_string());

                                    // Extract image URL
                                    let image_url = media
                                        .cover_image
                                        .and_then(|img| img.medium)
                                        .unwrap_or_default();

                                    // Convert format enum to string
                                    let format = match media.format {
                                        Some(f) => format!("{:?}", f),
                                        None => "Unknown".to_string(),
                                    };

                                    Some(SearchResult {
                                        id: media.id as i32,
                                        title,
                                        image_url,
                                        format,
                                        episodes: media.episodes.map(|e| e as i32),
                                        year: media.season_year.map(|y| y as i32),
                                        score: media.average_score.map(|s| s as f64),
                                    })
                                })
                                .collect();

                            Ok((results, has_next_page))
                        } else {
                            Ok((Vec::new(), false))
                        }
                    }
                    Err(e) => Err(e.to_string()),
                }
            },
            Message::ResultsReceived,
        )
    }

    pub fn view(&self) -> Element<Message> {
        // Search bar
        let search_bar = row![
            text_input("Search anime by title...", &self.query)
                .padding(10)
                .on_input(Message::QueryChanged),
            button(text("Search")).on_press(Message::Search).padding(10),
        ]
        .spacing(10)
        .padding(10)
        .width(Length::Fill);

        // Results area
        let mut results_column = column![].spacing(15).padding(10);

        if self.results.is_empty() && !self.is_loading && self.error.is_none() {
            results_column = results_column.push(
                text("Search for anime to see results")
                    .size(18)
                    .width(Length::Fill)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
            );
        } else {
            // Display results in a grid-like layout (rows of 4)
            for chunk in self.results.chunks(4) {
                let mut row_content = row![].spacing(20);

                for result in chunk {
                    let result_card = column![
                        // This would be an actual image in a real implementation
                        container(text("Image"))
                            .width(Length::Fixed(120.0))
                            .height(Length::Fixed(180.0))
                            .center_x()
                            .center_y(),
                        text(&result.title)
                            .size(16)
                            .width(Length::Fill)
                            .horizontal_alignment(iced::alignment::Horizontal::Center),
                        text(format!(
                            "{} {}",
                            result.format,
                            result
                                .episodes
                                .map_or_else(String::new, |e| format!("• {} eps", e))
                        ))
                        .size(12)
                        .width(Length::Fill)
                        .horizontal_alignment(iced::alignment::Horizontal::Center),
                        text(format!(
                            "{}{}",
                            result.year.map_or_else(String::new, |y| y.to_string()),
                            result
                                .score
                                .map_or_else(String::new, |s| format!(" • Score: {:.1}", s / 10.0))
                        ))
                        .size(12)
                        .width(Length::Fill)
                        .horizontal_alignment(iced::alignment::Horizontal::Center),
                    ]
                    .spacing(5)
                    .width(Length::Fixed(150.0))
                    .align_items(Alignment::Center);

                    // Make the card clickable
                    let result_id = result.id;
                    row_content = row_content.push(
                        button(result_card)
                            .on_press(Message::AnimeSelected(result_id))
                            .style(iced::theme::Button::Text)
                            .width(Length::Fixed(150.0)),
                    );
                }

                results_column = results_column.push(row_content);
            }

            // Add "Load More" button if there are more results
            if self.has_next_page {
                results_column = results_column.push(
                    button(text(if self.is_loading {
                        "Loading..."
                    } else {
                        "Load More"
                    }))
                    .on_press(Message::LoadMore)
                    .padding(10)
                    .width(Length::Fill),
                );
            }
        }

        // Loading indicator
        let loading_indicator: Element<Message> = if self.is_loading && self.results.is_empty() {
            container(text("Searching...").size(18))
                .width(Length::Fill)
                .center_x()
                .padding(20)
                .into()
        } else {
            container(text("")).into()
        };

        // Error message if any
        let error_display: Element<Message> = if let Some(error) = &self.error {
            container(
                text(error)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(
                        0.8, 0.2, 0.2,
                    )))
                    .size(16),
            )
            .width(Length::Fill)
            .padding(10)
            .into()
        } else {
            container(text("")).into()
        };

        // Main content
        let content = column![search_bar, error_display, loading_indicator, results_column,]
            .spacing(10)
            .width(Length::Fill);

        scrollable(content).height(Length::Fill).into()
    }
}
