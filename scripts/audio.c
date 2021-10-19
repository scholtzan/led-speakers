#include <stdio.h>
#include <assert.h>
#include <pthread.h>
#include <stdint.h>
#include <string.h>
#include <pulse/pulseaudio.h>
#include "audio.h"
#include "config.h"
#include <fftw3.h>
#include <pulse/simple.h>
#include <math.h>

char found = 0;
pa_stream * gStream = NULL;
sem_t sem_audio;
pa_mainloop *m_pulseaudio_mainloop;
pthread_mutex_t lock;

void init_audio(audio_t *audio) {
    audio->fftw_input_right = fftwf_alloc_real(BUFFER_SIZE);
    audio->fftw_input_left = fftwf_alloc_real(BUFFER_SIZE);
    audio->fftw_output_right = fftwf_alloc_real(BUFFER_SIZE);
    audio->fftw_output_left = fftwf_alloc_real(BUFFER_SIZE);
    audio->fftw_plan_left = fftwf_plan_dft_r2c_1d(BUFFER_SIZE, audio->fftw_input_left, audio->fftw_output_left, FFTW_ESTIMATE);
    audio->fftw_plan_right = fftwf_plan_dft_r2c_1d(BUFFER_SIZE, audio->fftw_input_right, audio->fftw_output_right, FFTW_ESTIMATE);

    audio->format = FORMAT;
    audio->source = NULL;
    audio->terminate = false;
    audio->channels = CHANNELS;
    audio->rate = RATE;

    for (int n = 0; n < 256; n++) {
        audio->prev_bands[n] = 0;
        audio->falloff[n] = 0;
        audio->band_peaks[n] = 0.0;
        audio->bands[n] = 0;
        audio->band_max[n] = 0.0;
    }
    
    uint16_t i;
    for (i = 0; i < BUFFER_SIZE; i++)
        audio->fftw_input_right[i] = 0.0f;
        audio->fftw_input_left[i] = 0.0f;

    sem_init(&sem_audio, 0, 0);

    pa_mainloop_api *mainloop_api;
    pa_context *context;

    int ret;
    m_pulseaudio_mainloop = pa_mainloop_new();
    mainloop_api = pa_mainloop_get_api(m_pulseaudio_mainloop);
    context = pa_context_new(mainloop_api, "led speaker");
    assert(context);
    pa_context_connect(context, NULL, PA_CONTEXT_NOFLAGS, NULL);

    pa_context_set_state_callback(context, &context_state_cb, (void *)audio);

    if (!(ret = pa_mainloop_iterate(m_pulseaudio_mainloop, 0, &ret)))
    {
        printf("Could not open pulseaudio mainloop to "
               "find default device name: %d\n"
               "check if pulseaudio is running\n",
               ret);

        exit(0);
    }

    pa_mainloop_run(m_pulseaudio_mainloop, &ret);
}

void free_audio(audio_t *audio) {
    fftwf_free(audio->fftw_input_right);
    fftwf_free(audio->fftw_input_left);
    fftwf_free(audio->fftw_output_right);
    fftwf_free(audio->fftw_output_left);
    fftwf_destroy_plan(audio->fftw_plan_left);
    fftwf_destroy_plan(audio->fftw_plan_right);
    free(audio->source);
    sem_destroy(&sem_audio);
}

void monstercat_filter(int *bars, int number_of_bars, double monstercat) {
    int m_y, de;

    for (int z = 0; z < number_of_bars; z++) {
        for (m_y = z - 1; m_y >= 0; m_y--) {
            de = z - m_y;
            bars[m_y] = fmax(bars[z] / pow(monstercat, de), bars[m_y]);
        }
        for (m_y = z + 1; m_y < number_of_bars; m_y++) {
            de = m_y - z;
            bars[m_y] = fmax(bars[z] / pow(monstercat, de), bars[m_y]);
        }
    }
}


bool find_beats(audio_t *audio) {
    int bands[256], fftw_lower_cutoff[256], fftw_upper_cutoff[256];
    float cutoff_frequency[256];

    for (int n = 0; n < 256; n++) {
        bands[n] = 0;
        fftw_lower_cutoff[n] = 0;
        fftw_upper_cutoff[n] = 0;
        cutoff_frequency[n] = 0.0;
    }

    audio->fftw_plan_left = fftwf_plan_dft_r2c_1d(BUFFER_SIZE, audio->fftw_input_left, audio->fftw_output_left, FFTW_ESTIMATE);
    audio->fftw_plan_right = fftwf_plan_dft_r2c_1d(BUFFER_SIZE, audio->fftw_input_right, audio->fftw_output_right, FFTW_ESTIMATE);
    fftwf_execute(audio->fftw_plan_left);
    fftwf_execute(audio->fftw_plan_right);

    double frequency_constant = log10((float) LOWER_CUTOFF / (float) UPPER_CUTOFF) /
                                    (1 / ((float)BANDS + 1) - 1);

    // compute cutoff frequencies
    for (int n = 0; n < BANDS + 1; n++) {
        double bar_distribution_coefficient = frequency_constant * (-1);
        bar_distribution_coefficient +=
                    ((float)n + 1) / ((float)BANDS + 1) * frequency_constant;
        cutoff_frequency[n] = UPPER_CUTOFF * pow(10, bar_distribution_coefficient);
        float frequency = cutoff_frequency[n] / (audio->rate / 2);
        fftw_lower_cutoff[n] = ((int) floor(frequency * BUFFER_SIZE / 4));

        if (n > 0) {
            if (fftw_lower_cutoff[n] <= fftw_lower_cutoff[n - 1]) {
                fftw_lower_cutoff[n] = fftw_lower_cutoff[n - 1] + 1;
            }
            fftw_upper_cutoff[n - 1] = fftw_lower_cutoff[n - 1];
        }
    }

    // frequency bands
    for (int n = 0; n < BANDS + 1; n++) {
        double freq_magnitude = 0.0;
        for (int cutoff_freq = fftw_lower_cutoff[n];
            cutoff_freq <= fftw_upper_cutoff[n] && cutoff_freq < BUFFER_SIZE + 1;
            ++cutoff_freq) {
            freq_magnitude += sqrt(*audio->fftw_output_left[cutoff_freq] * *audio->fftw_output_left[cutoff_freq] +
                *audio->fftw_output_right[cutoff_freq] * *audio->fftw_output_right[cutoff_freq]);
        }

        bands[n] = freq_magnitude / (fftw_upper_cutoff[n] - fftw_lower_cutoff[n] + 1);
        bands[n] *= log2(2 + n) * (100.0 / BANDS);
        bands[n] = pow(bands[n], 0.5);
    }

    // smoothing
    monstercat_filter(bands, BANDS, MONSTERCAT);

    // scaling
    double std_dev = 0.0;
    double moving_average = 0.0;
    int max_val = 0;
    int sum = 0;

    for (int n = 0; n < BANDS + 1; n++) {
        if (bands[n] > max_val) {
            max_val = bands[n];
        }
    }

    int prev_temp = audio->band_max[0];
    audio->band_max[0] = max_val;
    for (int n = 0; n < BANDS; n++) {
        int tmp = audio->band_max[n + 1];
        audio->band_max[n + 1] = prev_temp;
        prev_temp = tmp;
    }

    for (int n = 0; n < 256; n++) {
        sum += audio->band_max[n];
    }
    moving_average = (double)sum / 256.0;

    double squared_summation = 0.0;
    for (int n = 0; n < 256; n++) {
        squared_summation += audio->band_max[n] * audio->band_max[n];
    }
    std_dev = sqrt((squared_summation / 256.0) - pow(moving_average, 2));

    double max_height = moving_average + (2 * std_dev);
    max_height = fmax(max_height, 1.0);

    for (int n = 0; n < BANDS + 1; n++) {
        bands[n] = fmin(100 - 1, (int)((bands[n] / max_height) * 100 - 1));
    }

    // falloff
    bool senselow = true;

    for (int n = 0; n < BANDS + 1; n++) {
        if (bands[n] < audio->prev_bands[n]) {
            bands[n] = audio->band_peaks[n] - (GRAVITY * audio->falloff[n] * audio->falloff[n]);
            if (bands[n] < 0)
                bands[n] = 0;
            audio->falloff[n]++;
        } else {
            audio->band_peaks[n] = bands[n];
            audio->falloff[n] = 0;
        }

        audio->prev_bands[n] = bands[n];
    }

    for (int n = 0; n < BANDS + 1; n++) {
        audio->bands[n] = bands[n];
    }

    fftwf_destroy_plan(audio->fftw_plan_left);
    fftwf_destroy_plan(audio->fftw_plan_right);
    return true;
}

void context_state_cb(pa_context* context, void* mainloop) {
    pa_context_state_t state = pa_context_get_state(context);
    if( state == PA_CONTEXT_READY){
        printf( "Pulseaudio connection ready...");
        pa_operation * o = pa_context_get_sink_info_list(context, sink_info_cb, mainloop);
        pa_operation_unref(o);
    }
    else if( state == PA_CONTEXT_FAILED){
        printf( "Connection failed");    
    }
    else if( state == PA_CONTEXT_TERMINATED){
        printf( "Connection terminated");
    }
}

void sink_info_cb(pa_context *c, const pa_sink_info *l, int eol, void *userdata) {
    audio_t* audio = (audio_t *) userdata;
    if(audio->source != NULL)
        free(audio->source);
    audio->source = malloc(sizeof(char) * 1024);
    if(found !=1){
         fprintf(stderr, "index: %d\n", (*l).index, 30);
         fprintf(stderr, "name: %s\n", (*l).name, 30);
         fprintf(stderr, "description: %s\n", (*l).description, 30);
         if (strcmp((*l).name, SINK_NAME) == 0) {
            strcpy(audio->source, (*l).monitor_source_name);
            found = 1;
            printf( "Connected to: %s\n ", (*l).monitor_source_name);
            pa_context_disconnect(c);
            pa_context_unref(c);
            pa_mainloop_quit(m_pulseaudio_mainloop, 0);
            pa_mainloop_free(m_pulseaudio_mainloop);
         }
    }
}

void *pulse_input(void *data) 
{

    audio_t *audio = (audio_t *)data;
    uint16_t frames = CHUNK;
    const int channels = 2;
    int16_t buf[frames * channels];

    static const pa_sample_spec ss = {
        .format = PA_SAMPLE_S16LE, 
        .rate = RATE, 
        .channels = channels
        };
    audio->format = 16;

    const int frag_size = frames * channels * audio->format / 8 * 2;

    pa_buffer_attr pb = {
        .maxlength = (uint32_t)-1, // BUFSIZE * 2,
        .fragsize = frag_size
        };

    pa_simple *s = NULL;
    int error;

    if (!(s = pa_simple_new(NULL, "led speaker", PA_STREAM_RECORD, audio->source, "audio for led speaker", &ss,
                            NULL, &pb, &error))) 
    {
        fprintf(stderr, __FILE__ ": Could not open pulseaudio source: %s, %s. \
        To find a list of your pulseaudio sources run 'pacmd list-sources'\n",
                audio->source, pa_strerror(error));

        audio->terminate = true;
    }

    while (!audio->terminate) 
    {
        if (pa_simple_read(s, buf, sizeof(buf), &error) < 0) 
        {
            fprintf(stderr, __FILE__ ": pa_simple_read() failed: %s\n",
                    pa_strerror(error));
            audio->terminate = true;
        }

        pthread_mutex_lock(&lock);
        write_to_fftw_input_buffers(frames, buf, (audio_t *) data);
        pthread_mutex_unlock(&lock);
    }

    pa_simple_free(s);
    pthread_exit(NULL);
    return 0;
}

void reset_output_buffers(audio_t *data) {
    memset(data->fftw_input_right, 0, sizeof(double) * BUFFER_SIZE);
    memset(data->fftw_input_left, 0, sizeof(double) * BUFFER_SIZE);
}

int write_to_fftw_input_buffers(int16_t frames, int16_t buf[frames * 2], void *data) {
    if (frames == 0)
        return 0;
    audio_t *audio = (audio_t *)data;

    for (uint16_t n = BUFFER_SIZE; n > frames; n = n - frames) {
        for (uint16_t i = 1; i <= frames; i++) {
            audio->fftw_input_left[n - i] = audio->fftw_input_left[n - i - frames];
            if (audio->channels == 2)
                audio->fftw_input_right[n - i] = audio->fftw_input_right[n - i - frames];
        }
    }

    uint16_t n = frames - 1;
    for (uint16_t i = 0; i < frames; i++) {
        audio->fftw_input_left[n] = buf[i * 2];
        audio->fftw_input_right[n] = buf[i * 2 + 1];
        n--;
    }
    sem_post(&sem_audio);

    return 0;
}
