use serde::{Deserialize, Serialize};

use crate::viz::Viz;


#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct RotatingViz {
    pub pretty_name: String
}

#[typetag::serde]
impl Viz for RotatingViz {
    fn get_name(&self) -> &str {
        "rotating_viz"
    }

    fn get_pretty_name(&self) -> &str {
        &self.pretty_name
    }
}
