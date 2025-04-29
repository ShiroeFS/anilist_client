# AniList Desktop Client

A desktop client for AniList built with Rust and Iced.

## Features

- Browse and search anime from AniList
- View detailed information about anime
- Track your anime watching progress
- OAuth2 authentication with AniList
- Offline mode support with local caching
- Cross-platform (Windows, macOS, Linux)

## Prerequisites

- Rust (1.65+)
- Cargo
- AniList API credentials
  - You need to create a client on the [AniList Developer page](https://anilist.co/settings/developer)

## Setup

1. Clone the repository
2. Configure your AniList API credentials in `config.json` (will be created on first run):

```json
{
  "auth_config": {
    "client_id": "your-client-id",
    "client_secret": "your-client-secret",
    "redirect_uri": "http://localhost:8080/callback"
  },
  "theme": "default",
  "language": "en",
  "offline_mode": false
}
```

3. Build and run the application:

```bash
cargo build --release
cargo run --release
```

## Command-Line Options

The application supports the following command-line options:

- `--test-auth`: Test the authentication flow
- `--test-api`: Test the API connection
- `--help`: Show help message

## Project Structure

```
anilist_client/
├── Cargo.toml
├── schema.graphql          # GraphQL schema for AniList API
├── src/
│   ├── main.rs             # Application entry point
│   ├── app.rs              # Main application state and logic
│   ├── api/                # API interaction layer
│   │   ├── mod.rs
│   │   ├── client.rs       # AniList API client
│   │   ├── auth.rs         # Authentication handling with OAuth2
│   │   ├── queries/        # GraphQL queries
│   │   └── models/         # Data models for API responses
│   ├── ui/                 # User interface layer
│   │   ├── mod.rs
│   │   ├── app.rs          # Main UI application
│   │   ├── screens/        # Different application screens
│   │   │   ├── home.rs
│   │   │   ├── search.rs
│   │   │   ├── details.rs
│   │   │   ├── profile.rs
│   │   │   └── settings.rs
│   │   ├── components/     # Reusable UI components
│   │   │   ├── anime_card.rs
│   │   │   ├── media_list.rs
│   │   │   ├── user_stats.rs
│   │   │   └── auth.rs
│   │   └── theme.rs        # UI theming and styling
│   ├── data/               # Local data storage
│   │   ├── mod.rs
│   │   ├── database.rs     # Database interactions (SQLite)
│   │   ├── cache.rs        # In-memory caching
│   │   └── models/         # Local data models
│   └── utils/              # Utility functions
│       ├── mod.rs
│       ├── config.rs       # Configuration handling
│       └── error.rs        # Error types and handling
└── build.rs               # Build script
```

## Authentication

The application uses OAuth2 to authenticate with AniList. Upon first login, you'll be redirected to AniList in your browser to authorize the application. After authorizing, you'll be redirected back to the application, which will save your credentials for future use.

## Local Storage

The application uses SQLite to cache anime data and your lists locally, allowing it to work in offline mode when needed. The database is stored in your system's appropriate data directory.
