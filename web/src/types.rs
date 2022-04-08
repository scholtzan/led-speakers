use http::response;
use serde::{Deserialize, Serialize, Serializer};

use std::collections::HashMap;

/// Error types that occur after making a request to the server.
pub enum Error {
    FetchError(String, response::Parts),
    Misc(String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Visualizations {
    /// Unique identifier of visualization currently in use
    pub current: String,

    /// All available visualizations
    pub visualizations: Vec<Visualization>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Visualization {
    /// Unique visualization identifier
    pub identifier: String,

    /// Display friendly name of visualization
    pub pretty_name: String,

    /// Available visualization settings
    pub settings: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Themes {
    /// Unique identifier of theme currently in use
    pub current: String,

    /// All available themes
    pub themes: Vec<Theme>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Theme {
    /// Unique theme identifier
    pub name: String,

    #[serde(serialize_with = "serialize_colors")]
    /// Colors theme consists of
    pub colors: Vec<Color>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Color {
    /// Red
    pub r: u8,

    /// Green
    pub g: u8,

    /// Blue
    pub b: u8,
}

impl Color {
    /// Create color from hex string.
    pub fn from_hex(color: &str) -> Self {
        Color {
            r: u8::from_str_radix(&color[1..3], 16).unwrap(),
            g: u8::from_str_radix(&color[3..5], 16).unwrap(),
            b: u8::from_str_radix(&color[5..7], 16).unwrap(),
        }
    }

    /// Convert color to hex string.
    pub fn to_hex(&self) -> String {
        format!(
            "#{:02X}{:02X}{:02X}",
            self.r as f32 as u8, self.g as f32 as u8, self.b as f32 as u8
        )
    }
}

/// Custom color serialization.
fn serialize_colors<S>(colors: &Vec<Color>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let color_vec = colors
        .iter()
        .map(|c| vec![c.r, c.g, c.b])
        .collect::<Vec<_>>();
    color_vec.serialize(s)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChangeVisualization {
    /// Identifier of visualization LED speakers should change to
    pub visualization: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChangeTheme {
    /// Identifier of theme LED speakers should change to
    pub theme: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Status {
    /// Whether the viz processing is stopped
    pub is_stopped: bool,
}
