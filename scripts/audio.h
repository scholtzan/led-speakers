#ifndef _AUDIO_H_
#define _AUDIO_H_

#include <stdbool.h>
#include <fftw3.h>
#include <stdint.h>
#include <semaphore.h>
#include <pulse/pulseaudio.h>


typedef struct {
	char *source;
	bool terminate;
	int format;
	unsigned int rate;
    unsigned int channels;

    float *fftw_input_right;
    float *fftw_input_left;
    fftwf_complex *fftw_output_right;
    fftwf_complex *fftw_output_left;
    fftwf_plan fftw_plan_left;
    fftwf_plan fftw_plan_right;

    int bands[256];
    int prev_bands[256];
    int band_peaks[256];
    int falloff[256];
    int band_max[256];
} audio_t;

extern sem_t sem_audio;

void init_audio(audio_t *audio);
void free_audio(audio_t *audio);
bool find_beats(audio_t *audio);
void reset_output_buffers(audio_t *data);
int write_to_fftw_input_buffers(int16_t frames, int16_t buf[frames * 2], void *data);
void *pulse_input(void *data);
void sink_info_cb(pa_context *c, const pa_sink_info *l, int eol, void *userdata);
void context_state_cb(pa_context* context, void* mainloop);
void monstercat_filter(int *bars, int number_of_bars, double monstercat);

#endif