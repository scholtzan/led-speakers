use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, Weak};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::viz::Viz;
use crate::led::Led;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SolidVizConfig {
    pub pretty_name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SolidViz {
    pub config: SolidVizConfig,
}

#[typetag::serde]
impl Viz for SolidViz {
    fn get_name(&self) -> &str {
        "solid_viz"
    }

    fn get_pretty_name(&self) -> &str {
        &self.config.pretty_name
    }

    fn update(&mut self) {
        // todo: show first theme color on all leds
    }
}

unsafe impl Send for SolidViz {}
unsafe impl Sync for SolidViz {}

impl SolidViz {
    pub fn new(config: SolidVizConfig) -> Self {
        SolidViz {
            config
        }
    }
}