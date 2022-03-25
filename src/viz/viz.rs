use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::settings::OutputSettings;
use crate::theme::Color;
use crate::theme::Theme;
use crate::transform::AudioTransformer;

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

    // todo: get_settings()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Represents the visualization state of an LED pixel.
pub struct PixelViz {
    /// Index of the theme color to use
    pub color_index: usize,

    /// Multiplier applied to red
    pub red_mul: f32,

    // Multiplier applied to green
    pub green_mul: f32,

    /// Multiplier applied to blue
    pub blue_mul: f32,

    /// Brightness multiplier
    pub brightness: f32,
}

impl PixelViz {
    /// Turns off the pixel in the visualization.
    pub fn off(&mut self) {
        self.brightness = 0.0;
    }
}

impl Default for PixelViz {
    /// Creates a default `PixelViz` instance
    fn default() -> PixelViz {
        PixelViz {
            color_index: 0,
            red_mul: 1.0,
            green_mul: 1.0,
            blue_mul: 1.0,
            brightness: 1.0,
        }
    }
}

/// Executes and updates the visualization for all output channels.
pub struct VizRunner {
    /// Left visualization
    pub viz_left: Arc<Mutex<Box<dyn Viz>>>,

    /// Right visualization
    pub viz_right: Arc<Mutex<Box<dyn Viz>>>,

    /// Output settings
    pub output_settings: OutputSettings,

    /// Whether the visualization is running and is getting updated
    pub is_stopped: Arc<AtomicBool>,

    /// Theme to use in visualization
    pub theme: Theme,

    /// Audio transformer; used for visualization input
    pub transformer: Arc<Mutex<AudioTransformer>>,
}

impl VizRunner {
    pub fn start(&self) {
        // make values available in thread
        let stopped = self.is_stopped.clone();
        let left_viz = Arc::clone(&self.viz_left);
        let right_viz = Arc::clone(&self.viz_right);
        let output = self.output_settings.clone();
        let colors = self.theme.colors.clone();
        let transformer = Arc::clone(&self.transformer);

        let handle = thread::spawn(move || {
            // init outputs from settings
            let mut left_output = output.left.to_led();
            let mut right_output = output.right.to_led();

            while !stopped.load(Ordering::Relaxed) {
                // update visualizations for left and right channel
                let left_pixel_viz = left_viz.lock().unwrap().update(
                    &transformer.lock().unwrap().left_bands.lock().unwrap(),
                    &colors,
                );
                let right_pixel_viz = right_viz.lock().unwrap().update(
                    &transformer.lock().unwrap().right_bands.lock().unwrap(),
                    &colors,
                );

                // show pixel visualizations and apply multipliers
                for (i, pixel_viz) in left_pixel_viz.iter().enumerate() {
                    let color = colors[pixel_viz.color_index % colors.len()];
                    left_output.set_pixel(
                        i,
                        ((color.r as f32) * pixel_viz.red_mul) as u8,
                        ((color.g as f32) * pixel_viz.green_mul) as u8,
                        ((color.b as f32) * pixel_viz.blue_mul) as u8,
                        pixel_viz.brightness,
                    )
                }

                for (i, pixel_viz) in right_pixel_viz.iter().enumerate() {
                    let color = colors[pixel_viz.color_index % colors.len()];
                    right_output.set_pixel(
                        i,
                        ((color.r as f32) * pixel_viz.red_mul) as u8,
                        ((color.g as f32) * pixel_viz.green_mul) as u8,
                        ((color.b as f32) * pixel_viz.blue_mul) as u8,
                        pixel_viz.brightness,
                    )
                }

                left_output.show();
                right_output.show();
            }
        });
    }

    /// Stops the visualization from updating and running.
    pub fn stop(&mut self) {
        self.is_stopped = Arc::new(AtomicBool::from(false));
    }

    /// Sets the provided theme for the visualization.
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }
}
