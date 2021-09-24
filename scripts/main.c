#include "audio.h"
#include "led.h"
#include "config.h"
#include "viz.h"
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
    int16_t color = 255;
    float count = 100.0f;
    audio_t *audio = (audio_t *) data;
    float height = (float) LED_COUNT;
    float diff;
    uint8_t mode = 1;

    while(running)
    {
        sem_wait(&sem_audio);
        if(find_beats(audio))
        {
            update_viz(audio);

            // color = 255;
            // height = (float) LED_COUNT;
            // count = 100;
        }
        // switch (mode)
        // {
        // case 0:
        //     diff = (g * count * count);
        //     if(diff < 1)
        //         diff = 1;
        //     height -= diff;

        //     if(height < 0)
        //         height = 0;
        //     count -= 1.0f;
        //     if(count < 0)
        //         count = 0;
        //     set_led((uint16_t) height, get_color(color));
        //     // fprintf(stderr, "%d\n", height);
        //     break;
        
        // case 1:
        //     height = audio->bar * LED_COUNT;
        //     clear_leds();
        //     set_led((uint16_t) height, get_color(color));
        //     render_leds();
        //     break;
        // default:
        //     break;
        // }
        // color -= 2;
        // if (color < 0)
        //     color = 0;      
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
