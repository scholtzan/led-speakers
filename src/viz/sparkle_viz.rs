use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, Weak};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::viz::Viz;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SparkleVizConfig {
    pub pretty_name: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SparkleViz {
    pub config: SparkleVizConfig,
}

#[typetag::serde]
impl Viz for SparkleViz {
    fn get_name(&self) -> &str {
        "sparkle_viz"
    }

    fn get_pretty_name(&self) -> &str {
        &self.config.pretty_name
    }

    fn update(&mut self) {
        
    }
}

unsafe impl Send for SparkleViz {}
unsafe impl Sync for SparkleViz {}

impl SparkleViz {
    pub fn new(config: SparkleVizConfig) -> Self {
        SparkleViz {
            config
        }
    }
}