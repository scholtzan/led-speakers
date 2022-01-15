use led_speakers::transform::{AudioTransformer, TransformedAudio};

#[test]
fn test_cutoff_frequencies() {
    let total_bands = 3;
    let lower_cutoff = 50.0;
    let upper_cutoff = 10000.0;
    let rate = 48000;
    let fft_len = 3000;
    let (lower, upper) = AudioTransformer::cutoff_frequencies(
        total_bands,
        lower_cutoff,
        upper_cutoff,
        rate,
        fft_len,
    );
    assert_eq!(lower, vec![3, 23, 171]);
    assert_eq!(upper, vec![22, 170, 3000]);
}

#[test]
fn test_magnitudes() {
    let mut bands = vec![0.0, 0.0];
    let upper_cutoff_freq = vec![2, 4];
    let lower_cutoff_freq = vec![0, 3];
    let input = vec![1000.0, 1000.0, 1000.0, 1000.0, 1000.0];
    AudioTransformer::magnitudes(&mut bands, input, &lower_cutoff_freq, &upper_cutoff_freq);
    assert_eq!(bands, vec![223.6068, 281.51044]);

    let input = vec![1000.0, 1000.0, 0.0, 0.0, 0.0];
    AudioTransformer::magnitudes(&mut bands, input, &lower_cutoff_freq, &upper_cutoff_freq);
    assert_eq!(bands, vec![182.57419, 0.0]);
}

#[test]
fn test_smooth() {
    let mut input = vec![1.0, 100.0, 1.0];
    AudioTransformer::smooth(&mut input, 2.0);
    assert_eq!(input, vec![50.0, 100.0, 50.0]);
}

#[test]
fn test_scale() {
    let mut bands = vec![100.0, 100.0, 1.0];
    let mut transformed_audio = TransformedAudio::new(bands.len(), 1);

    AudioTransformer::scale(&mut bands, &mut transformed_audio);
    assert_eq!(bands, vec![100.0, 100.0, 1.0]);
    assert_eq!(transformed_audio.band_max, vec![100.0]);

    AudioTransformer::scale(&mut bands, &mut transformed_audio);
    assert_eq!(bands, vec![100.0, 100.0, 1.0]);
    assert_eq!(transformed_audio.band_max, vec![100.0]);
}

#[test]
fn test_scale_complex() {
    let mut bands = vec![100.0, 100.0];
    let mut transformed_audio = TransformedAudio::new(bands.len(), 2);

    AudioTransformer::scale(&mut bands, &mut transformed_audio);
    assert_eq!(transformed_audio.band_max, vec![100.0, 0.0]);

    let mut bands = vec![50.0, 10.0];
    AudioTransformer::scale(&mut bands, &mut transformed_audio);
    assert_eq!(bands, vec![40.0, 8.0]);
    assert_eq!(transformed_audio.band_max, vec![50.0, 100.0]);
}

#[test]
fn test_falloff() {
    let mut bands = vec![100.0, 100.0, 1.0];
    let mut transformed_audio = TransformedAudio::new(bands.len(), 50);
    let decay = 10.0;

    AudioTransformer::falloff(&mut bands, &mut transformed_audio, decay);
    assert_eq!(bands, vec![100.0, 100.0, 1.0]);
    assert_eq!(transformed_audio.band_peaks, vec![100.0, 100.0, 1.0]);
    assert_eq!(transformed_audio.bands, vec![100.0, 100.0, 1.0]);
    assert_eq!(transformed_audio.falloff, vec![1.0, 1.0, 1.0]);

    let mut bands_2 = vec![90.0, 90.0, 1.0];
    AudioTransformer::falloff(&mut bands_2, &mut transformed_audio, decay);
    assert_eq!(bands_2, vec![90.0, 90.0, 1.0]);
    assert_eq!(transformed_audio.band_peaks, vec![100.0, 100.0, 1.0]);
    assert_eq!(transformed_audio.bands, vec![90.0, 90.0, 1.0]);
    assert_eq!(transformed_audio.falloff, vec![2.0, 2.0, 1.0]);
}
