# 3D Printed Reactive LED Speakers

> Work in progress

<img src="https://github.com/scholtzan/led-speaker/raw/main/img/speakers-darkness.jpg" width="1000">


## Raspberry Pi Setup

### Audio

_Based on [Another How to turn your Pi in a Bluetooth Speaker Tutorial](https://forums.raspberrypi.com/viewtopic.php?t=235519)_

Install Pulseaudio and bluetooth module:
```sudo apt-get install pulseaudio pulseaudio-module-bluetooth```

Add user to group `bluetooth`:
```
sudo usermod -a -G bluetooth pi
sudo reboot
```

Make discoverable:
```
sudo nano /etc/bluetooth/main.conf

...
Class = 0x41C
DiscoverableTimeout = 0
...

sudo systemctl restart bluetooth
```

In `bluetoothctl`, setup Bluetooth:
```
pi@raspberrypi:~ $ bluetoothctl
[NEW] Controller XX:XX:XX:XX:XX:XX raspberrypi [default]
[bluetooth]# power on
Changing power on succeeded
[bluetooth]# discoverable on
Changing discoverable on succeeded
[CHG] Controller XX:XX:XX:XX:XX:XX Discoverable: yes
[bluetooth]# pairable on
Changing pairable on succeeded
[bluetooth]# agent on
Agent registered
[bluetooth]# default-agent
Default agent request successful
[bluetooth]# quit
Agent unregistered
[DEL] Controller XX:XX:XX:XX:XX:XX raspberrypi [default]
```


Start Pulseaudio and check status:
```
pulseaudio --start

sudo systemctl status bluetooth
```

Start Pulseaudio on boot
```
systemctl --user enable pulseaudio
```

Auto-pairing
```
sudo apt-get install bluez-tools

sudo nano /etc/systemd/system/bt-agent.service

[Unit]
Description=Bluetooth Auth Agent
After=bluetooth.service
PartOf=bluetooth.service

[Service]
Type=simple
ExecStart=/usr/bin/bt-agent -c NoInputNoOutput

[Install]
WantedBy=bluetooth.target


sudo systemctl enable bt-agent
```

Disable onboard Bluetooth module in favour of Bluetooth dongle:
```
sudo nano /etc/modprobe.d/blacklist-bluetooth.conf

blacklist btbcm
blacklist hci_uart
```

Controlling volume:
```
sudo nano /etc/systemd/system/bluetooth.target.wants/bluetooth.service

...
ExecStart=/usr/lib/bluetooth/bluetoothd --noplugin=avrcp
...
```

### LED

Use SPI for LED strip:
```
/boot/cmdline.txt 
```

append 
```
spidev.bufsiz=32768

core_freq=500
core_freq_min=500
```

## Software

### Building

Install [cross](https://github.com/rust-embedded/cross)

Build docker container with libpulse installed:

`docker build -t rbp-cross . `

Build project

`cross build --target=armv7-unknown-linux-gnueabihf`