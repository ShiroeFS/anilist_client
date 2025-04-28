// use iced::widget::{button, column, container, row, scrollable, text, text_input};
// use iced::{Alignment, Color, Command, Element, Length};

// use crate::api::client::{AniListClient, Media};
// use crate::utils::error::AppError;

// // Search screen state
// #[derive(Debug, Clone)]
// pub struct SearchScreen {
//     query: String,
//     results: Vec<Media>,
//     is_loading: bool,
//     error: Option<String>,
//     api_client: AniListClient,
// }

// #[derive(Debug, Clone)]
// pub enum Message {
//     QueryChanged(String),
//     Search,
//     ResultsReceived(Result<Vec<Media>, AppError>),
//     AnimeSelected(i32),
// }

// impl SearchScreen {
//     pub fn new(api_client: AniListClient) -> Self {
//         Self {
//             query: String::new(),
//             results: Vec::new(),
//             is_loading: false,
//             error: None,
//             api_client,
//         }
//     }

//     pub fn update(&mut self, message: Message) -> Command<Message> {}
// }
pub fn search_screen() {
    // Placeholder for search screen implementation
}
