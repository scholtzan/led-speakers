use ws2818_rgb_led_spi_driver::adapter_gen::WS28xxAdapter;
use ws2818_rgb_led_spi_driver::adapter_spi::WS28xxSpiAdapter;
use ws2818_rgb_led_spi_driver::encoding::encode_rgb;

pub struct Led {
    pixels: Vec<Pixel>,
    controller: WS28xxSpiAdapter,
}

impl Led {
    pub fn new(total_pixels: i32, spi: String, clock_speed_hz: u32) -> Self {
        Led {
            pixels: vec![Pixel::default(); total_pixels as usize],
            controller: WS28xxSpiAdapter::new(&spi).unwrap(),
        }
    }

    pub fn set_pixel(&mut self, pixel: usize, red: u8, green: u8, blue: u8, brightness: u8) {
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
        self.set_all_pixels(0, 0, 0, 0);
    }

    pub fn show(&mut self) {
        let rgb_values = self
            .pixels
            .iter()
            .map(|p: &Pixel| (p.red, p.green, p.blue))
            .collect::<Vec<_>>();
        // for (i, led) in leds.iter_mut().enumerate() {
        //     let pixel = &self.pixels[i];
        //     // eprintln!("{:?} {:?} {:?} {:?}", pixel.blue, pixel.green, pixel.red, pixel.brightness);
        //     *led = [pixel.blue, pixel.green, pixel.red, pixel.brightness];
        // }

        self.controller.write_rgb(&rgb_values).unwrap();
    }
}

#[derive(Clone)]
pub struct Pixel {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub brightness: u8,
}

impl Pixel {
    fn set_rgba(&mut self, red: u8, green: u8, blue: u8, brightness: u8) {
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
            brightness: 0,
        }
    }
}
