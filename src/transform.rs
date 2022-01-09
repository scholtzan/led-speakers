use byteorder::ByteOrder;
use byteorder::LittleEndian as Le;
use bytes::buf::BufMut;
use bytes::Buf;
use bytes::BytesMut;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;
use rustfft::FftPlanner;
use std::cmp;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time;

use crate::audio::AudioStream;

/// Audio stream transformed into frequency bands.
struct TransformedAudio {
    /// Previous max frequency values
    pub band_max: Vec<f32>,

    /// Frequency band values
    pub bands: Vec<f32>,

    /// Frequency band peaks;keep track of frequency max. while applying decay
    pub band_peaks: Vec<f32>,

    /// Falloff factor for each frequency band
    pub falloff: Vec<f32>,
}

impl TransformedAudio {
    /// Initializes a new transformed audio representation
    pub fn new(total_bands: usize, fft_len: usize) -> TransformedAudio {
        TransformedAudio {
            band_max: vec![0.0; fft_len],
            bands: vec![0.0; total_bands],
            band_peaks: vec![0.0; total_bands],
            falloff: vec![0.0; total_bands],
        }
    }
}

/// Transforms an audio stream to frequency bands.
pub struct AudioTransformer {
    /// Handle on the thread running the audio transformation
    handle: Option<JoinHandle<()>>,

    /// Name of the audio sink
    sink: String,

    /// Length of FFT
    fft_len: usize,

    /// Total number of frequency bands
    total_bands: usize,

    /// Upper cutoff frequency
    upper_cutoff: f32,

    /// Lower cutoff frequency
    lower_cutoff: f32,

    /// Factor for monstercat smoothing
    monstercat: f32,

    /// Frequency magnitude decay factor
    decay: f32,

    /// Size of the audio buffer
    buffer_size: usize,

    /// Whether the audio transformer is still running
    killed: Arc<AtomicBool>,

    /// Frequency bands for left channel
    pub left_bands: Arc<Mutex<Vec<f32>>>,

    /// Frequency bands for right channel
    pub right_bands: Arc<Mutex<Vec<f32>>>,
}

impl AudioTransformer {
    /// Instantiates a new `AudioTransformer`.
    ///
    /// # Arguments
    /// * `sink`: name of the audio sink audio is streamed from
    /// * `fft_len`: length of the FFT input
    /// * `total_bands`: total number of generated frequency bands
    /// * `lower_cutoff`: lower cutoff frequency
    /// * `upper_cutoff`: upper cutoff frequency
    /// * `monstercat`: factor for monstercat smoothing
    /// * `decay`: frequency magnitude decay factor
    /// * `buffer_size`: size of the audio buffer
    ///
    pub fn new(
        sink: String,
        fft_len: usize,
        total_bands: usize,
        lower_cutoff: f32,
        upper_cutoff: f32,
        monstercat: f32,
        decay: f32,
        buffer_size: usize,
    ) -> AudioTransformer {
        let transformer = AudioTransformer {
            handle: None,
            sink,
            fft_len,
            total_bands,
            upper_cutoff,
            lower_cutoff,
            monstercat,
            decay,
            buffer_size,
            killed: Arc::new(AtomicBool::from(false)),
            left_bands: Arc::new(Mutex::new(vec![0.0; total_bands])),
            right_bands: Arc::new(Mutex::new(vec![0.0; total_bands])),
        };

        transformer
    }

    /// Start transforming audio into frequency bands.
    pub fn start(&mut self) {
        // make fields available in thread
        let fft_len = self.fft_len;
        let total_bands = self.total_bands;
        let lower_cutoff = self.lower_cutoff;
        let upper_cutoff = self.upper_cutoff;
        let monstercat = self.monstercat;
        let decay = self.decay;
        let sink = self.sink.clone();
        let killed = self.killed.clone();
        let right_bands = self.right_bands.clone();
        let left_bands = self.left_bands.clone();
        let buffer_size = self.buffer_size;

        // transform audio in separate thread
        self.handle = Some(thread::spawn(move || {
            // initialize audio stream
            let audio = AudioStream::new("led speakers".to_string(), sink, buffer_size);

            // instance to compute forward FFTs
            let mut planner = FftPlanner::new();
            let fft = planner.plan_fft_forward(fft_len);

            // audio buffer
            let buffer = audio.buffer.unwrap();

            // audio inputs for FFT for left and right channel
            let mut left: Vec<Complex<f32>> = vec![Zero::zero(); fft_len];
            let mut right: Vec<Complex<f32>> = vec![Zero::zero(); fft_len];

            // transformed audio for each channel
            let mut left_transformed = TransformedAudio::new(total_bands, fft_len);
            let mut right_transformed = TransformedAudio::new(total_bands, fft_len);

            // get bytes per audio frame
            let byte_rate = *(*audio.rate).lock().unwrap();
            let target_bytes_per_frame = (byte_rate / 60) as usize;

            // number of bytes required as FFT input
            let fft_byte_len: usize = fft_len * 4;

            // data gets written into this temporary buffer from the audio buffer
            let mut stream_buf = BytesMut::with_capacity(target_bytes_per_frame * 6 + 32 * fft_len);

            // input buffer for FFT
            let mut fft_input_buffer: Vec<i16> = vec![0; fft_len * 2];

            let lin_fft_res = ((byte_rate / 2) as f64) / ((fft_len / 2) as f64);

            // factor used to normalize input samples
            let norm = 1.0 / (i16::max_value() as f32);

            while !killed.load(Ordering::Relaxed) {
                // check if there is enough data in the audio buffer
                let available = buffer.available();
                if available < (target_bytes_per_frame * 2) {
                    thread::sleep(time::Duration::from_micros(500));
                    continue;
                }

                // check if enough data is available for FFT in audio buffer
                let mut to_consume = if available > target_bytes_per_frame * 2 {
                    target_bytes_per_frame
                } else {
                    continue;
                };
                to_consume -= to_consume % 4;

                // read from audio buffer and write to stream buffer
                let fresh_bytes: &[u8] = &buffer.read(to_consume);
                stream_buf.reserve(to_consume);
                stream_buf.put(fresh_bytes);

                // check if enough data for FFT is available; otherwise wait until next iteration
                let fft_available = stream_buf.len();
                if fft_available > fft_byte_len {
                    stream_buf.advance(fft_available - fft_byte_len);
                }
                if stream_buf.len() < fft_byte_len {
                    continue;
                }

                {
                    // if enough data is in the stream buffer, copy data to FFT input buffer
                    Le::read_i16_into(
                        &stream_buf.clone().split_to(fft_byte_len),
                        &mut fft_input_buffer,
                    );

                    // extract left and right channel data from FFT input buffer
                    let mut left_channel_input = left.iter_mut();
                    let mut right_channel_input = right.iter_mut();

                    // normalize left and right channel samples
                    for sample in fft_input_buffer.chunks_exact(2) {
                        let normed = (sample[1] as f32) * norm;
                        *left_channel_input.next().unwrap() = Complex::new(normed, 0.0);
                        let normed = (sample[0] as f32) * norm;
                        *right_channel_input.next().unwrap() = Complex::new(normed, 0.0);
                    }
                }

                // FFT for left channel
                fft.process(&mut left);

                // convert complex values to real values
                let left_real: Vec<f32> = left
                    .iter()
                    .map(|c| (c.im.powf(2.0) + c.re.powf(2.0)).sqrt())
                    .collect();

                // determine frequency magnitudes for left channel
                *(left_bands.lock().unwrap()) = Self::frequency_magnitudes(
                    left_real,
                    &mut left_transformed,
                    lower_cutoff,
                    upper_cutoff,
                    total_bands,
                    byte_rate,
                    fft_len,
                    monstercat,
                    decay,
                );

                // FFT for right channel
                fft.process(&mut right);

                // convert complex values to real values
                let right_real: Vec<f32> = right
                    .iter()
                    .map(|c| (c.im.powf(2.0) + c.re.powf(2.0)).sqrt())
                    .collect();

                // determine frequency magnitudes for right channel
                *(right_bands.lock().unwrap()) = Self::frequency_magnitudes(
                    right_real,
                    &mut right_transformed,
                    lower_cutoff,
                    upper_cutoff,
                    total_bands,
                    byte_rate,
                    fft_len,
                    monstercat,
                    decay,
                );
            }
        }));
    }

    /// Compute frequency bands and magnitudes.
    /// todo: re-binning to log scale
    fn frequency_magnitudes(
        input: Vec<f32>,
        transformed_audio: &mut TransformedAudio,
        lower_cutoff: f32,
        upper_cutoff: f32,
        total_bands: usize,
        rate: u32,
        fft_len: usize,
        monstercat: f32,
        decay: f32,
    ) -> Vec<f32> {
        // contains indices for each band indicating range of FFT buckets that need to be merged
        let mut lower_cutoff_freq: Vec<usize> = vec![0; total_bands];
        let mut upper_cutoff_freq: Vec<usize> = vec![0; total_bands];

        // frequency magnitudes for each band
        let mut bands: Vec<f32> = vec![0.0; total_bands];

        // this constant is used to distribute frequency bands into the different buckets;
        // using log scale for bucketing since it matches more closely perception of sound spectrum
        let frequency_constant =
            (lower_cutoff / upper_cutoff).log(10.0) / (1.0 / total_bands as f32 - 1.0);

        // compute cutoff frequencies
        for n in 0..total_bands {
            // compute upper cutoff frequency for bucket
            let distribution_coefficient =
                -frequency_constant + ((n + 1) as f32 / (total_bands as f32)) * frequency_constant;
            let cutoff_frequency = upper_cutoff * (10 as f32).powf(distribution_coefficient);

            // for re-binning it is necessary to know what bins from the FFT result need to be merged
            // compute index of lowest FFT bin merging starts from for this band
            let frequency = cutoff_frequency / (rate as f32 / 2.0);
            lower_cutoff_freq[n] = (frequency * fft_len as f32 / 2.0).floor() as usize; // todo: /4?

            // assign FFT indices for uppper and lower frequency ranges
            if n > 0 {
                if lower_cutoff_freq[n] <= lower_cutoff_freq[n - 1] {
                    lower_cutoff_freq[n] = lower_cutoff_freq[n - 1] + 1;
                }
                upper_cutoff_freq[n - 1] = lower_cutoff_freq[n] - 1;
            }
        }

        // compute frequency magnitures for each bands
        for n in 0..total_bands {
            let mut frequency_magnitude: f32 = 0.0;
            let mut cutoff_freq = lower_cutoff_freq[n];

            // sum up all FFT bucket frequency magnitudes for band
            while cutoff_freq <= upper_cutoff_freq[n] && cutoff_freq < total_bands {
                cutoff_freq += 1;
                frequency_magnitude += input[cutoff_freq];
            }

            // compute frequency magnitude average
            bands[n] =
                frequency_magnitude / ((upper_cutoff_freq[n] - lower_cutoff_freq[n] + 1) as f32);
            // different weighting of frequencies; higher freqencies are more prominent
            bands[n] *= (2.0 + (n as f32)).log(2.0) * (100.0 / (total_bands as f32));
            bands[n] = bands[n].sqrt();
        }

        // smoothing
        Self::smooth(&mut bands, total_bands, monstercat);

        // scaling

        // determine maximum frequency magnitude
        let mut max_magnitude: f32 = 0.0;
        for n in 0..total_bands {
            if bands[n] > max_magnitude {
                max_magnitude = bands[n];
            }
        }

        // a record of previous max. frequency magnitudes is kept in `band_max`;
        // the most recent maximum frequency magnitudes is stored at index=0,
        // in each iteration update recent maximum frequency magnitudes
        let mut prev_max_magnitude = transformed_audio.band_max[0]; // todo: increase buffer size
        transformed_audio.band_max[0] = max_magnitude;
        for n in 0..(fft_len - 1) {
            let mut tmp = transformed_audio.band_max[n + 1];
            transformed_audio.band_max[n + 1] = prev_max_magnitude;
            prev_max_magnitude = tmp;
        }

        // total of max. frequency magnitudes
        let mut sum: f32 = 0.0;
        for n in 0..fft_len {
            sum += transformed_audio.band_max[n];
        }

        // compute the moving average of the max. frequency magnitudes
        let moving_average = sum / total_bands as f32;
        // compute squared sum of moving average
        let mut sqrt_sum = 0.0;
        for n in 0..fft_len {
            sqrt_sum += transformed_audio.band_max[n] * transformed_audio.band_max[n];
        }
        //compute std dev
        let std_dev = ((sqrt_sum / total_bands as f32) - moving_average.powf(2.0)).sqrt();
        // compute maximum allowed frequency magnitude
        let max_freq = (moving_average + (2.0 * std_dev)).max(1.0);

        // scale magnitudes to range of 0 to 100
        for n in 0..total_bands {
            bands[n] = ((bands[n] / max_freq) * 100.0 - 1.0).min(100.0 - 1.0);
        }

        // falloff
        for n in 0..total_bands {
            // apply a decay rate to each frequency magnitude
            if bands[n] < transformed_audio.bands[n] {
                // recent magnitude has not increased, apply decay rate
                bands[n] = transformed_audio.band_peaks[n] - transformed_audio.falloff[n] * decay;
                if bands[n] < 0.0 {
                    bands[n] = 0.0;
                }

                // the decay rate increases with each iteration if magnitude does not increase
                transformed_audio.falloff[n] += 1.0;
            } else {
                // recent magnitude is larger than previous one; don't apply decay rate
                transformed_audio.band_peaks[n] = bands[n];
                transformed_audio.falloff[n] = 0.0;
            }

            transformed_audio.bands[n] = bands[n];
        }

        return bands;
    }

    /// Apply monstercat filter to smooth frequency magnitudes
    fn smooth(input: &mut Vec<f32>, total_bands: usize, monstercat: f32) {
        for band in 0..total_bands {
            // look at previous bands and adjust if they deviate too much from current frequency magnitude
            for prev_band in (band - 1)..0 {
                let de = (band - prev_band) as f32;
                input[prev_band] = (input[band] / monstercat.powf(de)).max(input[prev_band]);
            }

            // look at following bands and adjust if they deviate too much from current frequency magnitude
            for following_band in (band + 1)..total_bands {
                let de = (following_band - band) as f32;
                input[following_band] =
                    (input[band] / monstercat.powf(de)).max(input[following_band]);
            }
        }
    }
}
