use chrono::prelude::*;
use chrono::Duration;
use rand::seq::SliceRandom;
use rand::{distributions::Uniform, Rng};
use serde::{Deserialize, Serialize};

use crate::theme::Color;
use crate::viz::PixelViz;
use crate::viz::Viz;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SparkleVizConfig {
    pub pretty_name: String,
    pub speed: f32,   // pixels per second
    pub falloff: f32, // factor of how much brightness is reduced
    pub max_ignite: f32,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SparkleViz {
    pub config: SparkleVizConfig,
    total_pixels: usize,
    falloffs: Vec<f32>,
    elapsed_time: DateTime<Utc>,
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

    fn update(&mut self, input: &Vec<f32>, colors: &Vec<Color>) -> Vec<PixelViz> {
        let total_bands = input.len();
        let mut rng = rand::thread_rng();

        let now = Utc::now();
        let elapsed: f32 = (now - self.elapsed_time).num_milliseconds() as f32;

        if elapsed > 1000.0 / self.config.speed {
            self.elapsed_time = now;
            for band in 0..total_bands {
                let total_ignite = ((input[band] / 100.0) * self.config.max_ignite) as usize;
                let off_pixels = self
                    .pixels
                    .iter()
                    .enumerate()
                    .flat_map(|(i, p)| if let Some(pixel) = p { None } else { Some(i) })
                    .collect::<Vec<usize>>();
                let pixels_to_spark: Vec<usize> = off_pixels
                    .choose_multiple(&mut rand::thread_rng(), total_ignite)
                    .cloned()
                    .collect();

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
