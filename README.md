# 3D Printed LED Speakers

todo


## Software

### Building

Install [cross](https://github.com/rust-embedded/cross)

Build docker container with libpulse installed:

`docker build -t rbp-cross . `

Build project

`cross build --target=armv7-unknown-linux-gnueabihf`