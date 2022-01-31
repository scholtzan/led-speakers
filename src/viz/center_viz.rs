use serde::{Deserialize, Serialize};

use crate::theme::Color;
use crate::viz::PixelViz;
use crate::viz::Viz;

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

    fn update(&mut self, input: &Vec<f32>, colors: &Vec<Color>) -> Vec<PixelViz> {
        let total_bands = input.len();
        let pixels_per_band: usize = (self.total_pixels / 2) / total_bands;
        let mut pixels = vec![PixelViz::default(); self.total_pixels];

        let mut overall_intensity = 0;
        for (band_index, band) in input.iter().enumerate() {
            let intensity = ((band / 100.0) * (pixels_per_band as f32)) as usize;
            overall_intensity += intensity;
        }
        let mut unused_pixels = self.total_pixels / 2 - overall_intensity;

        let mut pixel_index = 0;
        for (band_index, band) in input.iter().enumerate() {
            let intensity = ((band / 100.0) * (pixels_per_band as f32)) as usize;
            let amplified_intensity =
                intensity + ((band / 100.0) * unused_pixels as f32).round() as usize;
            unused_pixels -= ((band / 100.0) * unused_pixels as f32).round() as usize;

            for i in 0..amplified_intensity {
                pixels[(self.total_pixels / 2) + pixel_index + i].color_index = band_index;
                pixels[(self.total_pixels / 2) - pixel_index - i].color_index = band_index;
            }

            pixel_index += amplified_intensity;
        }

        for i in pixel_index..(self.total_pixels / 2) {
            pixels[(self.total_pixels / 2) + i].off();
            pixels[(self.total_pixels / 2) - i].off();
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
            total_pixels: 0,
        }
    }
}
