# DBus Reactor
A lightweight, extensible Rust framework for monitoring D-Bus property changes and triggering system actions. While designed for KDE Plasma environments, the core logic is generic enough to monitor any system or session bus property.

## Features

### Power Profile fps

A critical factor on my laptop is that it supports 120 fps. This change seems to changed idle battery usage from 5.2W to 4.6W.

Power Saver: 60 FPS 
Balanced/Performance: 120 FPS

## Setup

Run

```
[andrea@archlinux dbus-reactor]$ kscreen-doctor -o
```

The result will be something like
``` 
Output: 1 ...
    ...
	Modes:  1:2880x1800@120.00!  2:2880x1800@60.00*  3:1600x1200@59.87  4:1600x1200@119.82  ...
```

In this case i have `2880x1800@120.00!` and `2:2880x1800@60.00*`. Which are perfect for power safe and performance. So the power mode should switch with `kscreen-doctor output.1.mode.2` where 1 is the id of the Output and 2 is the id of the mode

### As service

Modify service to use current user

```
echo $XDG_RUNTIME_DIR
# Usually outputs: /run/user/1000
``` 

if user != 1000 -> `Environment=XDG_RUNTIME_DIR=/run/user/1000` should be changed


Copy service to systemd

```
cargo build -r
sudo cp target/release/dbus-reactor /usr/local/bin/dbus-reactor
sudo cp dbus-reactor.service /etc/systemd/system/
sudo systemctl enable --now dbus-reactor
```
