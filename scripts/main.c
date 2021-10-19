#include "audio.h"
#include "led.h"
#include "config.h"
#include "viz/intensity.h"
#include <stdio.h>
#include <stdint.h>
#include <pthread.h>
#include <assert.h>
#include <stdbool.h>
#include <signal.h>
#include <semaphore.h>

#include <math.h>
#include <fftw3.h>

volatile bool running = true;
static void ctrl_c_handler(int signum)
{
    (void)(signum);
    running = false;
}

static void setup_handlers(void)
{
    struct sigaction sa = 
    {
        .sa_handler = ctrl_c_handler
    };
    sigaction(SIGINT, &sa, NULL);
    sigaction(SIGTERM, &sa, NULL);
}

void *process_audio(void *data) {
    audio_t *audio = (audio_t *) data;

    while(running)
    {
        sem_wait(&sem_audio);
        if(find_beats(audio))
        {
            update_viz(audio);
        }     
    }
    audio->terminate = true;
    pthread_exit(NULL);
}

int main(int argc, char *argv[])
{
	pthread_t thr_pulse, thr_audio;
	audio_t audio;

	init_audio(&audio);
	init_leds();
    init_viz();

    setup_handlers();

    int err_code = pthread_create(&thr_pulse, NULL, pulse_input, (void*) &audio);
    assert(!err_code);
    err_code = pthread_create(&thr_audio, NULL, process_audio, (void*) &audio);
    assert(!err_code);
    pthread_join(thr_audio, NULL);
    pthread_join(thr_pulse, NULL);
    free_viz();
    free_leds();
    free_audio(&audio);
    return 0;
}
