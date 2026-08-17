# V.I.S.OR. - Raspberry Pi Core Logic

This directory contains the Rust application running on the **Raspberry Pi 4**. It serves as the central intelligence and user interface for the **V.I.S.O.R.** system, coordinating offline voice activation, camera snapshot capture, multimodal AI triage via the Gemini 3.7 Flash API, Arduino motor commands over serial, and kiosk UI / video guidance via Chromium.

---

## Architecture & Data Flow

```
[Voice Trigger: "VISOR help"] (CPAL + Rustpotter)
          │
          ▼
[Camera Snapshot] (rpicam-still -> /tmp/visor_frame.jpg)
          │
          ▼
[Gemini 3.7 Flash API] (Structured JSON with Boolean Dispense Flags)
          │
    ┌─────┴────────────────────────────────┐
    ▼                                      ▼
[Arduino Serial Bridge]       [YouTube Search & Kiosk Player]
- Sends <DISP:b,a,g>\n        - Queries YouTube Data API v3
- Receives ACK:DISP:...       - Plays top instructional video
- Receives Status Updates     - Returns to "Ready to Help" standby
```

---

## Module Breakdown

| Module | File | Responsibility |
| :--- | :--- | :--- |
| **Audio** | [`src/modules/audio.rs`](./src/modules/audio.rs) | Captures microphone audio using `cpal` and detects `"VISOR help"` wake words offline using `rustpotter`. Supports optional custom `.rpw` models at `assets/visor_help.rpw`. |
| **Camera** | [`src/modules/input.rs`](./src/modules/input.rs) | Captures camera frames to `/tmp/visor_frame.jpg` using the `rpicam-still` hardware utility on the Raspberry Pi. |
| **Gemini AI** | [`src/modules/gemini.rs`](./src/modules/gemini.rs) | Transmits base64-encoded image snapshots to Gemini 3.7 Flash with structured schema constraints, extracting boolean dispensing flags (`bandage`, `alcohol_pad`, `gauze_pad`), triage reasoning, and video search queries. |
| **Arduino Bridge** | [`src/modules/arduino.rs`](./src/modules/arduino.rs) | Manages USB Serial communication with the Arduino Uno. Formats and sends framed `<DISP:b,a,g>\n` commands and reads acknowledgments. |
| **YouTube & Display** | [`src/modules/youtube.rs`](./src/modules/youtube.rs) | Searches YouTube Data API v3 for instructional first-aid videos. Controls a persistent fullscreen Chromium kiosk session, monitoring video playback (`video.ended`) and returning to standby. |
| **Standby UI** | [`assets/standby.html`](./assets/standby.html) | Standalone dark-mode kiosk interface displaying "VISOR: Ready to Help", voice trigger cues, and cartridge inventory statuses. |

---

## Serial Communication Protocol

Commands transmitted from the Raspberry Pi to the Arduino Uno:

- **Format**: `<DISP:b,a,g>\n`
  - `b`: Bandage (`1` = dispense, `0` = hold)
  - `a`: Alcohol Pad (`1` = dispense, 0` = hold)
  - `g`: Gauze Pad (`1` = dispense, `0` = hold)
- **Examples**:
  - `<DISP:1,0,1>\n` — Dispense Bandage and Gauze Pad.
  - `<DISP:0,0,0>\n` — Hold all items (condition untreatable or emergency).

---

## Environment Variables

| Variable | Required | Default | Description |
| :--- | :--- | :--- | :--- |
| `GEMINI_API_KEY` | **Yes** | — | Google AI Studio API key for Gemini 3.7 Flash multimodal triage. |
| `YOUTUBE_API_KEY` | **Yes** | — | Google Cloud API key for YouTube Data API v3 video search. |
| `ARDUINO_PORT` | No | `/dev/ttyAMA0` | Serial port connected to the Arduino Uno (e.g. `/dev/ttyUSB0` or `COM3`). |
| `ARDUINO_BAUD` | No | `9600` | Baud rate for serial communication with the Arduino Uno. |
| `RUST_LOG` | No | `info` | Logging verbosity level (`debug`, `info`, `warn`, `error`). |

---

## Build & Execution

### 1. Prerequisites (Raspberry Pi OS)
Ensure standard audio and camera tools are installed:
```bash
sudo apt update
sudo apt install -y libasound2-dev pkg-config chromium-browser
```

### 2. Running Tests
```bash
cargo test
```

### 3. Running the Service
```bash
export GEMINI_API_KEY="your-gemini-api-key"
export YOUTUBE_API_KEY="your-youtube-api-key"
export ARDUINO_PORT="/dev/ttyUSB0"
export RUST_LOG="info"

cargo run --release
```
