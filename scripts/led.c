// gcc led.c -lws2811 -lm -Lvendor/rpi_ws281x -Ivendor/rpi_ws281x -o led

#include "ws2811/ws2811.h"
#include <ws2811/clk.h>
#include <ws2811/gpio.h>
#include <ws2811/dma.h>
#include <ws2811/rpihw.h>
#include <stdio.h>
#include "led.h"
#include "config.h"

ws2811_t leds =
{
    .freq = TARGET_FREQ,
    .dmanum = DMA,
    .channel =
    {
        [0] =
        {
            .gpionum = GPIO_PIN,
            .count = LED_COUNT,
            .invert = 0,
            .brightness = 255,
            .strip_type = STRIP_TYPE,
        },
        [1] =
        {
            .gpionum = 0,
            .count = 0,
            .invert = 0,
            .brightness = 0,
        },
    },
};

void init_leds() {
    ws2811_return_t ret;
    if ((ret = ws2811_init(&leds)) != WS2811_SUCCESS)
    {
        fprintf(stderr, "ws2811_init failed: %s\n", ws2811_get_return_t_str(ret));
        return;
    }

    clear_leds();
}

void clear_leds() {
    for (int x = 0; x < LED_COUNT; x++) {
        leds.channel[0].leds[x] = 0;
    }

    render_leds();
}

void free_leds() {
    clear_leds();
    ws2811_fini(&leds);
}

void render_leds() {
    ws2811_render(&leds);
}

void set_led(uint16_t position, uint32_t color) {
    leds.channel[0].leds[position] = color;
}

uint32_t get_color(uint8_t r, uint8_t g, uint8_t b, uint8_t w)
{
    return (((uint32_t)w) << 24 | ((uint32_t)r) << 16 | ((uint32_t)g) << 8 | ((uint32_t)b));
}

// uint32_t get_color(int16_t val)
// {
//     uint8_t r, g, b;
//     val *= 2;
//     if(val > 255)
//     {
//         r = val - 255;
//         g = (510 - val);
//         b = 0;
//     }
//     else
//     {
//         r = 0;
//         g = val;
//         b = (254 - val);
//     }
//     return (((uint32_t)r) << 16 | ((uint32_t)g) << 8 | ((uint32_t)b));
// }
