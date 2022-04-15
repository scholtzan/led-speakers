# Visualization

All visualizations implement the `Viz` trait:

```rust
#[typetag::serde]
/// Abstract type implemented by all visualizations.
pub trait Viz: DynClone + Sync + Send {
    /// Returns the unique identifier of the visualization.
    fn get_name(&self) -> &str;

    /// Returns a descriptive visualization name.
    fn get_pretty_name(&self) -> &str;

    /// Updates the visualization state based on the provided input.
    fn update(&mut self, input: &Vec<f32>, colors: &Vec<Color>) -> Vec<PixelViz>;

    /// Sets the number of total available pizels.
    fn set_total_pixels(&mut self, pixels: usize);

    /// Updates the settings.
    fn update_settings(&mut self, settings: HashMap<String, String>);

    /// Returns the viz settings as map.
    fn get_settings(&self) -> HashMap<String, String>;
}
```

The `update` method is where the pixel/LED colors will get updated in each iteration.


[todo]
