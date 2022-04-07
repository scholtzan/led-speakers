use crate::theme::Color;
use crate::viz::PixelViz;
use crate::viz::Viz;
use chrono::prelude::*;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct FadingBeatVizConfig {
    pub pretty_name: String,
    pub fade_duration: i64,
    pub fade_threshold: i64,
    pub frequency_magnitude_buffer_size: i64,
}

impl FadingBeatVizConfig {
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
pub struct FadingBeatViz {
    pub config: FadingBeatVizConfig,
    total_pixels: usize,
    elapsed_time: DateTime<Utc>,
    color_index: usize,
    dominant_frequencies: Vec<usize>,
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
        let total_bands = input.len();
        let max_magnitude = 100.0 * total_bands as f32;
        let magnitude: f32 = input.iter().sum();
        let mut viz = PixelViz::default();
        viz.brightness = magnitude / max_magnitude;
        viz.color_index = self.color_index;

        let next_color_index = (self.color_index + 1) % colors.len();
        let now = Utc::now();
        let elapsed = (now - self.elapsed_time).num_seconds();

        let mut dominant_frequency = 0;
        let mut max_magnitude = 0.0;
        for (i, mag) in input.into_iter().enumerate() {
            if mag > &max_magnitude {
                max_magnitude = *mag;
                dominant_frequency = i;
            }
        }

        let prev_dominant_frequency = Self::mode(&self.dominant_frequencies);

        let mut prev_freq = self.dominant_frequencies[0];
        self.dominant_frequencies[0] = dominant_frequency;
        for n in 0..(self.dominant_frequencies.len() - 1) {
            let tmp = self.dominant_frequencies[n + 1];
            self.dominant_frequencies[n + 1] = prev_freq;
            prev_freq = tmp;
        }

        let current_dominant_frequency = Self::mode(&self.dominant_frequencies);

        if elapsed > self.config.fade_threshold && !self.is_fading {
            if current_dominant_frequency != prev_dominant_frequency {
                self.is_fading = true;
                self.elapsed_time = now;
            }
        }

        if self.is_fading {
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

    // https://gist.github.com/ayoisaiah/185fec1ca98ce44fca1308753182ff2b
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
