use serde::{Deserialize, Serialize};

use crate::theme::Color;
use crate::viz::PixelViz;
use crate::viz::Viz;
use chrono::prelude::*;
use chrono::Duration;

use rand::{distributions::Uniform, Rng};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RotatingVizConfig {
    pub pretty_name: String,
    pub speed: f32,   // pixels per second
    pub falloff: f32, // factor of how much brightness is reduced
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RotatingViz {
    pub config: RotatingVizConfig,
    total_pixels: usize,
    elapsed_time: DateTime<Utc>,
    pixels: Vec<Option<PixelViz>>,
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
        let elapsed: f32 = (now - self.elapsed_time).num_milliseconds() as f32;

        // rotate pixels
        if self.pixels.len() > 0 {
            let first = self.pixels[0].clone();
            let first_falloff = self.falloffs[0];

            for pixel_index in 1..self.total_pixels {
                if elapsed > 1000.0 / self.config.speed {
                    // rotate one position
                    self.pixels[pixel_index - 1] = self.pixels[pixel_index].clone();
                    self.falloffs[pixel_index - 1] = self.falloffs[pixel_index];

                    if pixel_index == self.total_pixels - 1 {
                        self.pixels[pixel_index] = first.clone();
                        self.falloffs[pixel_index] = first_falloff;
                    }
                    self.elapsed_time = now;
                }
            }
        }

        for band in 0..total_bands {
            // renew active pixels
            let active_pixels =
                ((input[band] / 100.0) * (self.total_pixels as f32 / total_bands as f32)) as i64;
            let prev_active_pixels = self
                .pixels
                .iter()
                .enumerate()
                .flat_map(|(i, p)| {
                    if let Some(pixel) = p {
                        if pixel.color_index == band && self.falloffs[i] == 0.0 {
                            Some(i)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect::<Vec<usize>>();

            let change_probability: i64 =
                (100.0 * active_pixels as f32 / (prev_active_pixels.len() + 1) as f32) as i64;

            for &i in prev_active_pixels.iter() {
                let r = rng.gen_range(0..100) as i64;
                if r < change_probability {
                    self.falloffs[i] = 0.0;
                } else {
                    self.falloffs[i] = 1.0;
                }
            }

            if change_probability >= 100 {
                // turn some pixels back on falloff is applied to
                let left_to_turn_on = active_pixels - prev_active_pixels.len() as i64;
                let pixels_with_falloff = self
                    .pixels
                    .iter()
                    .enumerate()
                    .flat_map(|(i, p)| {
                        if let Some(pixel) = p {
                            if pixel.color_index == band && self.falloffs[i] > 0.0 {
                                Some(i)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<usize>>();

                // likelihood that falloff is reset
                let falloff_change_probability = (100.0 * left_to_turn_on as f32
                    / (pixels_with_falloff.len() + 1) as f32)
                    as i64;

                for &i in pixels_with_falloff.iter() {
                    let r = rng.gen_range(0..100) as i64;
                    if r < falloff_change_probability {
                        self.falloffs[i] = 0.0;
                    }
                }

                // check if more pixels need to be turned on
                if falloff_change_probability >= 100 {
                    // turn on black pixels
                    let turn_on_off = left_to_turn_on - pixels_with_falloff.len() as i64;
                    let off_pixels = self
                        .pixels
                        .iter()
                        .enumerate()
                        .flat_map(|(i, p)| if let Some(pixel) = p { None } else { Some(i) })
                        .collect::<Vec<usize>>();

                    let on_change_probability =
                        (100.0 * turn_on_off as f32 / (off_pixels.len() + 1) as f32) as i64;

                    for &i in off_pixels.iter() {
                        let r = rng.gen_range(0..100) as i64;
                        if r < on_change_probability {
                            self.falloffs[i] = 0.0;
                            self.pixels[i] = Some(PixelViz {
                                color_index: band,
                                red_mul: 1.0,
                                green_mul: 1.0,
                                blue_mul: 1.0,
                                brightness: 1.0,
                            });
                        }
                    }
                }
            }

            // apply brightness
            for mut p in self.pixels.iter_mut() {
                if let Some(ref mut pixel) = p {
                    if pixel.color_index == band {
                        pixel.brightness = input[band] as f32 / 100.0;
                    }
                }
            }
        }

        // apply falloffs
        for (i, falloff) in self.falloffs.iter_mut().enumerate() {
            if let Some(ref mut pixel) = self.pixels[i] {
                pixel.brightness *= 1.0 - (*falloff * self.config.falloff).min(1.0);
                *falloff += 1.0;
            }

            if *falloff >= 10.0 {
                self.pixels[i] = None;
            }
        }

        // map color_index -1 to off pixels
        let pixels = self
            .pixels
            .iter()
            .map(|p| {
                if let Some(pixel) = p {
                    return pixel.clone();
                } else {
                    return PixelViz {
                        color_index: 0,
                        red_mul: 0.0,
                        green_mul: 0.0,
                        blue_mul: 0.0,
                        brightness: 0.0,
                    };
                }
            })
            .collect::<Vec<PixelViz>>();

        pixels
    }

    fn set_total_pixels(&mut self, pixels: usize) {
        self.total_pixels = pixels;
        self.pixels = vec![None; pixels];
        self.falloffs = vec![0.0; pixels];
    }
}

unsafe impl Send for RotatingViz {}
unsafe impl Sync for RotatingViz {}

impl RotatingViz {
    pub fn new(config: RotatingVizConfig) -> Self {
        RotatingViz {
            config,
            total_pixels: 0,
            elapsed_time: Utc::now(),
            falloffs: Vec::new(),
            pixels: Vec::new(),
        }
    }
}
