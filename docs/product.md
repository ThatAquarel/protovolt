# Product overview

ProtoV MINI is a portable lab power supply designed for electronics
prototyping and field testing.

## Features

- Two independent, adjustable output channels
- USB-C Power Delivery input
- Credit card-sized enclosure
- D-pad and control-button interface
- Open Rust firmware and KiCad hardware design
- Headers compatible with common prototyping breadboards

## Specifications

| | |
| --- | --- |
| Input | USB-C PD 2.0, up to 100 W |
| Outputs | 2 adjustable channels |
| Voltage | 0–20 V in 10 mV steps |
| Current | 0–5 A in 50 mA steps |
| Dimensions | 85.5 × 54 × 18 mm |
| Display | 2-inch, 320 × 240 TFT |
| Controls | D-pad and control buttons |
| MCU | RP2040 |

## Hardware

Each channel uses an independent buck-boost power path. The USB-C input
negotiates up to 100 W, while the RP2040 controls output regulation,
measurement, protection, and the user interface.

The two 2×5, 2.54 mm-pitch output headers align with the power rails on BB400
and BB830 breadboards.

## Gallery

![MCU detail](res/protovolt-mcu-zoom.jpg)

![Back of the PCB](res/protovolt-back-pcb.jpg)

![User interface](res/protovolt-user-interface-zoom.jpg)
