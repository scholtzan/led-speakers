use serde::{Deserialize, Serialize};

use crate::theme::Color;
use crate::viz::PixelViz;
use crate::viz::Viz;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RotatingVizConfig {
    pub pretty_name: String,
    pub speed: f32,     // pixels per second
    pub falloff: f32,   // factor of how much brightness is reduced

}

#[derive(Serialize, Deserialize, Clone)]
pub struct RotatingViz {
    pub config: RotatingVizConfig,
    total_pixels: usize,
    elapsed_time: DateTime<Utc>,
    pixels: Vec<PixelViz>,
    falloffs: Vec<f32>,
}

#[typetag::serde]
impl Viz for RotatingViz {
    fn get_name(&self) -> &str {
        "rotating_viz"
    }

    fn get_pretty_name(&self) -> &str {
        &self.config.pretty_name
    }

    fn update(&mut self, input: &Vec<f32>, colors: &Vec<Color>) -> Vec<PixelViz> {
        let total_bands = input.len();
        let mut rng = rand::thread_rng();

        let now = Utc::now();
        let elapsed: f32 = (now - self.elapsed_time).num_milliseconds();

        if self.pixels.len() > 0 {
            let first = self.pixels[0];
            for pixel_index in 1..self.total_pixels {
                if elapsed > 1000.0 / self.speed {
                    // rotate one position
                    self.pixels[pixel_index - 1] = self.pixels[pixel_index];
                    if pixel_index == self.total_pixels - 1 {
                        self.pixels[pixel_index] = first;
                    }
                    self.elapsed_time = now;
                }
            }
        }

        for band in 0..total_bands {
            let active_pixels = input[band] as i64;
            let prev_active_pixels: Vec<usize> = self.pixels.enumerate().flat_map(|(i, p)| {
                if p.color_index == band && self.falloffs[i] == 0.0 {
                    Some(i)
                } else {
                    None
                }
            }).collect::<Vec<usize>>;

            let change_probability: i64 = (100.0 * prev_active_pixels.len() as f32 / active_pixels as f32) as i64;
            for i in prev_active_pixels.iter() {
                let r = rng.gen_range(0..100) as i64;
                if r < change_probability {
                    self.falloffs[i] = 1.0;
                }
            }

            if change_probability >= 100 {
                // turn some pixels back on falloff is applied to
                let left_to_turn_on = active_pixels - prev_active_pixels.len() as i64;
                let pixels_with_falloff = Vec<usize> = self.pixels.enumerate().flat_map(|(i, p)| {
                    if p.color_index == band && self.falloffs[i] > 0.0 {
                        Some(i)
                    } else {
                        None
                    }
                }).collect::<Vec<usize>>;
                let falloff_change_probability = (100.0 * left_to_turn_on as f32 / pixels_with_falloff.len() as f32) as i64;

                for i in pixels_with_falloff.iter() {
                    let r = rng.gen_range(0..100) as i64;
                    if r < falloff_change_probability {
                        self.falloffs[i] = 0.0;
                    }
                }

                // check if more pixels need to be turned on
                if falloff_change_probability >= 100 {
                    // turn on black pixels
                    let turn_on_off = left_to_turn_on - pixels_with_falloff.len() as i64;
                    let off_pixels = Vec<usize> = self.pixels.enumerate().flat_map(|(i, p)| {
                        if p.color_index == -1 {
                            Some(i)
                        } else {
                            None
                        }
                    }).collect::<Vec<usize>>;
                    let on_change_probability = (100.0 * turn_on_off as f32 / off_pixels.len() as f32) as i64;

                    for i in off_pixels.iter() {
                        let r = rng.gen_range(0..100) as i64;
                        if r < on_change_probability {
                            self.falloffs[i] = 0.0;
                            self.pixels[i].color_index = band;
                        }
                    }
                }
            }

            // apply brightness
            for mut pixel in self.pixels.iter_mut() {
                if pixel.color_index == band {
                    pixel.brightness = input[band] as f32 / 100.0;
                }
            } 
        }

        // apply falloffs
        for (i, falloff) in self.falloffs.iter_mut().enumerate() {
            self.pixels[i].brightness *= 1.0 - (falloff * self.config.falloff).max(1.0);
            falloff += 1.0;

            if falloff * self.config.falloff <= 1.0 {
                self.pixels[i].color_index = -1;
            }
        } 

        // map color_index -1 to off pixels
        let pixels = self.pixels.clone();
        for mut pixel in pixels.iter_mut() {
            if pixel.color_index == -1 {
                pixel.color_index = 0;
                pixel.off();
            }
        }

        pixels
    }

    fn set_total_pixels(&mut self, pixels: usize) {
        self.total_pixels = pixels;
    }
}

unsafe impl Send for RotatingViz {}
unsafe impl Sync for RotatingViz {}

impl RotatingViz {
    pub fn new(config: RotatingVizConfig) -> Self {
        RotatingViz {
            config,
            total_pixels: 0,
        }
    }
}
