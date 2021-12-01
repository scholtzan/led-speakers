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

            let byte_rate = audio.source.borrow().clone().unwrap().rate;
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


        // setup FFTW
        // create new thread here
        // move audio stream creation in to thread
    }
}