# V.I.S.O.R. Laser-Cut Chassis Subsystem

This directory contains the computer-aided manufacturing specifications and structural layout for the **V.I.S.O.R.** kiosk enclosure.

---

## Enclosure Overview

The V.I.S.O.R. console chassis is constructed from **$6\text{ mm}$ Pine Plywood** using precision laser cutting (or CNC routing). The enclosure is engineered to:
1. House and protect the central compute electronics (Raspberry Pi 4 and Arduino Uno R3).
2. Integrate the **HDMI kiosk touchscreen/display** and **Raspberry Pi Camera Module V3** behind a rigid front bezel.
3. Provide calibrated mounting bays for the 3 electromechanical dispensing cartridges, aligning the ejection slots directly with the delivery collection tray.
4. Facilitate adequate passive airflow and cable routing between the 5V power supply, controllers, and actuators.

---

## Structural Panel Specifications

| Panel Component | Material | Thickness | Dimensions / Features |
| :--- | :--- | :--- | :--- |
| **Front Bezel** | Pine Plywood | $6\text{ mm}$ | Screen aperture, camera lens standoff mount, and 3 lower cartridge exit windows. |
| **Dispensing Base Shelf** | Pine Plywood | $6\text{ mm}$ | Horizontal floor supporting the 3 cartridge modules with slots for rack motion. |
| **Side & Rear Panels** | Pine Plywood | $6\text{ mm}$ | Interlocking finger-joint tabs, ventilation louvers, and power inlet cutouts. |
| **Top Hood / Cover** | Pine Plywood | $6\text{ mm}$ | Removable service lid for restocking supply cartridges and accessing electronics. |

---

## Recommended Laser Cutting Settings ($6\text{ mm}$ Plywood)

> [!NOTE]
> Values serve as baseline recommendations for an 80W–100W $CO_2$ laser tube. Calibrate speed and power with a test cut on your specific sheet stock before running production parts.

| Operation | Laser Power (%) | Speed (mm/s) | Passes | Focus / Assist |
| :--- | :--- | :--- | :--- | :--- |
| **Vector Through-Cut** | $80\% - 90\%$ | $10 - 15\text{ mm/s}$ | 1 | Strong air assist on surface; beam focused to material core ($3\text{ mm}$ depth). |
| **Vector Engrave (Labels)** | $20\% - 25\%$ | $50 - 70\text{ mm/s}$ | 1 | Moderate assist to prevent scorching. |
| **Kerf Compensation** | N/A | N/A | N/A | Nominal kerf offset: $\approx 0.15\text{ mm} - 0.20\text{ mm}$ for tight tab-and-slot joint interference fits. |
