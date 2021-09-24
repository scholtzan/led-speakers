#ifndef _LED_H
#define _LED_H

void init_leds();
void free_leds();
void clear_leds();
void render_leds();
void set_led(uint16_t position, uint32_t color);
uint32_t get_color(uint8_t r, uint8_t g, uint8_t b, uint8_t w);

#endif