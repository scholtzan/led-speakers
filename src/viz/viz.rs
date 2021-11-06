#[typetag::serde(tag = "type")]
pub trait Viz {
    fn get_name(&self) -> &str;
    fn get_pretty_name(&self) -> &str;
}
