use chrono::prelude::*;

use rand::seq::SliceRandom;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::theme::Color;
use crate::viz::PixelViz;
use crate::viz::Viz;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SparkleVizConfig {
    /// Screen friendly name of visualization.
    pub pretty_name: String,

    /// Determines how frequently pixels should be ignited.
    pub speed: f32,

    /// Factor of how much brightness is reduced.
    pub falloff: f32,

    /// Maximum number of pixels ignited at the same time representing the same frequency.
    pub max_ignite: f32,
}

impl SparkleVizConfig {
    /// Convert settings in map of strings to visualization config.
    pub fn to_map(&self) -> HashMap<String, String> {
        let mut settings = HashMap::new();
        settings.insert("speed".to_string(), self.speed.to_string());
        settings.insert("falloff".to_string(), self.falloff.to_string());
        settings.insert("max_ignite".to_string(), self.max_ignite.to_string());
        settings
    }

    /// Create visualization config from map of strings.
    pub fn from_map(name: String, settings: HashMap<String, String>) -> Self {
        Self {
            pretty_name: name,
            falloff: settings
                .get(&"falloff".to_string())
                .unwrap_or(&"0".to_string())
                .parse::<f32>()
                .unwrap(),
            speed: settings
                .get(&"speed".to_string())
                .unwrap_or(&"0".to_string())
                .parse::<f32>()
                .unwrap(),
            max_ignite: settings
                .get(&"max_ignite".to_string())
                .unwrap_or(&"0".to_string())
                .parse::<f32>()
                .unwrap(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
/// Visualization randomly igniting pixels for a short amount of time.
/// The pixel color is determined by the frequency bands and their magnitudes.
pub struct SparkleViz {
    /// Visualization config.
    pub config: SparkleVizConfig,

    /// Total number of pixels.
    total_pixels: usize,

    /// Falloff factors applied to each pixel.
    falloffs: Vec<f32>,

    /// Elapsed time since last time pixels got ignited.
    elapsed_time: DateTime<Utc>,

    /// Pixel colors.
    pixels: Vec<Option<PixelViz>>,
}

#[typetag::serde]
impl Viz for SparkleViz {
    fn get_name(&self) -> &str {
        "sparkle_viz"
    }

    fn get_pretty_name(&self) -> &str {
        &self.config.pretty_name
    }

    fn update(&mut self, input: &Vec<f32>, _colors: &Vec<Color>) -> Vec<PixelViz> {
        let total_bands = input.len();
        let _rng = rand::thread_rng();

        let now = Utc::now();
        let elapsed: f32 = (now - self.elapsed_time).num_milliseconds() as f32;

        if elapsed > 1000.0 / self.config.speed {
            self.elapsed_time = now;
            for band in 0..total_bands {
                // total number of pixels to ignite
                let total_ignite = ((input[band] / 100.0) * self.config.max_ignite) as usize;

                // pixels that are currently off
                let off_pixels = self
                    .pixels
                    .iter()
                    .enumerate()
                    .flat_map(|(i, p)| if let Some(_pixel) = p { None } else { Some(i) })
                    .collect::<Vec<usize>>();

                // randomly determine which off pixels should be ignited
                let pixels_to_spark: Vec<usize> = off_pixels
                    .choose_multiple(&mut rand::thread_rng(), total_ignite)
                    .cloned()
                    .collect();

                // set color
                for pixel in pixels_to_spark {
                    self.pixels[pixel] = Some(PixelViz {
                        color_index: band,
                        red_mul: 1.0,
                        green_mul: 1.0,
                        blue_mul: 1.0,
                        brightness: input[band] / 100.0,
                    });
                    self.falloffs[pixel] = 1.0;
                }
            }
        }

        // apply falloffs
        for (i, falloff) in self.falloffs.iter_mut().enumerate() {
            if let Some(ref mut pixel) = self.pixels[i] {
                if *falloff >= 1.0 {
                    pixel.brightness *= self.config.falloff;
                }
                if pixel.brightness <= 0.1 {
                    self.pixels[i] = None;
                }
            }
        }

        // map color_index -1 to off pixels
        let pixels = self
            .pixels
            .iter()
            .map(|p| {
                if let Some(pixel) = p {
                    return pixel.clone();
                } else {
                    return PixelViz {
                        color_index: 0,
                        red_mul: 0.0,
                        green_mul: 0.0,
                        blue_mul: 0.0,
                        brightness: 0.0,
                    };
                }
            })
            .collect::<Vec<PixelViz>>();

        pixels
    }

    fn set_total_pixels(&mut self, pixels: usize) {
        self.total_pixels = pixels;
        self.pixels = vec![None; pixels];
        self.falloffs = vec![0.0; pixels];
    }

    fn get_settings(&self) -> HashMap<String, String> {
        self.config.to_map()
    }

    fn update_settings(&mut self, settings: HashMap<String, String>) {
        let new_settings = SparkleVizConfig::from_map(self.get_pretty_name().to_string(), settings);
        self.config = new_settings;
        self.pixels = vec![None; self.total_pixels];
        self.falloffs = vec![0.0; self.total_pixels];
    }
}

unsafe impl Send for SparkleViz {}
unsafe impl Sync for SparkleViz {}

impl SparkleViz {
    pub fn new(config: SparkleVizConfig) -> Self {
        SparkleViz {
            config,
            total_pixels: 0,
            elapsed_time: Utc::now(),
            falloffs: Vec::new(),
            pixels: Vec::new(),
        }
    }
}
