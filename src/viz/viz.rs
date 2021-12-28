use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, Mutex, Weak};
use std::sync::atomic::{AtomicBool, Ordering};
use dyn_clone::DynClone;

use crate::led::Led;
use crate::theme::Theme;
use std::time;


#[typetag::serde]
pub trait Viz: DynClone + Sync + Send {
    fn get_name(&self) -> &str;
    fn get_pretty_name(&self) -> &str;
    fn update(&mut self) -> Vec<PixelViz>;
    fn set_total_pixels(&mut self, pixels: usize);
}

#[derive(Clone)]
pub struct PixelViz {
    pub color_index: usize,  // index of the theme color
    pub brightness_mul: f32,  // brightness multiplier
    pub red_mul: f32,         // multiplier applied to red
    pub green_mul: f32,       // multiplier applied to green
    pub blue_mul: f32,        // multiplier applied to blue   
}

impl Default for PixelViz {
    fn default() -> PixelViz {
        PixelViz {
            color_index: 0,
            brightness_mul: 1.0,
            red_mul: 1.0,
            green_mul: 1.0,
            blue_mul: 1.0
        }
    }
}


// todo: move threading into viz runner
// todo: move start stop into viz runner
pub struct VizRunner {
    pub viz_left: Arc<Mutex<Box<dyn Viz>>>,
    pub viz_right: Arc<Mutex<Box<dyn Viz>>>,
    pub output_left: Arc<Mutex<Led>>,
    pub output_right: Arc<Mutex<Led>>,
    pub is_stopped: Arc<AtomicBool>,
    pub theme: Theme,
}

impl VizRunner {
    pub fn start(&self) {
        let stopped = self.is_stopped.clone();
        let left_viz = Arc::clone(&self.viz_left);
        let right_viz = Arc::clone(&self.viz_right);
        let left_output = Arc::clone(&self.output_left);
        let right_output = Arc::clone(&self.output_right);
        let colors = self.theme.colors.clone();

        let handle = thread::spawn(move || {
            while !stopped.load(Ordering::Relaxed) {
                let left_pixel_viz = left_viz.lock().unwrap().update();
                let right_pixel_viz = right_viz.lock().unwrap().update();

                for (i, pixel_viz) in left_pixel_viz.iter().enumerate() {
                    let color = colors[pixel_viz.color_index % colors.len()];
                    left_output.lock().unwrap().set_pixel(
                        i, 
                        (((color.r as f32) * pixel_viz.red_mul) as u8),
                        (((color.g as f32) * pixel_viz.green_mul) as u8),
                        (((color.b as f32) * pixel_viz.blue_mul) as u8),
                        (((color.a as f32) * pixel_viz.brightness_mul) as u8),
                    )
                }

                for (i, pixel_viz) in right_pixel_viz.iter().enumerate() {
                    let color = colors[pixel_viz.color_index % colors.len()];
                    right_output.lock().unwrap().set_pixel(
                        i, 
                        (((color.r as f32) * pixel_viz.red_mul) as u8),
                        (((color.g as f32) * pixel_viz.green_mul) as u8),
                        (((color.b as f32) * pixel_viz.blue_mul) as u8),
                        (((color.a as f32) * pixel_viz.brightness_mul) as u8),
                    )
                }

                left_output.lock().unwrap().show();
                right_output.lock().unwrap().show();
                thread::sleep(time::Duration::from_micros(500));
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

