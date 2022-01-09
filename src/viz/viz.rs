use dyn_clone::DynClone;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::thread;
use std::thread::JoinHandle;

use crate::led::Led;
use crate::settings::OutputSettings;
use crate::theme::Theme;
use crate::transform::AudioTransformer;
use std::time;

#[typetag::serde]
pub trait Viz: DynClone + Sync + Send {
    fn get_name(&self) -> &str;
    fn get_pretty_name(&self) -> &str;
    fn update(&mut self, input: &Vec<f32>) -> Vec<PixelViz>;
    fn set_total_pixels(&mut self, pixels: usize);
}

#[derive(Clone)]
pub struct PixelViz {
    pub color_index: usize,  // index of the theme color
    pub red_mul: f32,        // multiplier applied to red
    pub green_mul: f32,      // multiplier applied to green
    pub blue_mul: f32,       // multiplier applied to blue
}

impl PixelViz {
    /// Turns off the pixel in the viz.
    pub fn off(&mut self) {
        self.red_mul = 0.0;
        self.blue_mul = 0.0;
        self.green_mul = 0.0;
    }
}

impl Default for PixelViz {
    fn default() -> PixelViz {
        PixelViz {
            color_index: 0,
            red_mul: 1.0,
            green_mul: 1.0,
            blue_mul: 1.0,
        }
    }
}


pub struct VizRunner {
    pub viz_left: Arc<Mutex<Box<dyn Viz>>>,
    pub viz_right: Arc<Mutex<Box<dyn Viz>>>,
    pub output_settings: OutputSettings,
    pub is_stopped: Arc<AtomicBool>,
    pub theme: Theme,
    pub transformer: Arc<Mutex<AudioTransformer>>,
}

impl VizRunner {
    pub fn start(&self) {
        let stopped = self.is_stopped.clone();
        let left_viz = Arc::clone(&self.viz_left);
        let right_viz = Arc::clone(&self.viz_right);
        let output = self.output_settings.clone();
        let colors = self.theme.colors.clone();
        let transformer = Arc::clone(&self.transformer);

        let handle = thread::spawn(move || {
            let mut left_output = output.left.to_led();
            let mut right_output = output.right.to_led();

            while !stopped.load(Ordering::Relaxed) {
                let left_pixel_viz = left_viz
                    .lock()
                    .unwrap()
                    .update(&transformer.lock().unwrap().left_bands.lock().unwrap());
                let right_pixel_viz = right_viz
                    .lock()
                    .unwrap()
                    .update(&transformer.lock().unwrap().right_bands.lock().unwrap());

                for (i, pixel_viz) in left_pixel_viz.iter().enumerate() {
                    let color = colors[pixel_viz.color_index % colors.len()];
                    left_output.set_pixel(
                        i,
                        ((color.r as f32) * pixel_viz.red_mul) as u8,
                        ((color.g as f32) * pixel_viz.green_mul) as u8,
                        ((color.b as f32) * pixel_viz.blue_mul) as u8,
                    )
                }

                for (i, pixel_viz) in right_pixel_viz.iter().enumerate() {
                    let color = colors[pixel_viz.color_index % colors.len()];
                    right_output.set_pixel(
                        i,
                        ((color.r as f32) * pixel_viz.red_mul) as u8,
                        ((color.g as f32) * pixel_viz.green_mul) as u8,
                        ((color.b as f32) * pixel_viz.blue_mul) as u8,
                    )
                }

                left_output.show();
                right_output.show();
            }
        });
    }

    pub fn stop(&mut self) {
        self.is_stopped = Arc::new(AtomicBool::from(false));
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }
}
