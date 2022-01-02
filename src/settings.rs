use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::led::Led;
use crate::theme::Theme;
use crate::viz::{
    CenterViz, CenterVizConfig, RotatingViz, RotatingVizConfig, SolidViz, SolidVizConfig,
    SparkleViz, SparkleVizConfig, Viz,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct Output {
    pub clock_speed_hz: u32,
    pub spi: String,
    pub total_leds: i32,
}

impl Output {
    pub fn to_led(&self) -> Led {
        Led::new(self.total_leds, self.spi.clone(), self.clock_speed_hz)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OutputSettings {
    pub left: Output,
    pub right: Output,
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    #[serde(deserialize_with = "parse_visualizations")]
    pub vizualizations: Vec<Box<dyn Viz>>,
    pub output: OutputSettings,
    pub themes: Vec<Theme>,
    pub sink: String,
    pub bins: usize,
    pub total_bands: usize,
    pub lower_cutoff: f32,
    pub upper_cutoff: f32,
    pub monstercat: f32,
    pub decay: f32,
    pub buffer_size: usize,
}

fn parse_visualizations<'de, D>(d: D) -> Result<Vec<Box<dyn Viz>>, D::Error>
where
    D: Deserializer<'de>,
{
    let parsed = Value::deserialize(d)?;

    parsed
        .as_object()
        .unwrap()
        .into_iter()
        .map(|(name, args)| {
            let viz: Result<Box<dyn Viz>, D::Error> = match name.as_str() {
                // todo: move logic to viz?
                "rotating_viz" => {
                    let viz_config: RotatingVizConfig =
                        serde_json::from_value(args.clone()).unwrap();
                    Ok(Box::new(RotatingViz::new(viz_config)))
                }
                "sparkle_viz" => {
                    let viz_config: SparkleVizConfig =
                        serde_json::from_value(args.clone()).unwrap();
                    Ok(Box::new(SparkleViz::new(viz_config)))
                }
                "solid_viz" => {
                    let viz_config: SolidVizConfig = serde_json::from_value(args.clone()).unwrap();
                    Ok(Box::new(SolidViz::new(viz_config)))
                }
                "center_viz" => {
                    let viz_config: CenterVizConfig = serde_json::from_value(args.clone()).unwrap();
                    Ok(Box::new(CenterViz::new(viz_config)))
                }
                _ => Err(D::Error::custom(format!("Unknown {:?}", name.as_str()))),
            };
            viz
        })
        .collect::<Result<Vec<Box<dyn Viz>>, D::Error>>()
}
