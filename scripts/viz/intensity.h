#ifndef _INTENSITY_H_
#define _INTENSITY_H_

#include "../config.h"
#include "../audio.h"
#include "../led.h"

typedef struct {
	color_t pixels[LED_COUNT];
} viz_t;

void update_viz(audio_t *audio);
void init_viz();
void free_viz();
void render_viz();
void apply_falloff();
void apply_rotation();

#endif