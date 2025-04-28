use iced::widget::{column, row, scrollable, text};
use iced::{Element, Length};

use crate::api::models::MediaListEntry;
use crate::ui::components::anime_card::{AnimeCard, Message as CardMessage};

pub struct MediaList {
    entries: Vec<MediaListEntry>,
    on_select: Option<Box<dyn Fn(i32) -> Message + 'static>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    CardClicked(i32),
    Selected(i32),
}

impl MediaList {
    pub fn new(entries: Vec<MediaListEntry>) -> Self {
        Self {
            entries,
            on_select: None,
        }
    }

    pub fn on_select<F>(mut self, f: F) -> Self
    where
        F: Fn(i32) -> Message + 'static,
    {
        self.on_select = Some(Box::new(f));
        self
    }

    pub fn view(&self) -> Element<Message> {
        let mut list_content = column![text("Your List").size(24)].spacing(10).padding(20);

        // Group entries by status
        let mut current = Vec::new();
        let mut planning = Vec::new();
        let mut completed = Vec::new();
        let mut other = Vec::new();

        for entry in &self.entries {
            let target = match entry.status.as_str() {
                "CURRENT" => &mut current,
                "PLANNING" => &mut planning,
                "COMPLETED" => &mut completed,
                _ => &mut other,
            };

            if let Some(media) = &entry.media {
                target.push((entry, media.clone())); // Clone the media here
            }
        }

        // Display Current section if not empty
        if !current.is_empty() {
            list_content = list_content.push(text("Currently Watching").size(18));

            // Create rows of 4 cards
            for chunk in current.chunks(4) {
                let mut row_content = row![].spacing(10);

                for (_entry, media) in chunk {
                    // For each card, create a new function that generates the card view
                    let card_id = media.id;

                    // Create a button directly with the anime info
                    let btn = iced::widget::button(
                        column![
                            text(
                                &media
                                    .title
                                    .romaji
                                    .clone()
                                    .unwrap_or_else(|| "Unknown".to_string())
                            )
                            .size(14),
                            text(format!("Score: {}", media.average_score.unwrap_or(0.0))).size(12)
                        ]
                        .spacing(5)
                        .padding(10)
                        .width(Length::Fixed(120.0)),
                    )
                    .on_press(Message::CardClicked(card_id))
                    .style(iced::theme::Button::Secondary);

                    // Add this button to the row
                    row_content = row_content.push(btn);
                }

                // Add the completed row to the list
                list_content = list_content.push(row_content);
            }
        }

        // Similar sections for Planning and Completed would go here...

        // The final scrollable container
        scrollable(list_content)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }
}
