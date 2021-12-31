use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, Weak};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::viz::Viz;
use crate::led::Led;
use crate::viz::PixelViz;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SolidVizConfig {
    pub pretty_name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SolidViz {
    pub config: SolidVizConfig,
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

    fn update(&mut self, input: &Vec<f32>) -> Vec<PixelViz> {
        vec![
            PixelViz::default();
            self.total_pixels
        ]
    }

    fn set_total_pixels(&mut self, pixels: usize) {
        self.total_pixels = pixels;
    }
}

unsafe impl Send for SolidViz {}
unsafe impl Sync for SolidViz {}

impl SolidViz {
    pub fn new(config: SolidVizConfig) -> Self {
        SolidViz {
            config,
            total_pixels: 0
        }
    }
}