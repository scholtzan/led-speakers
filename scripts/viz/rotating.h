#ifndef _ROTATING_H_
#define _ROTATING_H_

#include "../config.h"
#include "../audio.h"
#include "../led.h"

typedef struct {
	color_t pixels[LED_COUNT];
	int rotation_skips;
} viz_t;

void update_viz(audio_t *audio);
void init_viz();
void free_viz();
void render_viz();
void apply_falloff();
void apply_rotation();

#endif