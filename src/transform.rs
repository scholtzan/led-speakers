use rustfft::FftPlanner;
use std::thread;
use std::thread::JoinHandle;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};


use crate::audio::AudioStream;

/// FFT of audio input 
pub struct AudioTransformer {
    handle: Option<JoinHandle<()>>,
    source: String,
    bins: u32,
    killed: Arc<AtomicBool>,
}

impl AudioTransformer {
    pub fn new(source: String, bins: u32) -> AudioTransformer {
        let transformer = AudioTransformer {
            handle: None,
            source,
            bins,
            killed: Arc::new(AtomicBool::from(false)),
        };

        transformer
    }

    fn start(&mut self) {
        let bins = self.bins;
        let source = self.source.clone();
        let killed = self.killed.clone();

        self.handle = Some(thread::spawn(move || {
            let audio = AudioStream::new("led speakers".to_string(), source);
            // let mut planner = FftPlanner::new();

            // let byte_rate = audio.
            // let target_bytes_per_frame = (byte_rate / 60) as usize;

            // while !killed.load(Ordering::Relaxed) {
            //     let available = audio.buffer.available();

            //     if available < () {
            //         thread::sleep(time::Duration::from_micros(500));
            //         continue;
            //     }
            // }
        }));


        // setup FFTW
        // create new thread here
        // move audio stream creation in to thread
    }
}