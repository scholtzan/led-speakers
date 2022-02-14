use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::prelude::*;
use rand::Rng;

use crate::theme::Color;
use crate::viz::PixelViz;
use crate::viz::Viz;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct BlendVizConfig {
    pub pretty_name: String,
    pub spread: u8,
    pub blend_speed: u8,
    pub offset_weight: u8,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct BlendViz {
    pub config: BlendVizConfig,
    total_pixels: usize,
    max_changes: u8,
    elapsed_time: DateTime<Utc>,
    offsets: Vec<i32>,
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
        let total_bands = input.len();

        let current_color = Color {
            r: (colors[self.pixels[0].color_index].r as f32 * self.pixels[0].red_mul)
                as u8,
            g: (colors[self.pixels[0].color_index].g as f32
                * self.pixels[0].green_mul) as u8,
            b: (colors[self.pixels[0].color_index].b as f32 * self.pixels[0].blue_mul)
                as u8,
        };

        let now = Utc::now();
        let elapsed = (now - self.elapsed_time).num_seconds();

        let mut color_index = self.pixels[0].color_index;
        let mut target_color = colors[color_index];

        if elapsed > (self.config.blend_speed as i64) {
            color_index = (self.pixels[0].color_index + 1) % colors.len();
            target_color = colors[color_index];
            self.elapsed_time = now;
            
            for pixel_index in 0..self.total_pixels {
                self.pixels[pixel_index].red_mul = colors[self.pixels[pixel_index].color_index].r as f32 / target_color.r as f32;
                self.pixels[pixel_index].green_mul = colors[self.pixels[pixel_index].color_index].g as f32 / target_color.g as f32;
                self.pixels[pixel_index].blue_mul = colors[self.pixels[pixel_index].color_index].b as f32 / target_color.b as f32;
                self.pixels[pixel_index].color_index = color_index;
            }
        }

        for pixel_index in 0..self.total_pixels {
            let blend_color = Self::blend(&current_color, &target_color);
            self.pixels[pixel_index].red_mul = blend_color.r as f32 / target_color.r as f32;
            self.pixels[pixel_index].green_mul = blend_color.g as f32 / target_color.g as f32;
            self.pixels[pixel_index].blue_mul = blend_color.b as f32 / target_color.b as f32;
        }

        self.pixels.clone()
    }

    fn set_total_pixels(&mut self, pixels: usize) {
        self.total_pixels = pixels;
        self.pixels = vec![PixelViz::default(); pixels];
        self.update_offsets();
    }
}

unsafe impl Send for BlendViz {}
unsafe impl Sync for BlendViz {}

impl BlendViz {
    pub fn new(config: BlendVizConfig) -> Self {
        BlendViz {
            config,
            total_pixels: 0,
            max_changes: 24,
            elapsed_time: Utc::now(),
            offsets: Vec::new(),
            pixels: Vec::new()
        }
    }

    fn update_offsets(&mut self) {
        let mut rng = rand::thread_rng();
        let total_pixels = self.total_pixels;
        let mut offsets: Vec<i32> = vec![0; total_pixels];
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
                    let mut increase = self.config.offset_weight as i32;
                    if !increasing {
                        let increase = -(self.config.offset_weight as i32);
                    } 

                    offsets[pixel_index + i] = offsets[pixel_index + i - 1] + increase;
                }
            }
            increasing = !increasing;
            pixel_index += n;
        }

        self.offsets = offsets;
    }

    fn blend(color_1: &Color, color_2: &Color) -> Color {
        // let blend_max = 24;
        Color {
            r: (color_1.r as i32 + (color_2.r as i32 - color_1.r as i32) / 3) as u8,
            g: (color_1.g as i32 + (color_2.g as i32 - color_1.g as i32) / 3) as u8,
            b: (color_1.b as i32 + (color_2.b as i32 - color_1.b as i32) / 3) as u8,
        }
    }

    // fn index_shift(index: u8) -> u8 {
    //     // triangle wave
    //     // let mut out = index;
    //     // if index & 0x80 == 1 {
    //     //     out = 255 - index;
    //     // }
    //     // out = out << 1;

    //     // return out

    //     // // smoothing
    //     // if out & 0x80 == 1 {
    //     //     out = 255 - out;
    //     // }

        

    //     // out = out * (out / 255);
    //     // out = out << 1;

    //     // if i & 0x80 == 1 {
    //     //     out = 255 - out;
    //     // }
    //     // out
    // }
}
