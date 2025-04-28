use iced::widget::{column, container, row, text};
use iced::{Element, Length};

use crate::api::models::User;

pub struct UserStats {
    user: User,
}

#[derive(Debug, Clone)]
pub enum Message {
    // No messages needed for this component yet
}

impl UserStats {
    pub fn new(user: User) -> Self {
        Self { user }
    }

    pub fn view(&self) -> Element<Message> {
        let anime_stats = column![
            text("Anime Stats").size(18),
            row![
                text("Total Anime:"),
                text("N/A") // This would come from the User statistics
            ],
            row![
                text("Mean Score:"),
                text("N/A") // This would come from the User statistics
            ],
            row![
                text("Episodes Watched:"),
                text("N/A") // This would come from the User statistics
            ],
        ]
        .spacing(5)
        .padding(10);

        let manga_stats = column![
            text("Manga Stats").size(18),
            row![
                text("Total Manga:"),
                text("N/A") // This would come from the User statistics
            ],
            row![
                text("Mean Score:"),
                text("N/A") // This would come from the User statistics
            ],
            row![
                text("Chapters Read:"),
                text("N/A") // This would come from the User statistics
            ],
        ]
        .spacing(5)
        .padding(10);

        container(
            column![text(&self.user.name).size(24), anime_stats, manga_stats]
                .spacing(20)
                .padding(20),
        )
        .width(Length::Fill)
        .into()
    }
}
