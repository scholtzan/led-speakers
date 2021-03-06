use byteorder::ByteOrder;
use byteorder::LittleEndian as Le;
use bytes::buf::BufMut;
use bytes::Buf;
use bytes::BytesMut;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;
use rustfft::FftPlanner;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time;

use crate::audio::AudioStream;
use crate::settings::TransformerSettings;

/// Audio stream transformed into frequency bands.
pub struct TransformedAudio {
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
            band_max: vec![0.0; fft_len], // reusing fft_len since buffer size should be larger for good moving average
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

    settings: TransformerSettings,

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
    /// * `settings`: transformer settings
    ///
    pub fn new(settings: TransformerSettings) -> AudioTransformer {
        let transformer = AudioTransformer {
            handle: None,
            settings: settings.clone(),
            killed: Arc::new(AtomicBool::from(false)),
            left_bands: Arc::new(Mutex::new(vec![0.0; settings.total_bands])),
            right_bands: Arc::new(Mutex::new(vec![0.0; settings.total_bands])),
        };

        transformer
    }

    /// Start transforming audio into frequency bands.
    pub fn start(&mut self) {
        // make fields available in thread
        let settings = self.settings.clone();
        let killed = self.killed.clone();
        let right_bands = self.right_bands.clone();
        let left_bands = self.left_bands.clone();

        // transform audio in separate thread
        self.handle = Some(thread::spawn(move || {
            // initialize audio stream
            let audio = AudioStream::new(
                "led speakers".to_string(),
                settings.sink.clone(),
                settings.buffer_size,
            );

            // instance to compute forward FFTs
            let mut planner = FftPlanner::new();
            let fft = planner.plan_fft_forward(settings.fft_len);

            // audio buffer
            let buffer = audio.buffer.unwrap();

            // audio inputs for FFT for left and right channel
            let mut left: Vec<Complex<f32>> = vec![Zero::zero(); settings.fft_len];
            let mut right: Vec<Complex<f32>> = vec![Zero::zero(); settings.fft_len];

            // transformed audio for each channel
            let mut left_transformed =
                TransformedAudio::new(settings.total_bands, settings.fft_len);
            let mut right_transformed =
                TransformedAudio::new(settings.total_bands, settings.fft_len);

            // get bytes per audio frame
            let byte_rate = *(*audio.rate).lock().unwrap();
            let target_bytes_per_frame = (byte_rate / 60) as usize;

            // number of bytes required as FFT input
            let fft_byte_len: usize = settings.fft_len * 4;

            // data gets written into this temporary buffer from the audio buffer
            let mut stream_buf =
                BytesMut::with_capacity(target_bytes_per_frame * 6 + 32 * settings.fft_len);

            // input buffer for FFT
            let mut fft_input_buffer: Vec<i16> = vec![0; settings.fft_len * 2];

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
                    settings.lower_cutoff,
                    settings.upper_cutoff,
                    settings.total_bands,
                    byte_rate,
                    settings.fft_len,
                    settings.monstercat,
                    settings.decay,
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
                    settings.lower_cutoff,
                    settings.upper_cutoff,
                    settings.total_bands,
                    byte_rate,
                    settings.fft_len,
                    settings.monstercat,
                    settings.decay,
                );
            }
        }));
    }

    /// Restart the transformer.
    ///
    /// Required when applying new settings.
    ///
    pub fn update_settings(&mut self, settings: TransformerSettings) {
        self.killed.swap(true, Ordering::Relaxed);
        self.settings = settings.clone();
        *self.left_bands.lock().unwrap() = vec![0.0; settings.total_bands];
        *self.right_bands.lock().unwrap() = vec![0.0; settings.total_bands];
        self.handle = None;
        self.killed.swap(false, Ordering::Relaxed);
        self.start();
    }

    /// Compute frequency bands and magnitudes.
    ///
    /// The sound spectrum is re-binned into log scale and a fewer number of bins.
    /// This means that buckets of the FFT will be merged together.
    ///
    pub fn frequency_magnitudes(
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
        // frequency magnitudes for each band
        let mut bands: Vec<f32> = vec![0.0; total_bands];

        let (lower_cutoff_freq, upper_cutoff_freq) =
            Self::cutoff_frequencies(total_bands, lower_cutoff, upper_cutoff, rate, fft_len);
        Self::magnitudes(&mut bands, input, &lower_cutoff_freq, &upper_cutoff_freq);
        Self::smooth(&mut bands, monstercat);
        Self::scale(&mut bands, transformed_audio);
        Self::falloff(&mut bands, transformed_audio, decay);

        return bands;
    }

    /// Compute lower and upper cutoff frequency indices.
    ///
    /// Indices will be used for determining which FFT buckets need to be merged for each band.
    ///
    pub fn cutoff_frequencies(
        total_bands: usize,
        lower_cutoff: f32,
        upper_cutoff: f32,
        rate: u32,
        fft_len: usize,
    ) -> (Vec<usize>, Vec<usize>) {
        // contains indices for each band indicating range of FFT buckets that need to be merged
        let mut lower_cutoff_freq: Vec<usize> = vec![0; total_bands];
        let mut upper_cutoff_freq: Vec<usize> = vec![0; total_bands];

        // this constant is used to distribute frequency bands into the different buckets;
        // using log scale for bucketing since it matches more closely perception of sound spectrum
        let frequency_constant =
            (lower_cutoff / upper_cutoff).log(2.0) / (1.0 / total_bands as f32 - 1.0);

        // compute cutoff frequencies
        for n in 0..total_bands {
            // compute upper cutoff frequency for bucket
            let distribution_coefficient = -frequency_constant
                + ((n + 1) as f32 / (total_bands as f32 + 1.0)) * frequency_constant;
            let cutoff_frequency = upper_cutoff * (2 as f32).powf(distribution_coefficient);

            // for re-binning it is necessary to know what bins from the FFT result need to be merged
            // compute index of lowest FFT bin merging starts from for this band
            let frequency = cutoff_frequency / (rate as f32 / 4.0);
            lower_cutoff_freq[n] = (frequency * fft_len as f32 / 2.0).floor() as usize;

            // assign FFT indices for uppper and lower frequency ranges
            if n > 0 {
                if lower_cutoff_freq[n] <= lower_cutoff_freq[n - 1] {
                    lower_cutoff_freq[n] = lower_cutoff_freq[n - 1] + 1;
                }
                upper_cutoff_freq[n - 1] = lower_cutoff_freq[n] - 1;
            }
        }

        upper_cutoff_freq[total_bands - 1] = fft_len;

        (lower_cutoff_freq, upper_cutoff_freq)
    }

    /// Computes frequency magnitudes for each band.
    pub fn magnitudes(
        bands: &mut Vec<f32>,
        input: Vec<f32>,
        lower_cutoff_freq: &Vec<usize>,
        upper_cutoff_freq: &Vec<usize>,
    ) {
        let fft_len = input.len();
        let total_bands = bands.len();
        for n in 0..total_bands {
            let mut frequency_magnitude: f32 = 0.0;
            let mut cutoff_freq = lower_cutoff_freq[n];

            // sum up all FFT bucket frequency magnitudes for band
            while cutoff_freq <= upper_cutoff_freq[n] && cutoff_freq < fft_len {
                frequency_magnitude += input[cutoff_freq];
                cutoff_freq += 1;
            }

            // compute frequency magnitude average
            bands[n] =
                frequency_magnitude / ((upper_cutoff_freq[n] - lower_cutoff_freq[n] + 1) as f32);
            // different weighting of frequencies; higher freqencies are more prominent
            bands[n] *= (2.0 + (n as f32)).log(2.0) * (100.0 / (total_bands as f32));
            bands[n] = bands[n].sqrt();
        }
    }

    /// Applies monstercat filter to smooth frequency magnitudes.
    pub fn smooth(bands: &mut Vec<f32>, monstercat: f32) {
        let total_bands = bands.len();
        for band in 1..total_bands {
            // look at previous bands and adjust if they deviate too much from current frequency magnitude
            for prev_band in (0..band).rev() {
                let de = (band - prev_band) as f32;
                bands[prev_band] = (bands[band] / monstercat.powf(de)).max(bands[prev_band]);
            }

            // look at following bands and adjust if they deviate too much from current frequency magnitude
            for following_band in (band + 1)..total_bands {
                let de = (following_band - band) as f32;
                bands[following_band] =
                    (bands[band] / monstercat.powf(de)).max(bands[following_band]);
            }
        }

        // round frequency numbers to reduce noise
        for i in 0..total_bands {
            bands[i] = (bands[i] as usize) as f32;
        }
    }

    /// Scales frequency magnitudes to values between 0 and 100.
    ///
    /// Frequency magnitudes are re-scaled based on previous maximum magnitude values.
    ///
    pub fn scale(bands: &mut Vec<f32>, transformed_audio: &mut TransformedAudio) {
        let total_bands = bands.len();

        // determine maximum frequency magnitude
        let mut max_magnitude: f32 = 0.0;
        for n in 0..total_bands {
            if bands[n] > max_magnitude {
                max_magnitude = bands[n];
            }
        }

        let band_max_len = transformed_audio.band_max.len();

        // a record of previous max. frequency magnitudes is kept in `band_max`;
        // the most recent maximum frequency magnitudes is stored at index=0,
        // in each iteration update recent maximum frequency magnitudes
        let mut prev_max_magnitude = transformed_audio.band_max[0]; // todo: increase buffer size
        transformed_audio.band_max[0] = max_magnitude;
        for n in 0..(band_max_len - 1) {
            let tmp = transformed_audio.band_max[n + 1];
            transformed_audio.band_max[n + 1] = prev_max_magnitude;
            prev_max_magnitude = tmp;
        }

        // total of max. frequency magnitudes
        let mut sum: f32 = 0.0;
        for n in 0..band_max_len {
            sum += transformed_audio.band_max[n];
        }

        // compute the moving average of the max. frequency magnitudes
        let moving_average = sum / band_max_len as f32;
        // compute squared sum of moving average
        let mut sqrt_sum = 0.0;
        for n in 0..band_max_len {
            sqrt_sum += transformed_audio.band_max[n] * transformed_audio.band_max[n];
        }
        // compute std dev
        let std_dev = ((sqrt_sum / band_max_len as f32) - moving_average.powf(2.0)).sqrt();
        // compute maximum allowed frequency magnitude
        let max_freq = (moving_average + (2.0 * std_dev)).max(1.0);

        // scale magnitudes to range of 0 to 100
        for n in 0..total_bands {
            bands[n] = ((bands[n] / max_freq) * 100.0).min(100.0);
        }
    }

    /// Applies decay rate to frequency magnitudes.
    pub fn falloff(bands: &mut Vec<f32>, transformed_audio: &mut TransformedAudio, decay: f32) {
        let total_bands = bands.len();

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
                transformed_audio.falloff[n] = 1.0;
            }

            transformed_audio.bands[n] = bands[n];
        }
    }
}
