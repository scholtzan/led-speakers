use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::theme::Color;
use crate::viz::PixelViz;
use crate::viz::Viz;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
/// Visualization config.
pub struct CenterVizConfig {
    /// Screen friendly name of visualization.
    pub pretty_name: String,
}

impl CenterVizConfig {
    /// Convert settings in map of strings to visualization config.
    pub fn to_map(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    /// Create visualization config from map of strings.
    pub fn from_map(name: String, _settings: HashMap<String, String>) -> Self {
        Self { pretty_name: name }
    }
}

#[derive(Deserialize, Serialize, Clone)]
/// Visualization turning on pixels from the center of the speaker.
/// Number of pixels turned on depends on audio magnitude.
/// Pixels visualizing lower frequency bands are closer to the center.
pub struct CenterViz {
    /// Visualization config.
    pub config: CenterVizConfig,

    /// Total number of pixels.
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

    fn update(&mut self, input: &Vec<f32>, _colors: &Vec<Color>) -> Vec<PixelViz> {
        let total_bands = input.len();
        // separate pixel bands for each frequency
        // pixel visualization gets mirrored at the speaker center, so divide by 2
        let pixels_per_band: usize = (self.total_pixels / 2) / total_bands;
        let mut pixels = vec![PixelViz::default(); self.total_pixels];

        // total number of pixels that will be turned on
        let mut overall_intensity = 0;
        for (_band_index, band) in input.iter().enumerate() {
            let intensity = ((band / 100.0) * (pixels_per_band as f32)) as usize;
            overall_intensity += intensity;
        }
        // total number of pixels that would be off
        let mut unused_pixels = self.total_pixels / 2 - overall_intensity;

        // determine color for pixels
        let mut pixel_index = 0;
        for (band_index, band) in input.iter().enumerate() {
            // determine magnitude of frequency
            let intensity = ((band / 100.0) * (pixels_per_band as f32)) as usize;

            // ideally no pixels should be turned off;
            // determine how how many of the off pixels should be turned on with the same color based on magnitude
            let amplified_intensity =
                intensity + ((band / 100.0) * unused_pixels as f32).round() as usize;
            // update number of pixels that are still off
            unused_pixels -= ((band / 100.0) * unused_pixels as f32).round() as usize;

            // assign colors to pixels
            for i in 0..amplified_intensity {
                pixels[(self.total_pixels / 2) + pixel_index + i].color_index = band_index;
                pixels[(self.total_pixels / 2) - pixel_index - i].color_index = band_index;
            }

            pixel_index += amplified_intensity;
        }

        // pixels that are unused, turn off
        for i in pixel_index..(self.total_pixels / 2) {
            pixels[(self.total_pixels / 2) + i].off();
            pixels[(self.total_pixels / 2) - i].off();
        }

        pixels
    }

    fn set_total_pixels(&mut self, pixels: usize) {
        self.total_pixels = pixels;
    }

    fn get_settings(&self) -> HashMap<String, String> {
        self.config.to_map()
    }

    fn update_settings(&mut self, settings: HashMap<String, String>) {
        let new_settings = CenterVizConfig::from_map(self.get_pretty_name().to_string(), settings);
        self.config = new_settings;
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
