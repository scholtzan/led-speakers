use crate::theme::Color;
use crate::viz::PixelViz;
use crate::viz::Viz;
use chrono::prelude::*;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
/// Visualization Config.
pub struct FadingBeatVizConfig {
    /// Screen friendly name of visualization.
    pub pretty_name: String,

    /// Speed for fading between different colors
    pub fade_duration: i64,

    /// Maximum time a color is shown without fading to the next one.
    pub fade_threshold: i64,

    /// Size of buffer keeping track of past frequency magnitudes.
    pub frequency_magnitude_buffer_size: i64,
}

impl FadingBeatVizConfig {
    /// Convert settings in map of strings to visualization config.
    pub fn to_map(&self) -> HashMap<String, String> {
        let mut settings = HashMap::new();
        settings.insert("fade_duration".to_string(), self.fade_duration.to_string());
        settings.insert(
            "fade_threshold".to_string(),
            self.fade_threshold.to_string(),
        );
        settings.insert(
            "frequency_magnitude_buffer_size".to_string(),
            self.frequency_magnitude_buffer_size.to_string(),
        );
        settings
    }

    /// Create visualization config from map of strings.
    pub fn from_map(name: String, settings: HashMap<String, String>) -> Self {
        Self {
            pretty_name: name,
            fade_duration: settings
                .get(&"fade_duration".to_string())
                .unwrap_or(&"0".to_string())
                .parse::<i64>()
                .unwrap(),
            fade_threshold: settings
                .get(&"fade_threshold".to_string())
                .unwrap_or(&"0".to_string())
                .parse::<i64>()
                .unwrap(),
            frequency_magnitude_buffer_size: settings
                .get(&"frequency_magnitude_buffer_size".to_string())
                .unwrap_or(&"0".to_string())
                .parse::<i64>()
                .unwrap(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
/// Visualization showing all pixels in the same color that fades
/// to the next theme color when the dominant frequency changes.
pub struct FadingBeatViz {
    /// Visualization config
    pub config: FadingBeatVizConfig,

    /// Total number of pixels.
    total_pixels: usize,

    /// Elapsed time from when pixels faded to a different color.
    elapsed_time: DateTime<Utc>,

    /// Current color displayed.
    color_index: usize,

    /// Past dominant frequencies.
    dominant_frequencies: Vec<usize>,

    /// Pixels are currently in the process of fading to a different color.
    is_fading: bool,
}

#[typetag::serde]
impl Viz for FadingBeatViz {
    fn get_name(&self) -> &str {
        "fading_beat_viz"
    }

    fn get_pretty_name(&self) -> &str {
        &self.config.pretty_name
    }

    fn update(&mut self, input: &Vec<f32>, colors: &Vec<Color>) -> Vec<PixelViz> {
        // determine color and brightness of pixels based on frequency magnitudes
        let total_bands = input.len();
        let max_magnitude = 100.0 * total_bands as f32;
        let magnitude: f32 = input.iter().sum();
        let mut viz = PixelViz::default();
        viz.brightness = magnitude / max_magnitude;
        viz.color_index = self.color_index;

        let next_color_index = (self.color_index + 1) % colors.len();
        let now = Utc::now();
        let elapsed = (now - self.elapsed_time).num_seconds();

        // determine current dominant frequency
        let mut dominant_frequency = 0;
        let mut max_magnitude = 0.0;
        for (i, mag) in input.into_iter().enumerate() {
            if mag > &max_magnitude {
                max_magnitude = *mag;
                dominant_frequency = i;
            }
        }

        // Update past dominant frequency buffer and add current
        let prev_dominant_frequency = Self::mode(&self.dominant_frequencies);
        let mut prev_freq = self.dominant_frequencies[0];
        self.dominant_frequencies[0] = dominant_frequency;
        for n in 0..(self.dominant_frequencies.len() - 1) {
            let tmp = self.dominant_frequencies[n + 1];
            self.dominant_frequencies[n + 1] = prev_freq;
            prev_freq = tmp;
        }

        let current_dominant_frequency = Self::mode(&self.dominant_frequencies);

        // check if pixels are currently fading to a different color
        if elapsed > self.config.fade_threshold && !self.is_fading {
            if current_dominant_frequency != prev_dominant_frequency {
                self.is_fading = true;
                self.elapsed_time = now;
            }
        }

        if self.is_fading {
            // fade pixel colors
            let current_color = colors[self.color_index];
            let next_color = colors[next_color_index];
            let elapsed_perc: f32 = elapsed as f32 / self.config.fade_duration as f32;
            viz.red_mul =
                (((next_color.r as f32 / current_color.r as f32) - 1.0) * elapsed_perc) + 1.0;
            viz.green_mul =
                (((next_color.g as f32 / current_color.g as f32) - 1.0) * elapsed_perc) + 1.0;
            viz.blue_mul =
                (((next_color.b as f32 / current_color.b as f32) - 1.0) * elapsed_perc) + 1.0;

            if elapsed_perc >= 1.0 {
                // stop fading if target color is reached
                self.color_index = next_color_index;
                self.elapsed_time = now;
                self.is_fading = false;
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
            FadingBeatVizConfig::from_map(self.get_pretty_name().to_string(), settings);
        self.config = new_settings;
        self.dominant_frequencies = vec![0; self.config.frequency_magnitude_buffer_size as usize];
        self.is_fading = false;
    }
}

unsafe impl Send for FadingBeatViz {}
unsafe impl Sync for FadingBeatViz {}

impl FadingBeatViz {
    pub fn new(config: FadingBeatVizConfig) -> Self {
        FadingBeatViz {
            config: config.clone(),
            total_pixels: 0,
            elapsed_time: Utc::now(),
            color_index: 0,
            dominant_frequencies: vec![0; config.clone().frequency_magnitude_buffer_size as usize],
            is_fading: false,
        }
    }

    /// Determines the mode from a list of values.
    /// https://gist.github.com/ayoisaiah/185fec1ca98ce44fca1308753182ff2b
    fn mode(numbers: &Vec<usize>) -> usize {
        let mut map = HashMap::new();
        for integer in numbers {
            let count = map.entry(integer).or_insert(0);
            *count += 1;
        }

        let max_value = map.values().cloned().max().unwrap_or(0);

        map.into_iter()
            .filter(|&(_, v)| v == max_value)
            .map(|(&k, _)| k)
            .collect::<Vec<usize>>()[0]
    }
}
