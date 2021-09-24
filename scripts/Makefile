make: main.c audio.c led.c viz.c 
	gcc main.c audio.c led.c viz.c -lws2811 -lpthread -lfftw3 -lfftw3f -lm -Lvendor/rpi_ws281x -Ivendor/rpi_ws281x `pkg-config --cflags --libs libpulse libpulse-simple` -g -O0 -o main
