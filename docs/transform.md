# Audio Processing and Transformation

The `AudioTransformer` initializes a new audio stream

## FFT

To get frequency information about the audio input (dividing into frequency components). Frequency components are sinus oscillations with certain amplitude an dphase. 
Sampling rate = average number of samples obtained in one second
Maximum frequency that can be observed = sampling rate / 2
Measurement duration = block length = number of samples to analyse


convert fft into smaller number of buckets reperesnting average magnitude of frequencies they represent
log magnitude instead of linear frequency


averaging spectra
* exponential mean
    * weighting is inversely propotional to age of result (oldest measurement taken into account least)

frequency magnitutes gets determined frequencies as input after FFT


frequency constant ~= 2.4

distribution coefficient = [-frequency constant; 0]

cutoff frequencies = [0; upper cutoff]

frequency = [0; 1]


upper cutoff = lower cutoff



band_frequency_magnitude = sqrt(avg_magnitude * (2 + n))




resampling/rebinning a spectrum
https://dsp.stackexchange.com/questions/129/how-can-i-compute-a-log-spaced-power-spectrum/2098#2098


The log scale more closely matches our perception of the sound spectrum.

Our perception makes us feel that C1 is the same distance from C2 as C2 is from C3 (they are each an octave apart, the same distance, right?). But their actual frequencies are doubled, rather than going up a fixed amount. There are many scientific reasons for this, but basically our ears respond 'logarithmically' to both level (volume) and frequency.

A logarithmic scale means that each octave occupies the same 'width' across the spectrum. This makes it easier to judge certain things. On a linear scale, though the frequencies are even distances apart, if you were to plot the octave band limits, you would see them getting further apart as you go along the axis. It is confusing stuff, i know, but basically it all boils down to the fact that most properties of sound are analysed logarithmically, unless you are pinpointing a specific frequency.
