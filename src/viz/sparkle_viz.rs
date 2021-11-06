use serde::{Deserialize, Serialize};

use crate::viz::Viz;


#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SparkleViz {
    pub pretty_name: String
}

#[typetag::serde]
impl Viz for SparkleViz {
    fn get_name(&self) -> &str {
        "sparkle_viz"
    }

    fn get_pretty_name(&self) -> &str {
        &self.pretty_name
    }
}
