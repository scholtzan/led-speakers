use ws2818_rgb_led_spi_driver::adapter_gen::WS28xxAdapter;
use ws2818_rgb_led_spi_driver::adapter_spi::WS28xxSpiAdapter;

/// Representation of an LED strip.
pub struct Led {
    /// Individual LEDs on the strip
    pixels: Vec<Pixel>,

    /// LED strip controller (default: SPI)
    controller: WS28xxSpiAdapter,
}

impl Led {
    /// Creates a new `LED` instance.
    ///
    /// # Examples
    /// ```
    /// let led = Led::new(150, "/dev/spidev0.0".to_string());
    /// ```
    pub fn new(total_pixels: i32, spi: String) -> Self {
        Led {
            pixels: vec![Pixel::default(); total_pixels as usize],
            controller: WS28xxSpiAdapter::new(&spi).unwrap(),
        }
    }

    /// Sets the color of a specific pixel.
    ///
    /// # Example
    /// ```
    /// let mut led = Led::new(150, "/dev/spidev0.0".to_string());
    /// led.set_pixel(255, 0, 0, 1.0);
    /// ```
    pub fn set_pixel(&mut self, pixel: usize, red: u8, green: u8, blue: u8, brightness: f32) {
        if let Some(pixel) = self.pixels.get_mut(pixel) {
            pixel.set_rgba(red, green, blue, brightness);
        }
    }

    /// Sets all pixels to the specified color.
    ///
    /// # Example
    /// ```
    /// let mut led = Led::new(150, "/dev/spidev0.0".to_string());
    /// led.set_all_pixels(255, 0, 0, 1.0);
    /// ```
    pub fn set_all_pixels(&mut self, red: u8, green: u8, blue: u8, brightness: f32) {
        for pixel in &mut self.pixels {
            pixel.set_rgba(red, green, blue, brightness);
        }
    }

    /// Turns off all pixels.
    ///
    /// # Example
    /// ```
    /// let mut led = Led::new(150, "/dev/spidev0.0".to_string());
    /// led.clear();
    /// ```
    pub fn clear(&mut self) {
        self.set_all_pixels(0, 0, 0, 0.0);
    }

    /// Updates pixel values and apply to LEDs of LED strip.
    ///
    /// # Example
    /// ```
    /// let mut led = Led::new(150, "/dev/spidev0.0".to_string());
    /// led.show();
    /// ```
    pub fn show(&mut self) {
        let rgb_values = self
            .pixels
            .iter()
            .map(|p: &Pixel| (p.red, p.green, p.blue))
            .collect::<Vec<_>>();
        self.controller.write_rgb(&rgb_values).unwrap();
    }
}

#[derive(Clone)]
/// Represents a single LED pixels of an LED strip.
pub struct Pixel {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Pixel {
    /// Sets the RGB value of the LED pixel.
    ///
    /// # Examples
    /// ```
    /// let mut pixel = Pixel {
    ///     red: 255,
    ///     blue: 255,
    ///     green: 0
    /// };
    /// led.set_rgba(255, 255, 255);
    /// ```
    fn set_rgba(&mut self, red: u8, green: u8, blue: u8, brightness: f32) {
        self.red = ((red as f32) * brightness) as u8;
        self.green = ((green as f32) * brightness) as u8;
        self.blue = ((blue as f32) * brightness) as u8;
    }
}

impl Default for Pixel {
    /// Creates a new pixel instance which is turned off by default.
    fn default() -> Pixel {
        Pixel {
            red: 0,
            green: 0,
            blue: 0,
        }
    }
}
