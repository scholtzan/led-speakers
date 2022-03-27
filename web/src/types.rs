use http::response;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

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
    pub settings: Option<Value>,
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

    /// Colors theme consists of
    pub colors: Vec<Color>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Color {
    /// Red
    pub r: u8,

    /// Green
    pub g: u8,

    /// Blue
    pub b: u8,
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
