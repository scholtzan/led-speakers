use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, Weak};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::viz::Viz;
use crate::led::Led;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RotatingVizConfig {
    pub pretty_name: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct RotatingViz {
    pub config: RotatingVizConfig,
    pub led: Led
}

#[typetag::serde]
impl Viz for RotatingViz {
    fn get_name(&self) -> &str {
        "rotating_viz"
    }

    fn get_pretty_name(&self) -> &str {
        &self.config.pretty_name
    }

    fn update(&mut self) {
        
    }
}

unsafe impl Send for RotatingViz {}
unsafe impl Sync for RotatingViz {}

impl RotatingViz {
    pub fn new(config: RotatingVizConfig) -> Self {
        RotatingViz {
            config
        }
    }
}
