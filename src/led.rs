use rppal::gpio::{Gpio, OutputPin};
use rppal::spi;

pub struct Led {
    pixels: Vec<Pixel>,
    spi: spi::Spi,
}

impl Led {
    pub fn new(total_pixels: usize, bus: spi::Bus, clock_speed_hz: u32) -> Self {
        Led {
            pixels: vec![Pixel::default(); total_pixels],
            spi: spi::Spi::new(bus,
                spi::SlaveSelect::Ss0,
                clock_speed_hz,
                spi::Mode::Mode0
            ).unwrap(),
        }
    }

    pub fn write(&mut self, data: &[u8]) {
        self.spi.write(data);
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
        // self.write(&[0u8; 4]);

        // LED frames (3*1, 5*brightness, 8*blue, 8*green, 8*red).
        let mut out: V = vec![];
        for pixel in self.pixels.clone() {
            self.write(&pixel.bytes());
        }

        let end_frame = vec![0u8; 4 + (((self.pixels.len() as f32 / 16.0f32) + 0.94f32) as usize)];

        // self.write(&end_frame);
    }
}

#[derive(Clone)]
pub struct Pixel {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub brightness: u8
}

impl Pixel {
    fn bytes(&self) -> Vec<u8> {
        // return vec![self.brightness, self.blue, self.green, self.red]
        let mut output = self.byte_to_spi_bytes(self.green);
        output.extend(self.byte_to_spi_bytes(self.red));
        output.extend(self.byte_to_spi_bytes(self.blue));
        output
        // return [self.byte_to_spi_bytes(self.green), self.byte_to_spi_bytes(self.red), self.byte_to_spi_bytes(self.blue)]
    }

    fn byte_to_spi_bytes(&self, input: u8) -> Vec<u8> {
        // first convert the u8 to 24 bits
        let mut bool_array = [false; 24];
        for bit_index in 0..8 {
            let bit = input & (1 << bit_index) != 0;
            let out_index = bit_index * 3;
    
            // first bit is always 0
            // this could be omitted because the array is initialized to false
            bool_array[out_index] = false;
    
            bool_array[out_index + 1] = bit;
    
            // last bit is always 1
            bool_array[out_index + 2] = true;
        }
    
        // then convert the 24 bits to three u8
        vec![
            self.bool_slice_to_u8(&bool_array[0..8]),
            self.bool_slice_to_u8(&bool_array[8..16]),
            self.bool_slice_to_u8(&bool_array[16..24]),
        ]
    }

    fn bool_slice_to_u8(&self, input: &[bool]) -> u8 {
        if input.len() != 8 { panic!("bool to u8 conversion requires exactly 8 booleans") }
    
        let mut out = 0b0000_0000u8;
    
        for (carry_bit, flag) in input.iter().enumerate() {
            if *flag { out += 0b0000_0001u8 << carry_bit }
        }
    
        out
    }

    fn set_rgba(&mut self, red: u8, green: u8, blue: u8, brightness: u8) {
        self.red = red;
        self.green = green;
        self.blue = blue;
        self.brightness = 0b1110_0000 | ((31 * brightness.max(0).min(1)) as u8);;
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
