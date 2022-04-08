use crate::theme::Color;
use crate::viz::PixelViz;
use crate::viz::Viz;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
/// Visualization Config.
pub struct SolidVizConfig {
    /// Screen friendly name of visualization.
    pub pretty_name: String,
}

impl SolidVizConfig {
    /// Convert settings in map of strings to visualization config.
    pub fn to_map(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    /// Create visualization config from map of strings.
    pub fn from_map(name: String, _settings: HashMap<String, String>) -> Self {
        Self { pretty_name: name }
    }
}

#[derive(Serialize, Deserialize, Clone)]
/// Visualization that shows all pixels in a static color without changing them.
pub struct SolidViz {
    /// Visualization config.
    pub config: SolidVizConfig,

    /// Total number of pixels.
    total_pixels: usize,
}

#[typetag::serde]
impl Viz for SolidViz {
    fn get_name(&self) -> &str {
        "solid_viz"
    }

    fn get_pretty_name(&self) -> &str {
        &self.config.pretty_name
    }

    fn update(&mut self, _input: &Vec<f32>, _colors: &Vec<Color>) -> Vec<PixelViz> {
        vec![PixelViz::default(); self.total_pixels]
    }

    fn set_total_pixels(&mut self, pixels: usize) {
        self.total_pixels = pixels;
    }

    fn get_settings(&self) -> HashMap<String, String> {
        self.config.to_map()
    }

    fn update_settings(&mut self, settings: HashMap<String, String>) {
        let new_settings = SolidVizConfig::from_map(self.get_pretty_name().to_string(), settings);
        self.config = new_settings;
    }
}

unsafe impl Send for SolidViz {}
unsafe impl Sync for SolidViz {}

impl SolidViz {
    pub fn new(config: SolidVizConfig) -> Self {
        SolidViz {
            config,
            total_pixels: 0,
        }
    }
}
