use rppal::gpio::{Gpio, OutputPin};
use rppal::spi;

pub struct Led {
    pixels: Vec<Pixel>,
    spi: spi::Spi,
}

impl Led {
    fn new(total_pixels: u8, bus: spi::Bus, clock_speed_hz: u32) -> Self {
        Led {
            pixels: vec![Pixel::default(); total_pixels],
            spi: Spi::new(bus,
                spi::SlaveSelect::Ss0,
                clock_speed_hz,
                spi::Mode::Mode0
            ),
        }
    }

    fn write(&mut self, data: &[u8]) -> Result<()> {
        self.spi.write(data)?;
        Ok(())
    }

    fn set_pixel(&mut self, pixel: usize, red: u8, green: u8, blue: u8, brightness: u8) {
        if let Some(pixel) = self.pixels.get_mut(pixel) {
            pixel.set_rgba(red, green, blue, brightness);
        }
    }

    pub fn set_all_pixels(&mut self, red: u8, green: u8, blue: u8, brightness: u8) {
        for pixel in &mut self.pixels {
            pixel.set_rgba(red, green, blue, brightness);
        }
    }

    pub fn clear(&mut self) {
        self.set_all_pixels(0, 0, 0);
    }

    pub fn show(&mut self) -> Result<()> {
        self.write(&[0u8; 4])?;

        // LED frames (3*1, 5*brightness, 8*blue, 8*green, 8*red).
        for pixel in &self.pixels {
            self.write(pixel.bytes())?;
        }

        // End frame (8*0 for every 16 pixels, 32*0 SK9822 reset frame).
        // The SK9822 won't update any pixels until it receives the next
        // start frame (32*0). The APA102 doesn't care if we send zeroes
        // instead of ones as the end frame. This workaround is
        // compatible with both the APA102 and SK9822.
        // self.spi.write(&self.end_frame)?;

        Ok(())
    }
}


pub struct Pixel {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub brightness: u8
}

impl Pixel {
    fn bytes(&self) -> &[u8] {
        return &[self.red, self.green, self.blue, self.brightness]
    }

    fn set_rgb(&mut self, red: u8, green: u8, blue: u8, brightness: u8) {
        self.red = red;
        self.green = green;
        self.blue = blue;
        self.brightness = brightness;
    }

    fn clear(&mut self) {
        self.red = 0;
        self.green = 0;
        self.blue = 0;
        self.brightness = 0;
    }
}

impl Default for Pixel {
    fn default() -> Pixel {
        Pixel {
            red: 0,
            green: 0,
            blue: 0,
            brightness: 0
        }
    }
}
