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
/// Settings for a single output
pub struct Output {
    /// SPI for output
    pub spi: String,

    /// Total available LEDs for output
    pub total_leds: i32,
}

impl Output {
    /// Converts output settings to an `Led` instance.
    pub fn to_led(&self) -> Led {
        Led::new(self.total_leds, self.spi.clone())
    }
}

#[derive(Serialize, Deserialize, Clone)]
/// Settings for left and right outputs.
pub struct OutputSettings {
    /// Left output settings
    pub left: Output,

    /// Right output settings
    pub right: Output,
}

#[derive(Serialize, Deserialize)]
/// Representation of the config.json file.
pub struct Settings {
    #[serde(deserialize_with = "parse_visualizations")]
    /// Available visualizations
    pub vizualizations: Vec<Box<dyn Viz>>,

    /// Output settings
    pub output: OutputSettings,

    /// Available themes
    pub themes: Vec<Theme>,

    /// Audio sink name
    pub sink: String,

    /// Number of FFT output buckets
    pub fft_len: usize,

    /// Number of total audio bands
    pub total_bands: usize,

    /// Lower cutoff frequency
    pub lower_cutoff: f32,

    /// Upper cutoff frequency
    pub upper_cutoff: f32,

    /// Monstercat smoothing factor
    pub monstercat: f32,

    /// Decay factor
    pub decay: f32,

    /// Audio buffer size
    pub buffer_size: usize,
}

/// Creates a new visualization from the settings.
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
