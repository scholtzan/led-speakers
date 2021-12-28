use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, Weak};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::viz::Viz;
use crate::viz::PixelViz;

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

    fn update(&mut self) -> Vec<PixelViz> {
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
            total_pixels: 0
        }
    }
}
