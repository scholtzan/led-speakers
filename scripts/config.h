#define SINK_NAME "alsa_output.usb-Generic_USB2.0_Device_20170726905959-00.analog-stereo" 

// viz config

#define DECAY 1.1
#define ROTATION_SPEED 5


// audio config

#define M_PI 3.1415926535897932385
#define MAVG_FILT_SIZE  4
#define CHUNK   1024
#define FFT_TRANF_SIZE CHUNK * 2
#define MAX_BPM 240u
#define BANDS 20
#define RATE 44100
#define CHANNELS 2
#define FORMAT 16
#define FFT_BASS_BUFFER_SIZE 4096
#define FFT_MID_BUFFER_SIZE 2048
#define FFT_TREBLE_BUFFER_SIZE 1024
#define LOWER_CUTOFF 50
#define UPPER_CUTOFF 10000
#define BASS_CUTOFF 150
#define TREBLE_CUTOFF 2500
#define SENSITIVITY 100
#define MONSTERCAT 100
#define WAVES 1
#define GRAVITY 100
#define BUFFER_SIZE 8192
#define PERIOD_SIZE 1024

// LED config

#define LED_COUNT 				120
#define TARGET_FREQ             WS2811_TARGET_FREQ
#define GPIO_PIN                10
#define DMA                     10
#define STRIP_TYPE              WS2811_STRIP_GBR
