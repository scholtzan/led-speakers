use chrono::prelude::*;
use chrono::Duration;
use rand::{distributions::Uniform, Rng};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::theme::Color;
use crate::viz::PixelViz;
use crate::viz::Viz;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct BlendVizConfig {
    pub pretty_name: String,
    pub spread: u8,
    pub blend_speed: u8,
    pub offset_weight: i64,
    pub blend_factor: u8,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct BlendViz {
    pub config: BlendVizConfig,
    total_pixels: usize,
    elapsed_time: Vec<DateTime<Utc>>,
    pixels: Vec<PixelViz>,
}

#[typetag::serde]
impl Viz for BlendViz {
    fn get_name(&self) -> &str {
        "blend_viz"
    }

    fn get_pretty_name(&self) -> &str {
        &self.config.pretty_name
    }

    fn update(&mut self, input: &Vec<f32>, colors: &Vec<Color>) -> Vec<PixelViz> {
        // todo: use bands for colors
        let total_bands = input.len();
        let now = Utc::now();

        for pixel_index in 0..self.total_pixels {
            let elapsed = (now - self.elapsed_time[pixel_index]).num_seconds();
            let current_color = Color {
                r: ((colors[self.pixels[pixel_index].color_index].r as f32)
                    * self.pixels[pixel_index].red_mul) as u8,
                g: ((colors[self.pixels[pixel_index].color_index].g as f32)
                    * self.pixels[pixel_index].green_mul) as u8,
                b: ((colors[self.pixels[pixel_index].color_index].b as f32)
                    * self.pixels[pixel_index].blue_mul) as u8,
            };

            let mut color_index = self.pixels[pixel_index].color_index;
            let mut target_color = colors[color_index];

            if elapsed > (self.config.blend_speed as i64) {
                color_index = (self.pixels[pixel_index].color_index
                    + (elapsed % self.config.blend_speed as i64) as usize)
                    % colors.len();
                target_color = colors[color_index];
                self.elapsed_time[pixel_index] = now;

                self.pixels[pixel_index].color_index = color_index;
            }

            let blend_color = Self::blend(&current_color, &target_color, self.config.blend_factor);
            self.pixels[pixel_index].red_mul = (blend_color.r as f32) / (target_color.r as f32);
            self.pixels[pixel_index].green_mul = (blend_color.g as f32) / (target_color.g as f32);
            self.pixels[pixel_index].blue_mul = (blend_color.b as f32) / (target_color.b as f32);
        }

        self.pixels.clone()
    }

    fn set_total_pixels(&mut self, pixels: usize) {
        self.total_pixels = pixels;
        self.pixels = vec![PixelViz::default(); pixels];
        self.elapsed_time = vec![Utc::now(); pixels];
        let offsets = self.offsets();

        for pixel_index in 0..self.total_pixels {
            self.elapsed_time[pixel_index] =
                Utc::now() + Duration::milliseconds(offsets[pixel_index]);
        }
    }
}

unsafe impl Send for BlendViz {}
unsafe impl Sync for BlendViz {}

impl BlendViz {
    pub fn new(config: BlendVizConfig) -> Self {
        BlendViz {
            config,
            total_pixels: 0,
            elapsed_time: Vec::new(),
            pixels: Vec::new(),
        }
    }

    fn offsets(&mut self) -> Vec<i64> {
        let mut rng = rand::thread_rng();
        let total_pixels = self.total_pixels;

        let mut offsets: Vec<i64> = (0..total_pixels)
            .map(|_| rng.sample(&Uniform::new(0, self.config.offset_weight)))
            .collect();
        let spread = self.config.spread;

        let mut pixel_index = 0;
        let mut increasing = true;

        while pixel_index < total_pixels {
            let mut n = rng.gen_range(0..spread) as usize;
            if pixel_index + n > total_pixels {
                n = total_pixels - pixel_index;
            }

            for i in 0..n {
                if pixel_index + i == 0 {
                    offsets[pixel_index + i] = 0;
                } else {
                    let mut increase = self.config.offset_weight as i64;
                    if !increasing {
                        increase = -(self.config.offset_weight as i64);
                    }

                    offsets[pixel_index + i] = offsets[pixel_index + i - 1] + increase;
                }
            }
            increasing = !increasing;
            pixel_index += n;
        }

        offsets
    }

    fn blend(color_1: &Color, color_2: &Color, blend_factor: u8) -> Color {
        let mut target_color = Color {
            r: color_1.r,
            g: color_1.g,
            b: color_1.b,
        };

        if color_2.r > color_1.r {
            if color_2.r - color_1.r > blend_factor {
                target_color.r += blend_factor;
            }
        } else {
            if color_1.r - color_2.r > blend_factor {
                target_color.r -= blend_factor;
            }
        }

        if color_2.g > color_1.g {
            if color_2.g - color_1.g > blend_factor {
                target_color.g += blend_factor;
            }
        } else {
            if color_1.g - color_2.g > blend_factor {
                target_color.g -= blend_factor;
            }
        }

        if color_2.b > color_1.b {
            if color_2.b - color_1.b > blend_factor {
                target_color.b += blend_factor;
            }
        } else {
            if color_1.b - color_2.b > blend_factor {
                target_color.b -= blend_factor;
            }
        }

        target_color
    }
}
