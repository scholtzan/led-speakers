use byteorder::ByteOrder;
use byteorder::LittleEndian as Le;
use bytes::Buf;
use bytes::BytesMut;
use bytes::buf::BufMut;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;
use rustfft::FftPlanner;
use std::thread;
use std::thread::JoinHandle;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time;
use std::cmp;


use crate::audio::AudioStream;

/// FFT of audio input 
pub struct AudioTransformer {
    handle: Option<JoinHandle<()>>,
    source: String,
    bins: usize,
    killed: Arc<AtomicBool>,
}

impl AudioTransformer {
    pub fn new(source: String, bins: usize) -> AudioTransformer {
        let transformer = AudioTransformer {
            handle: None,
            source,
            bins,
            killed: Arc::new(AtomicBool::from(false)),
        };

        transformer
    }

    pub fn start(&mut self) {
        let bins = self.bins;
        let source = self.source.clone();
        let killed = self.killed.clone();

        self.handle = Some(thread::spawn(move || {
            let audio = AudioStream::new("led speakers".to_string(), source);
            let mut planner = FftPlanner::new();
            let fft = planner.plan_fft_forward(bins);

            let buffer = audio.buffer.unwrap();

            let mut left: Vec<Complex<f32>> = vec![Zero::zero(); bins];
            let mut right: Vec<Complex<f32>> = vec![Zero::zero(); bins];

            let byte_rate = *(*audio.rate).lock().unwrap();
            eprintln!("rate {:?}", byte_rate);
            let target_bytes_per_frame = (byte_rate / 60) as usize;
            let fft_byte_len: usize = bins * 4;
            let mut stream_buf =
                BytesMut::with_capacity(target_bytes_per_frame * 6 + 32 * bins);
            let mut audio_buf: Vec<i16> = vec![0; bins * 2];

            let lin_fft_res = (byte_rate / 2) as f64 / (bins / 2) as f64; 
            let norm = 1.0 / (i16::max_value() as f32);

            while !killed.load(Ordering::Relaxed) {
                let available = buffer.available();

                if available < (target_bytes_per_frame * 2) {
                    thread::sleep(time::Duration::from_micros(500));
                    continue;
                }

                let mut to_consume = if available > target_bytes_per_frame * 2 {
                    target_bytes_per_frame
                } else {
                    continue;
                };
                to_consume -= to_consume % 4;

                let fresh_bytes: &[u8] = &buffer.read(to_consume);
                stream_buf.reserve(to_consume);
                stream_buf.put(fresh_bytes);
                let fft_available = stream_buf.len();
                if fft_available > fft_byte_len {
                    stream_buf.advance(fft_available - fft_byte_len);
                }

                if stream_buf.len() < fft_byte_len {
                    continue;
                }

                {
                    Le::read_i16_into(&stream_buf.clone().split_to(fft_byte_len), &mut audio_buf);
                    let mut lc = left.iter_mut();
                    let mut rc = right.iter_mut();
                    for sample in audio_buf.chunks_exact(2) {
                        let normed = sample[1] as f32 * norm;
                        *lc.next().unwrap() = Complex::new(normed, 0.0);
                        let normed = sample[0] as f32 * norm;
                        *rc.next().unwrap() = Complex::new(normed, 0.0);
                    }
                }

                fft.process(&mut left);
                eprintln!("{:?}", left);
                // let left_buffer = fft_bufpool.chunk(output.clone().into_iter()).unwrap();
                fft.process(&mut right);
                // let right_buffer = fft_bufpool.chunk(output.clone().into_iter()).unwrap();
            }
        }));
    }

    fn frequency_magnitudes(
        mut input: Vec<f32>, 
        mut band_max: Vec<f32>, 
        mut prev_bands: Vec<f32>, 
        mut band_peaks: Vec<f32>,
        mut falloff: Vec<f32>,
        lower_cutoff: f32, 
        upper_cutoff: f32, 
        total_bands: usize, 
        rate: u32, 
        bins: u32, 
        monstercat: f32,
        decay: f32) 
        -> Vec<f32> {
        let mut cutoff_frequencies: Vec<f32> = vec![0.0; total_bands];
        let mut lower_cutoff_freq: Vec<f32> = vec![0.0; total_bands];
        let mut upper_cutoff_freq: Vec<f32> = vec![0.0; total_bands];
        let mut bands: Vec<f32> = vec![0.0; total_bands];

        let frequency_constant = (lower_cutoff / upper_cutoff).log(10.0) / (1.0 / total_bands as f32 - 1.0);

        // compute cutoff frequencies
        for n in 0..total_bands {
            let distribution_coefficient = -frequency_constant + ((n + 1) as f32 / (total_bands as f32)) * frequency_constant;
            cutoff_frequencies[n] = upper_cutoff * (10 as f32).powf(distribution_coefficient);
            let frequency = cutoff_frequencies[n] / (rate as f32 / 2.0);
            lower_cutoff_freq[n] = (frequency * bins as f32 / 4.0).floor();

            if n > 0 {
                if lower_cutoff_freq[n] <= lower_cutoff_freq[n - 1] {
                    lower_cutoff_freq[n] = lower_cutoff_freq[n - 1] + 1.0;
                }
                upper_cutoff_freq[n - 1] = lower_cutoff_freq[n - 1];
            }
        }

        // frequency bands
        for n in 0..total_bands {
            let mut frequency_magnitude: f32 = 0.0;
            let mut cutoff_freq: usize = lower_cutoff_freq[n] as usize;
            while cutoff_freq <= upper_cutoff_freq[n] as usize && (cutoff_freq as usize) < total_bands {
                cutoff_freq += 1;
                frequency_magnitude += input[cutoff_freq];
            }

            bands[n] = frequency_magnitude / (upper_cutoff_freq[n] - lower_cutoff_freq[n] + 1.0);
            bands[n] *= (2.0 + (n as f32)).log(2.0) * (100.0 / (total_bands as f32));
            bands[n] = bands[n].sqrt();
        }

        // smoothing
        Self::smooth(&mut bands, total_bands, monstercat);

        // scaling
        let mut max_val: f32 = 0.0;
        let mut sum: f32 = 0.0;

        for n in 0..total_bands {
            if bands[n] > max_val {
                max_val = bands[n];
            }
        }

        let mut prev = band_max[0];
        band_max[0] = max_val;
        for n in 0..total_bands {
            let mut tmp = band_max[n + 1];
            band_max[n + 1] = prev;
            prev = tmp;
        }

        for n in 0..total_bands {
            sum += band_max[n];
        }

        let moving_average = sum / total_bands as f32;
        let mut sqrt_sum = 0.0;
        for n in 0..total_bands {
            sqrt_sum += band_max[n] * band_max[n];
        }
        let std_dev = ((sqrt_sum / total_bands as f32) - moving_average.powf(2.0)).sqrt();
        let max_height = (moving_average + (2.0 * std_dev)).max(1.0);

        for n in 0..total_bands {
            bands[n] = ((bands[n] / max_height) * 100.0 - 1.0).min(100.0 - 1.0);
        }

        // falloff
        for n in 0..total_bands {
            if bands[n] < prev_bands[n] {
                bands[n] = band_peaks[n] - falloff[n] * decay;
                if bands[n] < 0.0 {
                    bands[n] = 0.0;
                }

                falloff[n] += 1.0;
            }
            else {
                band_peaks[n] = bands[n];
                falloff[n] = 0.0;
            }

            prev_bands[n] = bands[n];
        }

        return bands;
    }

    /// Apply monstercat filter to smooth input
    fn smooth(input: &mut Vec<f32>, total_bands: usize, monstercat: f32) {
        for z in 0..(total_bands as usize) {
            for m_y in (z - 1)..0 {
                let de = (z - m_y) as f32;
                input[m_y] = (input[z] / monstercat.powf(de)).max(input[m_y]);
            }

            for m_y in (z + 1)..(total_bands as usize) {
                let de = (m_y - z) as f32;
                input[m_y] = (input[z] / monstercat.powf(de)).max(input[m_y]);
            }
        }
    }
}