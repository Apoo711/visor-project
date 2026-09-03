# V.I.S.O.R. CAD & Mechanical Design Subsystem

This directory contains the computer-aided design (CAD) models, parametric generation scripts, and manufacturing files for the **V.I.S.O.R. (Visual Inspection & Smart Occupational Relief)** physical kiosk and dispensing electromechanics.

---

## Architecture & Subdirectories

```
cad/
├── 3d-prints/           # Additive manufacturing components (FDM PLA)
│   ├── cartridge/       # Modular 5-stack medical supply cartridges
│   ├── gear/            # FEETECH FS90R servo pinions
│   ├── rack/            # Positive-thrust linear rack & pusher sleds
│   ├── generate.py      # Autodesk Fusion 360 parametric CAD generation script
│   └── README.md        # 3D printing documentation, specifications & viewer links
└── laser-cutting/       # Subtractive manufacturing (6mm Plywood chassis)
    └── README.md        # Enclosure structural panels & DXF specifications
```

### 1. [`3d-prints/`](./3d-prints/)
Houses the modular electromechanical dispensing hardware designed for FDM 3D printing in PLA bioplastic:
- **Cartridge (`cartridge/`)**: Houses stacked medical supplies (bandages, alcohol wipes, gauze) with integrated dispensing gates and mounting points.
- **Pinion Gear (`gear/`)**: Custom continuous-rotation servo gear designed with a standard 4.8mm spline bore for direct coupling to the FEETECH FS90R servo.
- **Rack & Sled (`rack/`)**: Linear rack featuring an anti-slip retaining guide rail and an integrated pusher sled with a chamfered leading wedge to eliminate flexible packet friction jams.
- **Parametric Generator (`generate.py`)**: An end-to-end Python script executed within Autodesk Fusion 360 to generate all cartridge, rack, and gear components programmatically.

### 2. [`laser-cutting/`](./laser-cutting/)
Houses the laser-cut and CNC sheet profiles (DXF format) for the structural kiosk:
- **6mm Plywood Chassis**: Front bezel with screen & camera apertures, side panels, internal shelf brackets, and dispensing bay trays.

---

## Dispensing Mechanism Overview

| Component | Manufacturing | Material | Purpose |
| :--- | :--- | :--- | :--- |
| **Cartridge Body** | 3D Print (FDM) | PLA (Black/White) | Retains a 5-unit vertical gravity stack of medical supplies with calibrated friction fit |
| **Drive Pinion** | 3D Print (FDM) | Tough PLA / PETG | 12-tooth circular gear coupled to continuous-rotation FS90R servo output spline |
| **Linear Rack & Sled** | 3D Print (FDM) | PLA (0.2mm layer) | Translates rotation into linear positive thrust, sliding under the stack to eject the bottom item |
| **Enclosure Chassis** | Laser Cutting / CNC | 6mm Pine Plywood | Rigid kiosk structure housing Pi 4, Arduino Uno, power distribution, and interactive display |

For detailed 3D printing parameters, STL model previews, and print orientation guidelines, refer to the [3D Prints Documentation](./3d-prints/README.md).
