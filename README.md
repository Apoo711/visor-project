# V.I.S.O.R. (Visual Inspection & Smart Occupational Relief)
**Year 12 Systems Engineering SAT Project**  
Designed and Developed by Aryan Gupta

<div align="center">

[![Live Telemetry Dashboard](https://img.shields.io/badge/Live%20Telemetry%20Dashboard-GitHub%20Pages-00d2ff?style=for-the-badge&logo=github&logoColor=white)](https://apoo711.github.io/visor-project/)
[![Test Report](https://img.shields.io/badge/Verification%20Report-48%20%2F%2048%20Passed-brightgreen?style=for-the-badge&logo=checkmarx&logoColor=white)](./tests/TEST_REPORT.md)
[![Rust](https://img.shields.io/badge/Rust-2024%20Edition-orange?style=for-the-badge&logo=rust&logoColor=white)](./src/pi_logic/)
[![Hardware](https://img.shields.io/badge/Hardware-Pi%205%20%2B%20Arduino%20Uno-8A2BE2?style=for-the-badge&logo=raspberrypi&logoColor=white)](./src/arduino_control/)

</div>

---


## Project Overview

**V.I.S.O.R.** is an automated, AI-driven first aid triage and dispensing console designed to eliminate the **"Panic Gap"**—the critical delay in first aid treatment caused by shock, hesitation, or a lack of medical knowledge during minor workshop emergencies.

Traditional first aid kits are completely passive, requiring an injured and panicked user to navigate supplies and instructional manuals under acute cognitive stress. V.I.S.O.R. automates this process. By combining a high-performance dual-controller architecture with the multimodal **Gemini 3.7 Flash API**, the system:
1. Listens for the **"VISOR help"** voice activation wake word.
2. Captures an immediate snapshot of the injury via the camera.
3. Autonomously diagnoses the condition and formulates a treatment and dispensing plan.
4. Electromechanically dispenses the required medical supplies from active rack-and-pinion cartridges.
5. Displays a relevant first-aid video on the kiosk display, automatically returning to standby once guidance concludes.

---

## The Engineering Challenge & Solution

#### The Problem with Existing Dispensing Solutions
During initial research and experimentation, standard gravity-fed chutes, solenoid trapdoors, and spiral vending coils were evaluated. Testing revealed that **soft, flexible medical packaging (such as bandage and pad wrappers) frequently snags and jams in passive systems**, causing an unacceptable failure rate during emergencies.

#### The V.I.S.O.R. Mechanical Solution
To guarantee **>90% dispensing reliability**, V.I.S.O.R. employs a custom 3D-printed **Rack and Pinion** mechanism. Driven by FEETECH FS90R continuous rotation micro servos, the mechanism translates rotational energy into positive linear horizontal thrust. A pusher sled forcibly guides the flexible package out of the cartridge, preventing friction jams and ensuring reliable dispensing.

---

## System Architecture

V.I.S.O.R. utilizes a robust **Dual-Microcontroller Architecture** to cleanly decouple high-level AI reasoning and UI management from low-level electromechanical PWM motor timing:

```
                      +---------------------------------------------------+
                      |             Raspberry Pi 4 (Rust)                 |
                      |  - Offline Wake Word Spotter ("VISOR help")       |
                      |  - Camera Capture (`rpicam-still`)                |
                      |  - Gemini 3.7 Flash Vision AI Assessment          |
                      |  - YouTube Data API v3 Instructional Search       |
                      |  - Fullscreen Chromium Kiosk Display Manager      |
                      +-------------------------+-------------------------+
                                                |
                                    USB Serial  | (<DISP:b,a,g>\n)
                                    9600 Baud   | (ACK:DISP:b,a,g / Status)
                                                v
                      +-------------------------+-------------------------+
                      |               Arduino Uno (C++)                   |
                      |  - Packet Framing & Handshake Parser              |
                      |  - Hardware PWM Timing & Auto-Detach              |
                      |  - 3x FS90R Continuous Rotation Micro Servos      |
                      |  - Active Linear Rack-and-Pinion Dispensers       |
                      +---------------------------------------------------+
```

### Data & Execution Flow
1. **Standby**: The kiosk display presents a sleek, medical-grade "VISOR: Ready to Help" screen while the audio subsystem listens for the wake word.
2. **Wake Trigger**: The user says *"VISOR help"*, detected offline via `rustpotter` and `cpal`.
3. **Capture & Vision Triage**: The camera captures `/tmp/visor_frame.jpg` and queries the Gemini 3.7 Flash API with structured JSON constraints, receiving boolean dispensing flags (`bandage`, `alcohol_pad`, `gauze_pad`).
4. **Actuation**: The Pi transmits a framed packet (e.g. `<DISP:1,0,1>\n`) over USB Serial. The Arduino acknowledges the command and drives the corresponding FS90R servos through a push $\rightarrow$ dwell $\rightarrow$ retract $\rightarrow$ detach cycle.
5. **Instructional Video & Return**: The Pi resolves the top instructional video via the YouTube Data API, autoplays the video in the Chromium kiosk, monitors completion (`video.ended`), and returns the screen to the standby UI.

---

## Technologies & Materials

### Hardware & Electronics
- **Compute:** Raspberry Pi 4 Model B (4GB), Arduino Uno R3
- **Actuators:** 3x FEETECH FS90R Continuous Rotation Micro Servos
- **Sensors & Input:** Raspberry Pi Camera Module V3, USB Microphone / ALSA capture
- **Display:** Integrated HDMI Display running fullscreen Chromium kiosk
- **Power:** Regulated 5V DC power supply with common-ground integration

### Software Stack
- **Raspberry Pi Core Logic ([`src/pi_logic/`](./src/pi_logic/)):**
  - **Language:** Rust (2024 Edition)
  - **Async Runtime:** `tokio`
  - **Wake Word Detection:** `rustpotter` (pure Rust offline spotter) + `cpal`
  - **AI & Cloud API:** `reqwest`, `serde`, `serde_json` (Google Gemini 3.7 Flash & YouTube Data API v3)
  - **Browser Kiosk Automation:** `chromiumoxide`
  - **Serial Bridge:** `serialport`
  - **User Interface:** HTML5, CSS3 Glassmorphism, Google Fonts (`Outfit` & `Space Grotesk`)
- **Arduino Firmware ([`src/arduino_control/`](./src/arduino_control/)):**
  - **Language:** C++ (Arduino Framework)
  - **Libraries:** `Servo.h`
  - **Mechanisms:** Non-blocking ASCII packet framing, PWM servo timing, idle power auto-detach

### Manufacturing & Fabrication
- **FDM 3D Printing:** PLA bioplastic for modular cartridge bodies, linear gear racks, and drive pinions with precise sliding tolerances.
- **Enclosure:** CNC / Laser-cut Pine Plywood chassis providing structural rigidity and mounting bezels for screen, camera, and dispensing bays.

---

## Repository Structure

```
├── cad/                     # 3D models (STL/STEP) for chassis & rack-and-pinion cartridges
├── docs/                    # SAT portfolio documentation, Criterion reports, and LaTeX sources
├── src/
│   ├── README.md            # Overview of source architecture
│   ├── pi_logic/            # Rust application for Raspberry Pi 4
│   │   ├── assets/          # Kiosk UI (standby.html) and audio models
│   │   ├── src/             # Rust modules (audio, camera, gemini, serial, youtube)
│   │   └── Cargo.toml       # Rust dependencies and configuration
│   └── arduino_control/     # C++ firmware for Arduino Uno
│       ├── arduino_control.ino # Arduino sketch for FS90R servo actuation
│       └── README.md        # Pinouts, wiring diagrams, and calibration guide
└── tests/                   # Hardware and mechanical test scripts
```

---

## Performance Targets

- **Response Latency:** `< 15 Seconds` from user voice prompt to active dispensing and video playback.
- **Dispensing Reliability:** `> 90% Success Rate` achieved via active rack-and-pinion horizontal thrust.
- **Interpretation Accuracy:** `100% Logical Match` between the injury assessment and the dispensed supply combination.

---

## License

This project is licensed under the [MIT License](./LICENSE).
