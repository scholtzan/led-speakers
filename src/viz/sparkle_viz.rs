use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, Weak};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::led::Led;
use crate::viz::Viz;
use crate::viz::PixelViz;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SparkleVizConfig {
    pub pretty_name: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SparkleViz {
    pub config: SparkleVizConfig,
    total_pixels: usize,
}

#[typetag::serde]
impl Viz for SparkleViz {
    fn get_name(&self) -> &str {
        "sparkle_viz"
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

unsafe impl Send for SparkleViz {}
unsafe impl Sync for SparkleViz {}

impl SparkleViz {
    pub fn new(config: SparkleVizConfig) -> Self {
        SparkleViz {
            config,
            total_pixels: 0
        }
    }
}