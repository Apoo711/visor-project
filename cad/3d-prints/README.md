# V.I.S.O.R. 3D-Printed Hardware Subsystem

This directory contains the 3D printable STL models and parametric generation script for the **V.I.S.O.R.** electromechanical rack-and-pinion dispensing assemblies.

The mechanical design solves the critical failure mode observed in traditional gravity and spiral vending systems: **flexible medical packaging (such as bandage wrappers and alcohol pads) jamming due to surface friction and compression.** By utilizing a positive-thrust linear rack driven by continuous-rotation micro servos, each item is positively ejected out of its cartridge.

---

## Component Overview & STL Inventory

### 1. Medical Supply Cartridge (`cartridge/`)
Vertical hopper designed to hold a 5-unit stack of first aid supplies ($53\text{ mm} \times 63\text{ mm} \times 7\text{ mm}$ per unit) with calibrated side clearances, bottom egress slot ($8\text{ mm}$ height), and rear aperture for the pusher sled.

| File | Interactive 3D View | Description & Revisions |
| :--- | :--- | :--- |
| **`Version_1.stl`** | [View in GitHub 3D Viewer](https://github.com/Apoo711/visor-project/blob/main/cad/3d-prints/cartridge/Version_1.stl) | Initial prototype cartridge geometry. |
| **`Version_2.stl`** | [View in GitHub 3D Viewer](https://github.com/Apoo711/visor-project/blob/main/cad/3d-prints/cartridge/Version_2.stl) | Optimized shell thickness ($3.0\text{ mm}$), reinforced side walls, calibrated $1.5\text{ mm}$ running tolerance, and rear cutout ($20\text{ mm} \times 15\text{ mm}$) for seamless sled entry. |

---

### 2. FEETECH FS90R Pinion Gear (`gear/`)
Continuous-rotation servo pinion engineered to mesh with the linear rack.

| File | Interactive 3D View | Description & Specifications |
| :--- | :--- | :--- |
| **`Version_1.stl`** | [View in GitHub 3D Viewer](https://github.com/Apoo711/visor-project/blob/main/cad/3d-prints/gear/Version_1.stl) | 12-tooth spur pinion gear ($4.0\text{ mm}$ pitch, $1.8\text{ mm}$ tooth depth, $6.0\text{ mm}$ face width) with central $4.8\text{ mm}$ press-fit spline bore directly compatible with FEETECH FS90R servo shafts. |

---

### 3. Linear Rack & Pusher Sled (`rack/`)
Linear rack arm with an integrated pusher sled. Features an anti-slip retaining guide rail on the $-X$ edge to prevent the servo pinion from walking or slipping axially under load.

| File | Interactive 3D View | Description & Revisions |
| :--- | :--- | :--- |
| **`Version_1_1.stl`** | [View in GitHub 3D Viewer](https://github.com/Apoo711/visor-project/blob/main/cad/3d-prints/rack/Version_1_1.stl) | Early linear rack prototype segment. |
| **`Version_1_2.stl`** | [View in GitHub 3D Viewer](https://github.com/Apoo711/visor-project/blob/main/cad/3d-prints/rack/Version_1_2.stl) | Extended rack iteration with basic pusher contact surface. |
| **`Version_2.stl`** | [View in GitHub 3D Viewer](https://github.com/Apoo711/visor-project/blob/main/cad/3d-prints/rack/Version_2.stl) | **Current Production Model**: $120\text{ mm}$ stroke arm, $3.5\text{ mm}$ chamfered leading wedge to prevent packaging snagging, integrated $2.0\text{ mm}$ retaining guide rail with $0.5\text{ mm}$ running clearance, and servo standoff mounts with $27.5\text{ mm}$ hole pitch. |

---

## Parametric Generation Script: [`generate.py`](./generate.py)

All current mechanical components can be generated and modified directly in **Autodesk Fusion 360** using the bundled parametric Python script:

- **Script location**: [`cad/3d-prints/generate.py`](./generate.py)
- **Key Parameters**:
  - `item_width` ($53.0\text{ mm}$), `item_length` ($63.0\text{ mm}$), `item_thickness` ($7.0\text{ mm}$)
  - `stack_count` ($5$ units) $\rightarrow$ $35.0\text{ mm}$ internal height
  - `sled_chamfer` ($3.5\text{ mm}$) leading wedge to separate bottom item from the gravity stack
  - `rail_thickness` ($2.0\text{ mm}$), `rail_height` ($3.5\text{ mm}$) anti-slip retaining guide rail
  - `servo_hole_pitch` ($27.5\text{ mm}$) for M2 screws

### Running in Autodesk Fusion 360:
1. Open Autodesk Fusion 360.
2. Navigate to **UTILITIES** $\rightarrow$ **Scripts and Add-Ins** (or press `Shift + S`).
3. Under the **Scripts** tab, click the **+** (Add) button next to *My Scripts*.
4. Select the [`generate.py`](./generate.py) file from this directory.
5. Click **Run** to automatically generate the parameterized 3D geometry in your active workspace.

---

## Recommended 3D Printing Slicer Settings

| Parameter | Recommended Value | Rationale |
| :--- | :--- | :--- |
| **Filament Material** | PLA / PLA+ / PETG | High dimensional accuracy, minimal thermal warping on flat sliding surfaces. |
| **Layer Height** | $0.16\text{ mm}$ - $0.20\text{ mm}$ | Smooth tooth engagement on the gear and rack; low friction for the sled. |
| **Infill Percentage** | $25\% - 40\%$ | High rigidity required for the rack arm and gear teeth under continuous torque. |
| **Infill Pattern** | Gyroid or Grid | Uniform multi-axial mechanical strength. |
| **Wall / Perimeter Count** | $3 - 4$ perimeters | Ensures gear teeth and rack guide rails are printed solid for maximum shear strength. |
| **Print Orientation** | Flat on bed | - **Cartridge**: Print upright on its bottom base.<br>- **Rack**: Print flat along the arm axis for optimal layer shear resistance.<br>- **Gear**: Print flat on either face. |
| **Supports** | Minimal / Tree supports | Only required under the cartridge front ejection slot bridge. |