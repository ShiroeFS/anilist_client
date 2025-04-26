mod app;
mod components;
mod screens;
mod theme;

pub use app::AniListApp;

// Re-export useful components
pub use components::anime_card::AnimeCard;
pub use components::media_list::MediaList;
pub use components::user_stats::UserStats;
