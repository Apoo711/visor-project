# V.I.S.O.R. (Visual Inspection & Smart Occupational Relief)
**Year 12 Systems Engineering SAT Project**  
Designed and Developed by Aryan Gupta

(NEED CAD RENDER)

## Project Overview

**V.I.S.O.R.** is an automated, AI-driven first aid console designed to address the **"Panic Gap"--**the critical delay in first aid treatment caused by shock, hesitation, or a lack of medical knowledge during minor workshop emergencies.

Current first aid kits are entirely passive, forcing an injured, panicked user to navigate supplies and manuals under stress. V.I.S.O.R. eliminates this cognitive load. By utilizing a dual-microcontroller architecture and the Gemini AI API, the system takes visual or audio distress signals, autonomously diagnoses the injury, and actively dispenses the correct medical supply while playing a relevant instructional video.

## The Engineering Challenge & Solution

#### The Problem with Existing Solutions

During the research phase, standard gravity-fed mechanisms, solenoid trapdoors, and spiral vending coils were evaluated. Testing revealed a fatal flaw: **soft, flexible medical packaging (like bandage wrappers) frequently snag and jam in gravity or coil-based systems,** leading to an unacceptable failure rate during emergencies. Additionally, acoustic voice-activation systems failed reliably in 80+ decibel workshop environments.

#### The V.I.S.O.R. Solution

To guarantee a **>90% dispensing reliability,** V.I.S.O.R. utilizes a custom 3D-printed **Rack and Pinion** mechanism. Driven by a continuous rotation servo, this system translates rotational kinetic energy into linear horizontal thrust. A wedge-shaped "pusher sled" actively forces the flexible packet through a tight 2.5mm slit, entirely overcoming friction and preventing jams.

For the interface, visual AI processing (via a camera) was selected to circumvent the unreliability of workshop acoustics.

## System Architecture

V.I.S.O.R. relies on a complex, integrated **Dual-Microcontroller Architecture** to separate high-level logical processing from low-level electromechanical actuation.

- **High-Level Logic (Raspberry Pi 4):** Runs a Python environment to handle USB camera/microphone input, securely query the Gemini Vision-Language API, and display YouTube first aid tutorials on an integrated 7-inch HDMI touchscreen.

- **Serial Communication (UART):** The Pi parses the AI's diagnosis into a deterministic binary string (e.g., `010` for Gauze) and transmits it via asynchronous serial communication to the motor controller.

- **Low-Level Actuation (Arduino Uno R3):** The Arduino reads the serial bytes and generates precise Pulse Width Modulation (PWM) signals to drive the FS90R continuous rotation servos. It also monitors SPDT micro limit switches for closed-loop mechanical feedback, ensuring the rack returns to its exact "home" position after dispensing.

## Technologies & Materials

#### Hardware & Electronics

- **Microcontrollers:** Raspberry Pi 4 Model B (4GB), Arduino Uno R3

- **Actuators:** 3x FS90R Continuous Rotation Servos

- **Sensors:** Raspberry Pi Camera Module V3, USB Microphone, SPDT Micro Limit Switches

- **Interface:** 7-inch HDMI Touchscreen Display

- **Power:** Isolated 240V AC to 5V DC (3A) mains adapter

#### Manufacturing Processes

- **Fused Deposition Modelling (FDM) 3D Printing:** Used PLA bioplastic to rapidly prototype and manufacture the complex involute gear teeth for the rack and pinion cartridges with strict 1.5mm sliding tolerances.

- **Laser Cutting / CNC:** The main chassis is constructed from 6mm Pine Plywood for high tensile strength, acoustic dampening, and sustainability, featuring precision-cut bezels for the screen and dispensing tray.

#### Software Stack

- **Python 3:** API integration, JSON parsing, UI handling, and serial transmission.

- **C++ (Arduino):** Interrupt polling, hardware PWM timing, and serial reading.

- **Google Gemini API:** Cloud-based Vision-Language Model processing.

## Performance Metrics

The system was designed and evaluated against strict operational criteria:

- **Response Latency:** `< 15 Seconds` (From user input to video display and motor actuation) to effectively bridge the "Panic Gap".

- **Dispensing Reliability:** `> 90% Success Rate` achieved via the active linear thrust of the rack and pinion mechanism.

- **Interpretation Accuracy**: `100% Logical Match` between the user's spoken/visual distress signal and the dispensed item.

*This repository contains the CAD models (STL/DXF files), system schematics, Arduino C++ code, and Python scripts developed for this portfolio project.*
