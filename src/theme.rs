use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8
}

impl Color {
    pub fn from_vec(v: &Vec<u8>) -> Self {
        Color {
            r: v[0],
            g: v[1],
            b: v[2],
            a: v[3]
        }
    }
}


#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Theme {
    pub name: String,
    #[serde(deserialize_with="parse_colors")]
    pub colors: Vec<Color>
}

fn parse_colors<'de, D>(d: D) -> Result<Vec<Color>, D::Error> where D: Deserializer<'de> {
    let colors = Vec::deserialize(d).unwrap();
    Ok(colors.iter().map(|c| Color::from_vec(&c)).collect::<Vec<_>>())
}
