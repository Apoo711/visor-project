# V.I.S.O.R. Source Code

This directory houses the complete dual-controller source code for the **V.I.S.O.R.** (Visual Inspection & Smart Occupational Relief) automated first-aid diagnosis and dispensing system.

---

## System Architecture

V.I.S.O.R. partitions high-level AI analysis and low-level electromechanical actuation across a **Raspberry Pi 4** and an **Arduino Uno** connected via USB Serial:

```
                      +------------------------------------------+
                      |         Raspberry Pi 4 (pi_logic)        |
                      |  - Wake Word Spotter ("VISOR help")     |
                      |  - Camera Capture (rpicam-still)         |
                      |  - Gemini 3.7 Flash AI Medical Assessment|
                      |  - YouTube Instructional Video Search    |
                      |  - Fullscreen Chromium Kiosk Display     |
                      +--------------------+---------------------+
                                           |
                               USB Serial  | (<DISP:b,a,g>\n)
                               9600 Baud   | (ACK / Status)
                                           v
                      +--------------------+---------------------+
                      |      Arduino Uno (arduino_control)       |
                      |  - Packet Framing & Handshake Parser     |
                      |  - 3x FS90R Continuous Rotation Servos   |
                      |  - Linear Rack-and-Pinion Dispensers     |
                      +------------------------------------------+
```

---

## Subdirectories

### 1. [`pi_logic/`](./pi_logic/)
The Rust application running on the Raspberry Pi:
- **Audio Listener**: Listens offline for the `"VISOR help"` wake word using `rustpotter` and `cpal`.
- **Visual Inspection**: Captures a snapshot via `rpicam-still` and evaluates injury severity using Gemini 3.7 Flash.
- **Dispense Dispatcher**: Encodes the boolean dispensing decisions into framed serial packets (`<DISP:1,0,1>\n`) for the Arduino.
- **Kiosk & Video Guidance**: Displays a sleek "VISOR: Ready to Help" standby UI via `chromiumoxide` and automatically streams instructional first-aid YouTube videos.

### 2. [`arduino_control/`](./arduino_control/)
The C++ firmware for the Arduino Uno:
- **PWM Motor Actuation**: Controls 3x FEETECH FS90R continuous rotation micro servos driving linear rack-and-pinion dispensers for Bandages, Alcohol Prep Pads, and Gauze Pads.
- **Serial Handshake**: Reads framed ASCII packets, returns acknowledgments (`ACK:DISP:b,a,g`), and emits real-time status telemetry.
- **Anti-Jitter & Power Management**: Automatically detaches servos when idle to prevent zero-point drift, buzzing, and current draw.

---

## Development & Deployment

Refer to the respective README files in each subdirectory for detailed setup, wiring diagrams, and build instructions:
- [Raspberry Pi Logic Documentation](./pi_logic/README.md)
- [Arduino Uno Control Documentation](./arduino_control/README.md)
