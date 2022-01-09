use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
/// Represents the RGB and brightness values of a color.
pub struct Color {
    /// Red
    pub r: u8,

    /// Green
    pub g: u8,

    /// Blue
    pub b: u8,

    /// Brightness
    pub a: u8,
}

impl Color {
    /// Creates a new `Color` instance from a vector of color values.
    ///
    /// # Examples
    /// ```
    /// let color = Color::from_vec(vec![255, 0, 255, 255]);
    /// ```
    pub fn from_vec(v: &Vec<u8>) -> Self {
        Color {
            r: v[0],
            g: v[1],
            b: v[2],
            a: v[3],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
/// A set of colors representing a theme.
pub struct Theme {
    /// Theme identifier
    pub name: String,

    #[serde(deserialize_with = "parse_colors")]
    /// Set of colors theme consists of
    pub colors: Vec<Color>,
}

/// Custom color parsing from JSON file.
fn parse_colors<'de, D>(d: D) -> Result<Vec<Color>, D::Error>
where
    D: Deserializer<'de>,
{
    let colors = Vec::deserialize(d).unwrap();
    Ok(colors
        .iter()
        .map(|c| Color::from_vec(&c))
        .collect::<Vec<_>>())
}
