use crate::theme::Color;
use crate::viz::PixelViz;
use crate::viz::Viz;
use chrono::prelude::*;

use std::time::Duration;

use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SolidBeatVizConfig {
    pub pretty_name: String,
    pub fade_colors: bool,
    pub fade_duration: i64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SolidBeatViz {
    pub config: SolidBeatVizConfig,
    total_pixels: usize,
    elapsed_time: DateTime<Utc>,
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
        let total_bands = input.len();
        let max_magnitude = 100.0 * total_bands as f32;
        let magnitude: f32 = input.iter().sum();
        let mut viz = PixelViz::default();
        viz.brightness = magnitude / max_magnitude;
        viz.color_index = self.color_index;

        if self.config.fade_colors {
            let next_color_index = (self.color_index + 1) % colors.len();
            let now = Utc::now();
            let elapsed = (now - self.elapsed_time).num_seconds();

            if elapsed > self.config.fade_duration {
                self.color_index = next_color_index;
                self.elapsed_time = now;
            } else {
                let current_color = colors[self.color_index];
                let next_color = colors[next_color_index];
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
