# V.I.S.O.R. System Verification & Test Report

**System Name:** V.I.S.O.R. (Visual Inspection & Smart Occupational Relief)  
**Report Version:** 1.0.0  
**Test Status:** **ALL 48 AUTOMATED TESTS PASSED (100% PASS RATE)**  
**Environment:** Cross-Platform Local Test Harness (Windows/Linux x86_64 & Linux ARM64 Target)

---

## 1. Executive Summary & Test Dashboard

This document provides the formal test report for the V.I.S.O.R. dual-controller first-aid dispensing system. The testing suite provides comprehensive test coverage spanning firmware serial protocols, audio signal downmixing/normalization, AI interaction schema validation, browser kiosk URL generation, end-to-end simulated triage pipelines, and high-precision latency benchmarks.

```
================================================================================
                              TEST SUITE DASHBOARD
================================================================================
 Suite Name                       Scope                  Tests   Passed   Failed
--------------------------------------------------------------------------------
 Rust Unit Tests (lib.rs)         Pi Logic Modules         27       27       0
 Rust Integration Tests           End-to-End Pipelines      3        3       0
 Rust Latency Benchmarks          Micro-benchmarks          4        4       0
 Arduino Protocol Suite (Python)  Firmware & Framing       14       14       0
--------------------------------------------------------------------------------
 TOTAL TESTS EXECUTED                                      48       48       0
 OVERALL STATUS                                                   [ PASS (100%) ]
================================================================================
```

---

## 2. Module Test Breakdown & Coverage Matrix

### 2.1. Arduino Serial Bridge (`modules/arduino.rs` & `arduino_control.ino`)
Tests the packet framing, baud rate protocol, command encoding, and status acknowledgment parsing between the Raspberry Pi and Arduino Uno.

| Test Function | Target Feature | Validation Logic | Status |
| :--- | :--- | :--- | :--- |
| `test_format_dispense_command_all_combinations` | Command Encoding | Verifies correct `<DISP:b,a,g>\n` generation for all 8 boolean permutations | **PASS** |
| `test_format_ping_command` | Keepalive | Verifies `<PING>\n` payload generation | **PASS** |
| `test_parse_serial_response_ready` | Status Parsing | Validates `STATUS:READY` detection and whitespace trimming | **PASS** |
| `test_parse_serial_response_ack` | Handshake ACK | Validates `ACK:DISP:1,0,1` tokenization into boolean flags | **PASS** |
| `test_parse_serial_response_status_messages` | Status Telemetry | Validates `STATUS:DISPENSING_*`, `STATUS:HOLD_ALL`, `STATUS:DISPENSE_COMPLETE`, and `PONG` | **PASS** |
| `test_parse_serial_response_errors_and_edge_cases` | Fault Tolerance | Validates `ERR:UNKNOWN_COMMAND`, empty strings, and unstructured text | **PASS** |

### 2.2. Gemini AI Medical Assessment (`modules/gemini.rs`)
Validates request payload schema construction, base64 image encapsulation, response parsing, and medical emergency triage deserialization.

| Test Function | Target Feature | Validation Logic | Status |
| :--- | :--- | :--- | :--- |
| `test_dispense_items_boolean_deserialization` | Minor Injury Assessment | Validates deserialization of positive triage with selective dispensing | **PASS** |
| `test_cannot_help_emergency_deserialization` | Emergency Triage | Validates `can_help = false`, hold flags, and null `video_search_query` | **PASS** |
| `test_omitted_video_search_query` | Schema Flexibility | Validates optional video query field when omitted from JSON | **PASS** |
| `test_malformed_json_failure` | Error Handling | Asserts deserialization failure on broken/incomplete JSON | **PASS** |
| `test_missing_dispense_field_failure` | Schema Enforcement | Asserts deserialization error when `dispense` object is missing | **PASS** |
| `test_build_request_body_structure` | API Formatting | Verifies Gemini 3.7 Flash model targeting, system prompt, and MIME structure | **PASS** |
| `test_extract_response_text_interactions_format` | Response Parsing | Validates parsing of `output[0].text` interaction format | **PASS** |
| `test_extract_response_text_candidates_format` | Response Parsing | Validates fallback parsing of `candidates[0].content.parts[0].text` format | **PASS** |
| `test_extract_response_text_invalid_format` | Error Extraction | Asserts error extraction when response format lacks standard parts | **PASS** |

### 2.3. Video Guidance & Kiosk Display (`modules/youtube.rs`)
Tests YouTube Data API v3 search response tokenization, video ID extraction, and Chromium kiosk URL formatting.

| Test Function | Target Feature | Validation Logic | Status |
| :--- | :--- | :--- | :--- |
| `test_format_embed_url` | Embed Formatting | Validates `https://www.youtube-nocookie.com/embed/{id}?autoplay=1...` | **PASS** |
| `test_parse_youtube_search_response_valid` | API Deserialization | Extracts video ID, title, and watch URL from search item list | **PASS** |
| `test_parse_youtube_search_response_empty_items` | Empty Results | Handles empty `items: []` list returning `None` | **PASS** |
| `test_parse_youtube_search_response_missing_video_id` | Channel/Playlist Filter | Rejects non-video items missing `videoId` | **PASS** |
| `test_resolve_standby_url_fallback` | Kiosk Fallback | Generates embedded HTML data URI when local asset is missing | **PASS** |

### 2.4. Audio Signal Preprocessing (`modules/audio.rs`)
Validates microphone PCM stream conversion, multi-channel downmixing, and 16-bit integer to floating-point sample normalization.

| Test Function | Target Feature | Validation Logic | Status |
| :--- | :--- | :--- | :--- |
| `test_downmix_f32_mono` | Mono Pass-through | Verifies 1-channel buffer unchanged | **PASS** |
| `test_downmix_f32_stereo` | Stereo Downmixing | Verifies exact channel averaging `(L + R) / 2` | **PASS** |
| `test_downmix_f32_quad_channel` | 4-Channel Array | Verifies multi-microphone array downmixing | **PASS** |
| `test_convert_i16_to_f32_mono` | PCM Normalization | Verifies $[-32768, 32767] \to [-1.0, 1.0]$ normalization | **PASS** |
| `test_convert_i16_to_f32_stereo` | Stereo PCM Downmix | Converts and normalizes interleaved 16-bit stereo PCM stream | **PASS** |

### 2.5. Vision & File I/O Subsystem (`modules/input.rs`)
Validates target snapshot directory resolution, recursive directory creation, and error boundaries.

| Test Function | Target Feature | Validation Logic | Status |
| :--- | :--- | :--- | :--- |
| `test_ensure_parent_dir_creates_directories` | Directory Handling | Creates missing nested parent directories for camera output | **PASS** |
| `test_ensure_parent_dir_handles_flat_filename` | Path Parsing | Handles flat filename paths without parent directories | **PASS** |

---

## 3. End-to-End Pipeline Integration Tests

Simulated full pipeline integration flows located in `src/pi_logic/tests/pipeline_integration_test.rs`:

1. **Minor Injury Pipeline Flow (`test_end_to_end_minor_injury_pipeline_flow`)**:
   - Audio trigger preprocessing $\to$ Camera snapshot directory creation $\to$ Base64 request body generation $\to$ AI triage response parsing (`can_help: true`) $\to$ Serial packet generation (`<DISP:1,1,0>\n`) $\to$ Simulated Arduino ACK & dispensing sequence $\to$ YouTube instructional video query resolution $\to$ Standby return.
   - **Result:** **PASSED**

2. **Emergency Hold Flow (`test_end_to_end_emergency_hold_pipeline_flow`)**:
   - Severe trauma detection $\to$ `can_help: false` $\to$ Dispenser lockout (`<DISP:0,0,0>\n`) $\to$ Arduino `STATUS:HOLD_ALL` acknowledgment.
   - **Result:** **PASSED**

3. **All Supplies Dispense Flow (`test_end_to_end_all_items_dispense_pipeline_flow`)**:
   - Comprehensive treatment scenario $\to$ Bandage + Alcohol + Gauze $\to$ `<DISP:1,1,1>\n` $\to$ `ACK:DISP:1,1,1`.
   - **Result:** **PASSED**

---

## 4. Latency & Performance Benchmarks

Micro-benchmarks located in `src/pi_logic/tests/latency_benchmarks.rs`:

```
+-------------------------------------------------------------+-------------------+-----------------+
| Benchmark Routine                                           | Measured Latency  | SLA Threshold   |
+-------------------------------------------------------------+-------------------+-----------------+
| Audio Downmix & Normalization (96,000 samples / 1s audio)   | ~2.03 ms          | < 50.00 ms      |
| Gemini Request Payload Building (300 KB snapshot image)     | ~25.86 µs         | < 5000.00 µs    |
| JSON Deserialization (VisorAnalysis struct)                 | ~3.96 µs / op     | < 100.00 µs     |
| Serial Packet Framing & Response Parsing                    | ~0.80 µs / op     | < 20.00 µs      |
+-------------------------------------------------------------+-------------------+-----------------+
```

> [!NOTE]
> All data transformation and serialization operations on the Raspberry Pi execute in **under 3 milliseconds total**, leaving $>99\%$ of the compute budget dedicated to network transmission and AI model inference.

---

## 5. Arduino Firmware Protocol Suite Results

Results from `tests/arduino_protocol_suite.py`:

```
======================================================================
      V.I.S.O.R. ARDUINO PROTOCOL & FIRMWARE VERIFICATION REPORT     
======================================================================
Total Tests Executed: 14
Passed: 14
Failed: 0
----------------------------------------------------------------------
01. [PASS] Ping-Pong Keepalive
02. [PASS] Dispense Combination (0,0,0) -> Hold All
03. [PASS] Dispense Combination (1,0,0) -> Bandage Only
04. [PASS] Dispense Combination (0,1,0) -> Alcohol Pad Only
05. [PASS] Dispense Combination (0,0,1) -> Gauze Pad Only
06. [PASS] Dispense Combination (1,1,0) -> Bandage + Alcohol
07. [PASS] Dispense Combination (1,0,1) -> Bandage + Gauze
08. [PASS] Dispense Combination (0,1,1) -> Alcohol + Gauze
09. [PASS] Dispense Combination (1,1,1) -> All Three Items
10. [PASS] Serial Garbage Prefix Filtering
11. [PASS] Concatenated Multi-Packet Stream
12. [PASS] Unknown Command Error Emission (<UNKNOWN_ACTION>)
13. [PASS] Serial Buffer Overflow Boundary Limit (>64 bytes)
14. [PASS] Incomplete Frame Reset on New Start Delimiter
======================================================================
RESULT: ALL PROTOCOL TESTS PASSED SUCCESSFULLY.
======================================================================
```

---

## 6. Hardware-in-the-Loop (HIL) Deployment Verification Checklist

For physical deployment verification on the assembled Raspberry Pi 4 + Arduino Uno enclosure:

- [ ] **Serial Wiring & Permissions**: Connect Arduino Uno via USB Serial. Confirm port exists (`ls /dev/ttyAMA0` or `ls /dev/ttyUSB*`) and user is in `dialout` group.
- [ ] **Wake Word Audio Check**: Run `cargo run --release` and verify microphone stream initializes at 44.1/48 kHz. Speak `"VISOR help"` into the microphone and check for terminal trigger log.
- [ ] **Camera Sensor Test**: Verify `rpicam-still -o /tmp/test.jpg` creates a valid JPEG snapshot in under 1 second.
- [ ] **Actuator Physical Ejection**:
  - Test Bandage ejection cycle ($2200\text{ ms}$ push, $150\text{ ms}$ pause, $2300\text{ ms}$ retract).
  - Test Alcohol Pad ejection cycle.
  - Test Gauze Pad ejection cycle.
  - Confirm servos detach to $0\text{ mA}$ idle current after cycle completion.
- [ ] **Kiosk Video Streaming**: Verify Chromium launches in `--kiosk` mode and automatically displays instructional first-aid video upon receiving Gemini analysis.
