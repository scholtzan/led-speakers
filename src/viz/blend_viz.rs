use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::theme::Color;
use crate::viz::PixelViz;
use crate::viz::Viz;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct BlendVizConfig {
    pub pretty_name: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct BlendViz {
    pub config: BlendVizConfig,
    total_pixels: usize,
    max_changes: u8,
    blend_speed: u8,
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
        let mut pixels = vec![PixelViz::default(); self.total_pixels];

        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let in_ms = since_the_epoch.as_secs() * 1000 +
            since_the_epoch.subsec_nanos() as u64 / 1_000_000;
        let shift = (in_ms * ((self.blend_speed as u64 >> 3) + 1) >> 8) as u8;

        for pixel_index in 0..self.total_pixels {
            let current_color = Color {
                r: (colors[pixels[pixel_index].color_index].r as f32 * pixels[pixel_index].red_mul)
                    as u8,
                g: (colors[pixels[pixel_index].color_index].g as f32
                    * pixels[pixel_index].green_mul) as u8,
                b: (colors[pixels[pixel_index].color_index].b as f32 * pixels[pixel_index].blue_mul)
                    as u8,
            };
            let index_shift = Self::index_shift((pixel_index as u8 + 1) * 16);
            let color_index = (shift + index_shift) % colors.len() as u8;
            let target_color = colors[color_index as usize];
            let blend_color = Self::blend(&current_color, &target_color, self.blend_speed);
            pixels[pixel_index].red_mul = blend_color.r as f32 / colors[pixels[pixel_index].color_index].r as f32;
            pixels[pixel_index].green_mul = blend_color.g as f32 / colors[pixels[pixel_index].color_index].g as f32;
            pixels[pixel_index].blue_mul = blend_color.b as f32 / colors[pixels[pixel_index].color_index].b as f32;
        }

        pixels
    }

    fn set_total_pixels(&mut self, pixels: usize) {
        self.total_pixels = pixels;
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
            blend_speed: 10,
        }
    }

    fn blend(color_1: &Color, color_2: &Color, blend: u8) -> Color {
        let blend_max = 128;
        let shift = 16;

        if blend == 0 {
            return color_1.clone();
        } else if blend >= blend_max {
            return color_2.clone();
        }

        Color {
            r: ((color_2.r * blend) + (color_1.r * (blend_max - blend))) >> shift,
            g: ((color_2.g * blend) + (color_1.g * (blend_max - blend))) >> shift,
            b: ((color_2.b * blend) + (color_1.b * (blend_max - blend))) >> shift,
        }
    }

    fn index_shift(index: u8) -> u8 {
        // triangle wave
        let mut out = index;
        if index & 0x80 == 1 {
            out = 255 - index;
        }
        out = out << 1;

        return out

        // // smoothing
        // if out & 0x80 == 1 {
        //     out = 255 - out;
        // }

        

        // out = out * (out / 255);
        // out = out << 1;

        // if i & 0x80 == 1 {
        //     out = 255 - out;
        // }
        // out
    }
}
