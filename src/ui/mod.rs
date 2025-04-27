mod components;
mod screens;
mod theme;

// Re-export useful components
pub use components::anime_card::AnimeCard;
pub use components::media_list::MediaList;
pub use components::user_stats::UserStats;

// Re-export screens
pub use screens::details::details_screen;
pub use screens::home::home_screen;
pub use screens::profile::profile_screen;
pub use screens::search::search_screen;
pub use screens::settings::settings_screen;
