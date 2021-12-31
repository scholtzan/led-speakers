use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, Weak};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::led::Led;
use crate::viz::Viz;
use crate::viz::PixelViz;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct CenterVizConfig {
    pub pretty_name: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct CenterViz {
    pub config: CenterVizConfig,
    total_pixels: usize,
}

#[typetag::serde]
impl Viz for CenterViz {
    fn get_name(&self) -> &str {
        "center_viz"
    }

    fn get_pretty_name(&self) -> &str {
        &self.config.pretty_name
    }

    fn update(&mut self, input: &Vec<f32>) -> Vec<PixelViz> {
        let total_bands = input.len();
        let pixels_per_band: usize = (self.total_pixels / 2) / total_bands;
        let mut pixels = vec![
            PixelViz::default();
            self.total_pixels
        ];

        let mut pixel_index = 0;
        for (band_index, band) in input.iter().enumerate() {
            let intensity = ((band / 100.0) * (pixels_per_band as f32)) as usize;

            for i in 0..intensity {
                pixels[(self.total_pixels / 2) + pixel_index + i].color_index = band_index;
                pixels[(self.total_pixels / 2) - pixel_index - i].color_index = band_index;
            }

            pixel_index += intensity + 1;
        }

        for i in pixel_index..((self.total_pixels / 2) - 1) {
            // todo: PixelViz::off
            pixels[(self.total_pixels / 2) + i].color_index = 0;
            pixels[(self.total_pixels / 2) - i].color_index = 0;
            pixels[(self.total_pixels / 2) + i].red_mul = 0.0;
            pixels[(self.total_pixels / 2) - i].red_mul = 0.0;
            pixels[(self.total_pixels / 2) + i].green_mul = 0.0;
            pixels[(self.total_pixels / 2) - i].green_mul = 0.0;
            pixels[(self.total_pixels / 2) + i].blue_mul = 0.0;
            pixels[(self.total_pixels / 2) - i].blue_mul = 0.0;
            pixels[(self.total_pixels / 2) + i].brightness_mul = 0.0;
            pixels[(self.total_pixels / 2) - i].brightness_mul = 0.0;
        }

        pixels
    }

    fn set_total_pixels(&mut self, pixels: usize) {
        self.total_pixels = pixels;
    }
}

unsafe impl Send for CenterViz {}
unsafe impl Sync for CenterViz {}

impl CenterViz {
    pub fn new(config: CenterVizConfig) -> Self {
        CenterViz {
            config,
            total_pixels: 0
        }
    }
}