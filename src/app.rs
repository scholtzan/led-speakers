use crate::settings::Settings;
use crate::theme::Theme;
use crate::viz::VizRunner;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Serializable representation of a visualization
#[derive(Serialize, Deserialize, Clone)]
pub struct Visualization {
    /// Visualization name.
    pub pretty_name: String,

    /// Unique identifier of visualization.
    pub identifier: String,

    /// Settings of current visualization as map.
    pub settings: Option<HashMap<String, String>>,
}

/// Shared web server state
pub struct AppState {
    /// Visualization runner
    pub viz_runner: Arc<Mutex<VizRunner>>,

    /// Available themes.
    pub themes: Vec<Theme>,

    /// Available visualizations.
    pub visualizations: Vec<Visualization>,

    /// Shared settings.
    pub settings: Arc<Mutex<Settings>>,
}
