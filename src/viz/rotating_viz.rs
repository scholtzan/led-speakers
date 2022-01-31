use serde::{Deserialize, Serialize};

use crate::theme::Color;
use crate::viz::PixelViz;
use crate::viz::Viz;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RotatingVizConfig {
    pub pretty_name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RotatingViz {
    pub config: RotatingVizConfig,
    total_pixels: usize,
}

#[typetag::serde]
impl Viz for RotatingViz {
    fn get_name(&self) -> &str {
        "rotating_viz"
    }

    fn get_pretty_name(&self) -> &str {
        &self.config.pretty_name
    }

    fn update(&mut self, input: &Vec<f32>, colors: &Vec<Color>) -> Vec<PixelViz> {
        vec![]
    }

    fn set_total_pixels(&mut self, pixels: usize) {
        self.total_pixels = pixels;
    }
}

unsafe impl Send for RotatingViz {}
unsafe impl Sync for RotatingViz {}

impl RotatingViz {
    pub fn new(config: RotatingVizConfig) -> Self {
        RotatingViz {
            config,
            total_pixels: 0,
        }
    }
}
