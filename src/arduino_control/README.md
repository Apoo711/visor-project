# V.I.S.O.R. - Arduino Uno Dispenser Control

This directory contains the C++ firmware for the Arduino Uno microcontroller responsible for driving the 3x FEETECH FS90R continuous rotation micro servos in the rack-and-pinion dispensing cartridges.

---

## Hardware Configuration & Pinout

| Subsystem / Cartridge | Item Dispensed | Arduino Pin | Servo Wire Color |
| :--- | :--- | :--- | :--- |
| **Slot 1** | Bandage (Normal Size) | **Digital Pin 9** (PWM) | Orange/White (Signal) |
| **Slot 2** | Alcohol Prep Pad | **Digital Pin 10** (PWM) | Orange/White (Signal) |
| **Slot 3** | Gauze Pad | **Digital Pin 11** (PWM) | Orange/White (Signal) |
| **Power (+5V)** | All Servos | **5V Rail** (External / USB) | Red |
| **Ground (GND)** | All Servos | **GND Rail** (Common GND) | Brown/Black |
| **Status LED** | System Activity | **Digital Pin 13** | Built-in LED |

> [!IMPORTANT]
> **Common Ground**: Ensure that if an external 5V/6V power supply is used for the servos, the external power ground and the Arduino ground are tied together (**Common Ground**).

---

## Serial Communication Protocol

- **Connection**: USB Serial (`/dev/ttyAMA0` or `/dev/ttyUSB0` / `/dev/ttyACM0` on the Raspberry Pi)
- **Baud Rate**: `9600`
- **Command Framing**: `<DISP:b,a,g>\n`
  - `b`: `1` = Dispense Bandage, `0` = Hold
  - `a`: `1` = Dispense Alcohol Pad, `0` = Hold
  - `g`: `1` = Dispense Gauze Pad, `0` = Hold

### Example Commands & Responses

1. **Dispense Bandage & Gauze Pad**:
   - Raspberry Pi transmits: `<DISP:1,0,1>\n`
   - Arduino responds: `ACK:DISP:1,0,1`
   - Arduino status updates:
     - `STATUS:DISPENSING_BANDAGE`
     - `STATUS:DISPENSING_GAUZE`
     - `STATUS:DISPENSE_COMPLETE`

2. **Hold All Items**:
   - Raspberry Pi transmits: `<DISP:0,0,0>\n`
   - Arduino responds: `ACK:DISP:0,0,0` followed by `STATUS:HOLD_ALL`

---

## FS90R Continuous Rotation Calibration & Tuning

1. **Zero-Point Trimpot Adjustment**:
   - FS90R servos feature a miniature potentiometer adjustment screw on the side.
   - If a servo slowly drifts or creeps when powered before `servo.detach()` is called, gently turn the trimpot using a small screwdriver until the motor stops moving completely at signal `90`.

2. **Stroke Timing Customization**:
   In `arduino_control.ino`, the timing constants are set to match the rack length to ensure that the servo extends and retracts far enough to push the item out:
   ```cpp
   const unsigned long TIME_PUSH_MS    = 2200; // Extend rack to push item out
   const unsigned long TIME_PAUSE_MS   = 150;  // Dwell buffer
   const unsigned long TIME_RETRACT_MS = 2300; // Retract rack to home
   ```

---

## How to Flash / Upload

1. Connect the Arduino Uno to your computer via USB.
2. Open [`arduino_control.ino`](./arduino_control.ino) in the Arduino IDE (or use the Arduino CLI / PlatformIO).
3. Select Board: **Arduino Uno** and select the appropriate serial port.
4. Click **Upload**.