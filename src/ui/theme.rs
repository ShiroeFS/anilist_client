use iced::{Color, Theme};
use log::{debug, error};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Theme config that can be loaded from JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub name: String,
    pub colors: ThemeColors,
    pub fonts: ThemeFonts,
    pub spacing: ThemeSpacing,
    #[serde(rename = "borderRadius")]
    pub border_radius: ThemeBorderRadius,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub primary: String,
    pub secondary: String,
    pub background: String,
    #[serde(default = "default_surface_color")]
    pub surface: String,
    pub text: String,
    #[serde(default = "default_text_secondary_color")]
    pub text_secondary: String,
    pub error: String,
    pub success: String,
    pub warning: String,
    pub info: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeFonts {
    pub regular: String,
    pub bold: String,
    pub italic: String,
    pub size: FontSizes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSizes {
    pub small: u16,
    pub medium: u16,
    pub large: u16,
    pub xlarge: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSpacing {
    pub small: u16,
    pub medium: u16,
    pub large: u16,
    pub xlarge: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeBorderRadius {
    pub small: u16,
    pub medium: u16,
    pub large: u16,
    pub round: u16,
}

fn default_surface_color() -> String {
    "#FFFFFF".to_string()
}

fn default_text_secondary_color() -> String {
    "#4A4A4A".to_string()
}

// Custom application theme that extends the built-in Iced theme
pub struct AppTheme {
    iced_theme: Theme,
    config: Arc<ThemeConfig>,
}

impl AppTheme {
    pub fn new(theme_type: ThemeType) -> Self {
        let iced_theme = match theme_type {
            ThemeType::Light => Theme::Light,
            ThemeType::Dark => Theme::Dark,
            ThemeType::Custom(_) => {
                debug!("Loading custom theme");
                // In Iced 0.9, we don't have application::Style, so we'll use the Light theme as a base
                Theme::Light
            }
        };

        let config = match theme_type {
            ThemeType::Light => DEFAULT_LIGHT_THEME.clone(),
            ThemeType::Dark => DEFAULT_DARK_THEME.clone(),
            ThemeType::Custom(name) => match load_theme_config(&name) {
                Ok(config) => Arc::new(config),
                Err(e) => {
                    error!("Failed to load custom theme '{}': {}", name, e);
                    DEFAULT_LIGHT_THEME.clone()
                }
            },
        };

        Self { iced_theme, config }
    }

    pub fn get_color(&self, color_type: ColorType) -> Color {
        match color_type {
            ColorType::Primary => hex_to_color(&self.config.colors.primary),
            ColorType::Secondary => hex_to_color(&self.config.colors.secondary),
            ColorType::Background => hex_to_color(&self.config.colors.background),
            ColorType::Surface => hex_to_color(&self.config.colors.surface),
            ColorType::Text => hex_to_color(&self.config.colors.text),
            ColorType::TextSecondary => hex_to_color(&self.config.colors.text_secondary),
            ColorType::Error => hex_to_color(&self.config.colors.error),
            ColorType::Success => hex_to_color(&self.config.colors.success),
            ColorType::Warning => hex_to_color(&self.config.colors.warning),
            ColorType::Info => hex_to_color(&self.config.colors.info),
        }
    }

    pub fn get_spacing(&self, spacing_type: SpacingType) -> u16 {
        match spacing_type {
            SpacingType::Small => self.config.spacing.small,
            SpacingType::Medium => self.config.spacing.medium,
            SpacingType::Large => self.config.spacing.large,
            SpacingType::XLarge => self.config.spacing.xlarge,
        }
    }

    pub fn get_font_size(&self, size_type: FontSizeType) -> u16 {
        match size_type {
            FontSizeType::Small => self.config.fonts.size.small,
            FontSizeType::Medium => self.config.fonts.size.medium,
            FontSizeType::Large => self.config.fonts.size.large,
            FontSizeType::XLarge => self.config.fonts.size.xlarge,
        }
    }

    pub fn get_border_radius(&self, radius_type: BorderRadiusType) -> u16 {
        match radius_type {
            BorderRadiusType::Small => self.config.border_radius.small,
            BorderRadiusType::Medium => self.config.border_radius.medium,
            BorderRadiusType::Large => self.config.border_radius.large,
            BorderRadiusType::Round => self.config.border_radius.round,
        }
    }

    pub fn as_iced_theme(&self) -> Theme {
        self.iced_theme.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThemeType {
    Light,
    Dark,
    Custom(String),
}

#[derive(Debug, Clone, Copy)]
pub enum ColorType {
    Primary,
    Secondary,
    Background,
    Surface,
    Text,
    TextSecondary,
    Error,
    Success,
    Warning,
    Info,
}

#[derive(Debug, Clone, Copy)]
pub enum SpacingType {
    Small,
    Medium,
    Large,
    XLarge,
}

#[derive(Debug, Clone, Copy)]
pub enum FontSizeType {
    Small,
    Medium,
    Large,
    XLarge,
}

#[derive(Debug, Clone, Copy)]
pub enum BorderRadiusType {
    Small,
    Medium,
    Large,
    Round,
}

// Default light theme
static DEFAULT_LIGHT_THEME: Lazy<Arc<ThemeConfig>> = Lazy::new(|| {
    Arc::new(ThemeConfig {
        name: "Default Light".to_string(),
        colors: ThemeColors {
            primary: "#7957D5".to_string(),
            secondary: "#2B2D42".to_string(),
            background: "#FFFFFF".to_string(),
            surface: "#F7F7F7".to_string(),
            text: "#2B2D42".to_string(),
            text_secondary: "#4A4A4A".to_string(),
            error: "#E63946".to_string(),
            success: "#2A9D8F".to_string(),
            warning: "#F4A261".to_string(),
            info: "#457B9D".to_string(),
        },
        fonts: ThemeFonts {
            regular: "Roboto-Regular".to_string(),
            bold: "Roboto-Bold".to_string(),
            italic: "Roboto-Italic".to_string(),
            size: FontSizes {
                small: 12,
                medium: 16,
                large: 20,
                xlarge: 28,
            },
        },
        spacing: ThemeSpacing {
            small: 5,
            medium: 10,
            large: 20,
            xlarge: 40,
        },
        border_radius: ThemeBorderRadius {
            small: 3,
            medium: 5,
            large: 10,
            round: 9999,
        },
    })
});

// Default dark theme
static DEFAULT_DARK_THEME: Lazy<Arc<ThemeConfig>> = Lazy::new(|| {
    Arc::new(ThemeConfig {
        name: "Default Dark".to_string(),
        colors: ThemeColors {
            primary: "#A485FF".to_string(),
            secondary: "#6C63FF".to_string(),
            background: "#1A1B26".to_string(),
            surface: "#24283B".to_string(),
            text: "#C0CAF5".to_string(),
            text_secondary: "#565F89".to_string(),
            error: "#F7768E".to_string(),
            success: "#9ECE6A".to_string(),
            warning: "#E0AF68".to_string(),
            info: "#7AA2F7".to_string(),
        },
        fonts: ThemeFonts {
            regular: "Roboto-Regular".to_string(),
            bold: "Roboto-Bold".to_string(),
            italic: "Roboto-Italic".to_string(),
            size: FontSizes {
                small: 12,
                medium: 16,
                large: 20,
                xlarge: 28,
            },
        },
        spacing: ThemeSpacing {
            small: 5,
            medium: 10,
            large: 20,
            xlarge: 40,
        },
        border_radius: ThemeBorderRadius {
            small: 3,
            medium: 5,
            large: 10,
            round: 9999,
        },
    })
});

// Helper function to convert hex color strings to iced::Color
fn hex_to_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');

    if hex.len() != 6 {
        error!("Invalid hex color: {}", hex);
        return Color::BLACK;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

    Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

// Load theme config from JSON file
fn load_theme_config(name: &str) -> Result<ThemeConfig, Box<dyn std::error::Error>> {
    let file_path = format!("resources/themes/{}.json", name);

    // First try to load from the file system
    match std::fs::read_to_string(&file_path) {
        Ok(content) => {
            let config: ThemeConfig = serde_json::from_str(&content)?;
            return Ok(config);
        }
        Err(e) => {
            debug!("Failed to load theme from file system: {}", e);
            // If file system fails, try to load from embedded resource
            let resource_path = format!("themes/{}.json", name);
            if let Some(content) = get_embedded_theme(&resource_path) {
                let config: ThemeConfig = serde_json::from_str(&content)?;
                return Ok(config);
            }
        }
    }

    Err(format!("Theme '{}' not found", name).into())
}

// Get embedded theme resources
fn get_embedded_theme(path: &str) -> Option<String> {
    match path {
        "themes/default.json" => Some(DEFAULT_THEME_JSON.to_string()),
        "themes/dark.json" => Some(DARK_THEME_JSON.to_string()),
        _ => None,
    }
}

// Apply the theme to the application
pub fn apply_theme(theme_type: ThemeType) -> AppTheme {
    AppTheme::new(theme_type)
}

// Get the current theme based on system preferences or settings
pub fn get_current_theme(theme_setting: &str) -> ThemeType {
    match theme_setting {
        "light" => ThemeType::Light,
        "dark" => ThemeType::Dark,
        "system" => {
            // Check system preferences
            #[cfg(target_os = "windows")]
            {
                #[cfg(feature = "winreg")]
                {
                    use winreg::enums::*;
                    use winreg::RegKey;

                    let hkcu = match RegKey::predef(HKEY_CURRENT_USER) {
                        Ok(key) => key,
                        Err(_) => return ThemeType::Light,
                    };

                    if let Ok(personalize) = hkcu.open_subkey(
                        r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
                    ) {
                        if let Ok(apps_use_light_theme) =
                            personalize.get_value::<u32, _>("AppsUseLightTheme")
                        {
                            if apps_use_light_theme == 0 {
                                return ThemeType::Dark;
                            }
                        }
                    }
                }
            }

            #[cfg(target_os = "macos")]
            {
                use std::process::Command;

                if let Ok(output) = Command::new("defaults")
                    .args(&["read", "-g", "AppleInterfaceStyle"])
                    .output()
                {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        if stdout.trim() == "Dark" {
                            return ThemeType::Dark;
                        }
                    }
                }
            }

            // Default to light
            ThemeType::Light
        }
        custom => ThemeType::Custom(custom.to_string()),
    }
}

// Initialize the default theme
pub fn default_theme() -> AppTheme {
    apply_theme(ThemeType::Light)
}

// Embedded theme JSON strings
static DEFAULT_THEME_JSON: &str = r##"{
  "name": "Default",
  "colors": {
    "primary": "#7957D5",
    "secondary": "#2B2D42",
    "background": "#FFFFFF",
    "text": "#2B2D42",
    "error": "#E63946",
    "success": "#2A9D8F",
    "warning": "#F4A261",
    "info": "#457B9D"
  },
  "fonts": {
    "regular": "Roboto-Regular",
    "bold": "Roboto-Bold",
    "italic": "Roboto-Italic",
    "size": {
      "small": 12,
      "medium": 16,
      "large": 20,
      "xlarge": 28
    }
  },
  "spacing": {
    "small": 5,
    "medium": 10,
    "large": 20,
    "xlarge": 40
  },
  "borderRadius": {
    "small": 3,
    "medium": 5,
    "large": 10,
    "round": 9999
  }
}"##;

static DARK_THEME_JSON: &str = r##"{
  "name": "Dark",
  "colors": {
    "primary": "#A485FF",
    "secondary": "#6C63FF",
    "background": "#1A1B26",
    "surface": "#24283B",
    "text": "#C0CAF5",
    "textSecondary": "#565F89",
    "error": "#F7768E",
    "success": "#9ECE6A",
    "warning": "#E0AF68",
    "info": "#7AA2F7"
  },
  "fonts": {
    "regular": "Roboto-Regular",
    "bold": "Roboto-Bold",
    "italic": "Roboto-Italic",
    "size": {
      "small": 12,
      "medium": 16,
      "large": 20,
      "xlarge": 28
    }
  },
  "spacing": {
    "small": 5,
    "medium": 10,
    "large": 20,
    "xlarge": 40
  },
  "borderRadius": {
    "small": 3,
    "medium": 5,
    "large": 10,
    "round": 9999
  }
}"##;
