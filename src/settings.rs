use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::led::Led;
use crate::theme::Theme;
use crate::viz::{
    BlendViz, BlendVizConfig, CenterViz, CenterVizConfig, FadingBeatViz, FadingBeatVizConfig,
    RotatingViz, RotatingVizConfig, SolidBeatViz, SolidBeatVizConfig, SolidViz, SolidVizConfig,
    SparkleViz, SparkleVizConfig, Viz,
};
use std::collections::HashMap;

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
        Led::new(self.total_leds, &self.spi)
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

#[derive(Serialize, Deserialize, Clone)]
pub struct TransformerSettings {
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

impl TransformerSettings {
    /// Transform the settings into a map of string values.
    pub fn to_map(&self) -> HashMap<String, String> {
        let mut settings = HashMap::new();
        settings.insert("sink".to_string(), self.sink.clone());
        settings.insert("fft_len".to_string(), self.fft_len.to_string());
        settings.insert("total_bands".to_string(), self.total_bands.to_string());
        settings.insert("lower_cutoff".to_string(), self.lower_cutoff.to_string());
        settings.insert("upper_cutoff".to_string(), self.upper_cutoff.to_string());
        settings.insert("monstercat".to_string(), self.monstercat.to_string());
        settings.insert("decay".to_string(), self.decay.to_string());
        settings.insert("buffer_size".to_string(), self.buffer_size.to_string());
        settings
    }

    /// Create transformer settings from map of string values.
    pub fn from_map(settings: HashMap<String, String>) -> Self {
        TransformerSettings {
            sink: settings
                .get(&"sink".to_string())
                .unwrap_or(&"".to_string())
                .to_string(),
            fft_len: settings
                .get(&"fft_len".to_string())
                .unwrap_or(&"0".to_string())
                .parse::<usize>()
                .unwrap(),
            total_bands: settings
                .get(&"total_bands".to_string())
                .unwrap_or(&"0".to_string())
                .parse::<usize>()
                .unwrap(),
            lower_cutoff: settings
                .get(&"lower_cutoff".to_string())
                .unwrap_or(&"0".to_string())
                .parse::<f32>()
                .unwrap(),
            upper_cutoff: settings
                .get(&"upper_cutoff".to_string())
                .unwrap_or(&"0".to_string())
                .parse::<f32>()
                .unwrap(),
            monstercat: settings
                .get(&"monstercat".to_string())
                .unwrap_or(&"0".to_string())
                .parse::<f32>()
                .unwrap(),
            decay: settings
                .get(&"decay".to_string())
                .unwrap_or(&"0".to_string())
                .parse::<f32>()
                .unwrap(),
            buffer_size: settings
                .get(&"buffer_size".to_string())
                .unwrap_or(&"0".to_string())
                .parse::<usize>()
                .unwrap(),
        }
    }
}

#[derive(Serialize, Deserialize)]
/// Representation of the config.json file.
pub struct Settings {
    #[serde(deserialize_with = "parse_visualizations")]
    /// Available visualizations
    pub visualizations: Vec<Box<dyn Viz>>,

    /// Output settings
    pub output: OutputSettings,

    /// Available themes
    pub themes: Vec<Theme>,

    pub transformer: TransformerSettings,

    /// Server host IP
    pub server_host: String,

    /// Server host port
    pub server_port: String,
}

impl Settings {
    /// Update the transformer related settings.
    pub fn apply_transformer_settings(&mut self, transformer_settings: TransformerSettings) {
        self.transformer = transformer_settings;
    }
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
                "blend_viz" => {
                    let viz_config: BlendVizConfig = serde_json::from_value(args.clone()).unwrap();
                    Ok(Box::new(BlendViz::new(viz_config)))
                }
                "solid_beat_viz" => {
                    let viz_config: SolidBeatVizConfig =
                        serde_json::from_value(args.clone()).unwrap();
                    Ok(Box::new(SolidBeatViz::new(viz_config)))
                }
                "fading_beat_viz" => {
                    let viz_config: FadingBeatVizConfig =
                        serde_json::from_value(args.clone()).unwrap();
                    Ok(Box::new(FadingBeatViz::new(viz_config)))
                }
                _ => Err(D::Error::custom(format!("Unknown {:?}", name.as_str()))),
            };
            viz
        })
        .collect::<Result<Vec<Box<dyn Viz>>, D::Error>>()
}
