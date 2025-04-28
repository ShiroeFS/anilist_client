mod api;
mod app;
mod data;
mod ui;
mod utils;

use app::App;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the app
    let app = App::new().await?;

    // Create and launch the UI
    let _ui_app = app.create_ui_app();

    println!("Starting AniList Desktop Client...");

    // In a real application, you'd launch the UI with iced
    // ui_app.launch()?;

    // For now, let's just do some test operations
    let client = app.get_api_client();

    // Example: Search for anime
    if let Ok(results) = client
        .search_anime("Shingeki no Kyojin".to_string(), Some(1), Some(5))
        .await
    {
        println!("Search results for 'Shingeki no Kyojin':");
        if let Some(page) = results.page {
            if let Some(media_list) = page.media {
                for media in media_list {
                    if let Some(media) = media {
                        if let Some(title) = media.title {
                            println!("- {} (ID: {})", title.romaji.unwrap_or_default(), media.id);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
