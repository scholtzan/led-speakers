use crate::settings::Settings;
use crate::theme::Theme;
use crate::viz::VizRunner;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use std::sync::{Arc, Mutex, Weak};

/// Serializable representation of a visualization
#[derive(Serialize, Deserialize, Clone)]
pub struct Visualization {
    pub pretty_name: String,
    pub identifier: String,
    pub settings: Option<Value>,
}

/// Shared web server state
pub struct AppState {
    pub viz_runner: Arc<Mutex<VizRunner>>,
    pub themes: Vec<Theme>,
    pub visualizations: Vec<Visualization>,
    pub settings: Arc<Settings>,
}
