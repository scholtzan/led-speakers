#include "viz.h"
#include "audio.h"
#include "config.h"
#include "led.h"

viz_t viz = {};

void update_viz(audio_t *audio) {
    viz.rotation_skips += 1;
    apply_falloff();

    if (viz.rotation_skips >= ROTATION_SPEED) {
        apply_rotation();
        viz.rotation_skips = 0;

        for (int b = 0; b < BANDS + 1; b++) {
            int h = audio->bands[b] + 1;
            int n = LED_COUNT / (BANDS + 1) * h / 100;
            float r = 1;
            float b = 1;
            float g = 1;

            if (b < BANDS * 1/3) {
                r = 0.8;
            } else if (b < BANDS * 2/3 ) {
                g = 0.8;
            } else {
                b = 0.8;
            }

            if (n > 0) {
                for (int p = b+3; p < LED_COUNT; p+=LED_COUNT/n) {
                    viz.pixels[p].r = 255 * h/50 * r;
                    viz.pixels[p].g = 255 * h/50 * g;
                    viz.pixels[p].b = 255 * h/50 * b;
                    viz.pixels[p].w = 255 * h/50;
                }
            }

        }
    }
    render_viz();
}

void apply_rotation() {
    color_t prev = viz.pixels[0];
    for (int n = 0; n < LED_COUNT; n++) {
        color_t tmp = viz.pixels[n + 1];
        viz.pixels[n + 1] = prev;
        prev = tmp;
    }
}

void apply_falloff() {
    for (int n = 0; n < LED_COUNT; n++) {
        viz.pixels[n].r = (int) (viz.pixels[n].r / DECAY);
        viz.pixels[n].g = (int) (viz.pixels[n].g / DECAY);
        viz.pixels[n].b = (int) (viz.pixels[n].b / DECAY);
        viz.pixels[n].w = (int) (viz.pixels[n].w / DECAY);
    }
}


void render_viz() {
    for (int n = 0; n < LED_COUNT; n++) {
        set_led(n, get_color(viz.pixels[n].r, viz.pixels[n].g, viz.pixels[n].b, viz.pixels[n].w));
    }
    render_leds();
}


void init_viz() {
    for (int n = 0; n < LED_COUNT; n++) {
        color_t color = {0, 0, 0, 0};
        viz.pixels[n] = color;
    }
    viz.rotation_skips = 0;
    clear_leds();
    render_leds();
}

void free_viz() {

}