use serde::{Deserialize, Deserializer, Serialize};
use serde::de::Error;
use serde_json::Value;

use crate::viz::{Viz, RotatingViz, SparkleViz};
use crate::theme::Theme;



#[derive(Serialize, Deserialize)]
pub struct Settings {
    #[serde(deserialize_with="parse_visualizations")]
    pub vizualizations: Vec<Box<dyn Viz>>,
    pub themes: Vec<Theme>,
    pub sink: String,
    pub bins: usize,
    pub total_bands: usize,
    pub lower_cutoff: f32,
    pub upper_cutoff: f32,
    pub monstercat: f32,
    pub decay: f32,
}


fn parse_visualizations<'de, D>(d: D) -> Result<Vec<Box<dyn Viz>>, D::Error> where D: Deserializer<'de> {
    let parsed = Value::deserialize(d)?;

    parsed.as_object().unwrap().into_iter().map(|(name, args)| {
        let viz: Result<Box<dyn Viz>, D::Error> = match name.as_str() {
            "rotating_viz" => {
                let viz: RotatingViz = serde_json::from_value(args.clone()).unwrap();
                Ok(Box::new(viz))
            },
            "sparkle_viz" => {
                let viz: SparkleViz = serde_json::from_value(args.clone()).unwrap();
                Ok(Box::new(viz))
            },
            _ => {
                Err(D::Error::custom(format!("Unknown {:?}", name.as_str())))
            }
        };
        viz
    }).collect::<Result<Vec<Box<dyn Viz>>, D::Error>>()
}
