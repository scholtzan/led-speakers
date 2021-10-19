#include "intensity.h"
#include "../audio.h"
#include "../config.h"
#include "../led.h"

viz_t intensity_viz = {};

color_t colors[] = {
    {255, 0, 0, 1},
    {0, 255, 0, 1},
    {0, 0, 255, 1},
    {255, 0, 255, 1},
    {255, 255, 0, 1},
    {0, 255, 255, 1},
    {50, 255, 50, 1},
};

void update_viz(audio_t *audio) {
    apply_falloff();

    for (int b = 0; b < BANDS + 1; b++) {
        int h = audio->bands[b] + 1;
        int n = (LED_COUNT / (BANDS + 1)) * b + 50;

        int intensity = h / 20;
        color_t color = colors[b];

        for (int i = 0; i < intensity; i++) {
            intensity_viz.pixels[n + i].r = h / 50 * color.r;
            intensity_viz.pixels[n + i].g = h / 50 * color.g;
            intensity_viz.pixels[n + i].b = h / 50 * color.b;
            intensity_viz.pixels[n + i].w = h / 50 * color.w;
        }
    }

    render_viz();
}

void apply_falloff() {
    for (int n = 0; n < LED_COUNT; n++) {
        intensity_viz.pixels[n].r = (int) (intensity_viz.pixels[n].r / DECAY);
        intensity_viz.pixels[n].g = (int) (intensity_viz.pixels[n].g / DECAY);
        intensity_viz.pixels[n].b = (int) (intensity_viz.pixels[n].b / DECAY);
        intensity_viz.pixels[n].w = (int) (intensity_viz.pixels[n].w / DECAY);
    }
}

void render_viz() {
    for (int n = 0; n < LED_COUNT; n++) {
        set_led(n, get_color(intensity_viz.pixels[n].r, intensity_viz.pixels[n].g, intensity_viz.pixels[n].b, intensity_viz.pixels[n].w));
    }
    render_leds();
}


void init_viz() {
    for (int n = 0; n < LED_COUNT; n++) {
        color_t color = {0, 0, 0, 0};
        intensity_viz.pixels[n] = color;
    }
    clear_leds();
    render_leds();
}

void free_viz() {

}