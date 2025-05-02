# Resources Directory

This directory contains static assets used by the AniList client.

## Structure

- `icons/`: Application icons and other UI elements
- `themes/`: Theme definitions for the application
- `fonts/`: Custom fonts used in the UI
- `locales/`: Localization files for different languages

## Usage

Assets in this directory are embedded in the application binary at compile time using the `include_bytes!` macro.
