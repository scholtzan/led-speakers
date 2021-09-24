#ifndef _VIZ_H_
#define _VIZ_H_

#include "config.h"
#include "audio.h"

typedef struct {
	uint8_t r;
	uint8_t g; 
	uint8_t b; 
	uint8_t w;
} color_t;

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