use crate::theme::Color;
use crate::viz::PixelViz;
use crate::viz::Viz;
use chrono::prelude::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
/// Visualization Config.
pub struct SolidBeatVizConfig {
    /// Screen friendly name of visualization.
    pub pretty_name: String,

    /// Whether colors should be faded to a different color randomly.
    pub fade_colors: bool,

    /// Speed of fading the pixels to a different color.
    pub fade_duration: i64,
}

impl SolidBeatVizConfig {
    /// Convert settings in map of strings to visualization config.
    pub fn to_map(&self) -> HashMap<String, String> {
        let mut settings = HashMap::new();
        settings.insert("fade_colors".to_string(), self.fade_colors.to_string());
        settings.insert("fade_duration".to_string(), self.fade_duration.to_string());
        settings
    }

    /// Create visualization config from map of strings.
    pub fn from_map(name: String, settings: HashMap<String, String>) -> Self {
        Self {
            pretty_name: name,
            fade_colors: settings
                .get(&"fade_colors".to_string())
                .unwrap_or(&"true".to_string())
                .parse::<bool>()
                .unwrap(),
            fade_duration: settings
                .get(&"fade_duration".to_string())
                .unwrap_or(&"0".to_string())
                .parse::<i64>()
                .unwrap(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
/// Visualizations that shows all pixels in the same color and changes
/// their brightness based on the current beat.
/// Optionally, fades pixels to a different color.
pub struct SolidBeatViz {
    /// Visualization config.
    pub config: SolidBeatVizConfig,

    /// Total number of pixels.
    total_pixels: usize,

    /// Time since the last fade happened.
    elapsed_time: DateTime<Utc>,

    /// Current color pixels are displayed in.
    color_index: usize,
}

#[typetag::serde]
impl Viz for SolidBeatViz {
    fn get_name(&self) -> &str {
        "solid_beat_viz"
    }

    fn get_pretty_name(&self) -> &str {
        &self.config.pretty_name
    }

    fn update(&mut self, input: &Vec<f32>, colors: &Vec<Color>) -> Vec<PixelViz> {
        // determine the total frequency magnitude and compute brightness based on it
        let total_bands = input.len();
        let max_magnitude = 100.0 * total_bands as f32;
        let magnitude: f32 = input.iter().sum();
        let mut viz = PixelViz::default();
        viz.brightness = magnitude / max_magnitude;
        viz.color_index = self.color_index;

        if self.config.fade_colors {
            // if colors are configured to be fading then wait until fade threshold is reached
            let next_color_index = (self.color_index + 1) % colors.len();
            let now = Utc::now();
            let elapsed = (now - self.elapsed_time).num_seconds();

            if elapsed > self.config.fade_duration {
                self.color_index = next_color_index;
                self.elapsed_time = now;
            } else {
                // fade
                let current_color = colors[self.color_index % colors.len()];
                let next_color = colors[next_color_index % colors.len()];
                let elapsed_perc: f32 = elapsed as f32 / self.config.fade_duration as f32;
                viz.red_mul =
                    (((next_color.r as f32 / current_color.r as f32) - 1.0) * elapsed_perc) + 1.0;
                viz.green_mul =
                    (((next_color.g as f32 / current_color.g as f32) - 1.0) * elapsed_perc) + 1.0;
                viz.blue_mul =
                    (((next_color.b as f32 / current_color.b as f32) - 1.0) * elapsed_perc) + 1.0;
            }
        }

        vec![viz; self.total_pixels]
    }

    fn set_total_pixels(&mut self, pixels: usize) {
        self.total_pixels = pixels;
    }

    fn get_settings(&self) -> HashMap<String, String> {
        self.config.to_map()
    }

    fn update_settings(&mut self, settings: HashMap<String, String>) {
        let new_settings =
            SolidBeatVizConfig::from_map(self.get_pretty_name().to_string(), settings);
        self.config = new_settings;
    }
}

unsafe impl Send for SolidBeatViz {}
unsafe impl Sync for SolidBeatViz {}

impl SolidBeatViz {
    pub fn new(config: SolidBeatVizConfig) -> Self {
        SolidBeatViz {
            config,
            total_pixels: 0,
            elapsed_time: Utc::now(),
            color_index: 0,
        }
    }
}
