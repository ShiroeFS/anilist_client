use iced::widget::{button, column, container, text};
use iced::{Element, Length};

use crate::api::models::Media;

pub struct AnimeCard {
    media: Media,
    on_click: Option<Box<dyn Fn(i32) -> Message + 'static>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Clicked(i32),
}

impl AnimeCard {
    pub fn new(media: Media) -> Self {
        Self {
            media,
            on_click: None,
        }
    }

    pub fn on_click<F>(mut self, f: F) -> Self
    where
        F: Fn(i32) -> Message + 'static,
    {
        self.on_click = Some(Box::new(f));
        self
    }

    pub fn view(&self) -> Element<Message> {
        let title = self
            .media
            .title
            .romaji
            .as_ref()
            .or(self.media.title.english.as_ref())
            .unwrap_or(&"Unknown".to_string())
            .clone();

        let _image_url = self
            .media
            .cover_image
            .as_ref()
            .and_then(|cover| cover.medium.clone())
            .unwrap_or_default();

        // For now, we'll use a placeholder instead of loading the actual image
        let image_placeholder = container(text("Image"))
            .width(Length::Fixed(100.0))
            .height(Length::Fixed(150.0));

        let card_content = column![
            image_placeholder,
            text(&title).size(14),
            text(format!(
                "Score: {}",
                self.media.average_score.unwrap_or(0.0)
            ))
            .size(12),
        ]
        .spacing(5)
        .padding(10)
        .width(Length::Fixed(120.0));

        let card_container = container(card_content).style(iced::theme::Container::Box);

        if let Some(on_click) = &self.on_click {
            button(card_container)
                .on_press(on_click(self.media.id))
                .into()
        } else {
            card_container.into()
        }
    }
}
