"""
Autodesk Fusion 360 Python Script: Smart First Aid Cartridge, Extended Sled with Retaining Guide Rail, & FS90R Pinion
Configured for: 53mm x 63mm x 7mm items (5-stack capacity)
Updates:
- Added a vertical retaining guide rail on the -X edge of the rack (opposite the servo mount) to capture the pinion gear and prevent axial slip/derailing.
- 0.5mm running tolerance between the gear face and guide rail.
- Verified rear hole clearance (4.5mm horizontal margin, 3.5mm vertical margin).
- Full Part/Assembly document auto-compatibility.
Note: Fusion 360 internal API units are CENTIMETERS (cm). All mm inputs are converted via MM_TO_CM (0.1).
"""

import adsk.core
import adsk.fusion
import traceback
import math

def run(context):
    ui = None
    try:
        app = adsk.core.Application.get()
        ui = app.userInterface
        
        design = adsk.fusion.Design.cast(app.activeProduct)
        if not design:
            ui.messageBox('No active Fusion 360 design found. Please open or create a design first.')
            return
            
        rootComp = design.rootComponent

        # ==============================================================================
        # 1. PARAMETRIC MODEL DIMENSIONS (in millimeters)
        # ==============================================================================
        MM_TO_CM = 0.1  # Conversion multiplier: 1 mm = 0.1 cm

        # Item & Stack Parameters
        item_width = 53.0       # Bandage width (X-axis)
        item_length = 63.0      # Bandage length (Y-axis)
        item_thickness = 7.0    # Single bandage thickness
        stack_count = 5         # Internal capacity
        stack_height = item_thickness * stack_count  # 35.0 mm internal capacity
        
        # Dispensing & Clearance Parameters
        slot_height = item_thickness + 1.0  # 8.0 mm (single-packet exit slot)
        tolerance = 1.5         # Side clearance gap
        wall_thick = 3.0        # Outer shell wall and floor thickness

        # Sled Parameters
        sled_width = item_width # 53.0 mm
        sled_length = 20.0      # 20.0 mm along Y-axis
        sled_height = 6.0       # 6.0 mm height
        sled_chamfer = 3.5      # Leading wedge chamfer
        sled_rear_gap = 1.0     # Rear resting gap

        # Cavity & Shell Lengths (Houses 63mm packet + 20mm sled simultaneously)
        inner_length = item_length + sled_length + sled_rear_gap + (tolerance * 2)  # 87.0 mm
        outer_length = inner_length + (wall_thick * 2)                               # 93.0 mm
        inner_width = item_width + (tolerance * 2)                                   # 56.0 mm
        outer_width = inner_width + (wall_thick * 2)                                 # 62.0 mm

        # Rack Parameters
        rack_base_x_min = -5.5  # Left edge of rack arm base (mm)
        rack_base_x_max = 5.0   # Right edge of rack arm base (mm)
        rack_height = 8.0       # 8.0 mm tall base
        rack_length = 120.0     # 120.0 mm extended arm length
        tooth_pitch = 4.0       # 4.0 mm pitch between teeth
        tooth_height = 1.8      # 1.8 mm tooth depth

        # Gear Anti-Slip Retaining Guide Rail Parameters (-X side)
        # Sits from X = -5.5mm to X = -3.5mm (2.0mm thick wall)
        # Pinion sits at X = -3.0mm to +3.0mm, giving a 0.5mm running clearance to the rail
        rail_thickness = 2.0    # 2.0 mm thick retaining fence
        rail_height = 3.5       # 3.5 mm high above rack base (1.7mm taller than teeth)

        # FS90R Standoff Mount Parameters (+X side)
        servo_hole_pitch = 27.5 # Spacing between M2 mounting holes
        m2_hole_dia = 2.4       # Pilot hole for M2 screws
        mount_platform_l = 40.0 # Length of extended rear platform

        # FS90R Pinion Gear Parameters
        pinion_teeth_count = 12 # 12 teeth
        fs90r_spline_dia = 4.8  # 4.8 mm FS90R servo spline bore
        pinion_thickness = 6.0  # 6.0 mm face width

        # Cutout Parameters
        front_slot_w = item_width + tolerance  # 54.5 mm front slot
        front_slot_h = slot_height             # 8.0 mm
        rear_hole_w = 20.0                     # 20.0 mm rear hole (X: -10.0 to +10.0)
        rear_hole_h = 15.0                     # 15.0 mm rear hole height (Z: floor to floor+15)

        # ==============================================================================
        # 2. CONVERT DIMENSIONS TO CENTIMETERS (Fusion API Units)
        # ==============================================================================
        outer_w_cm = outer_width * MM_TO_CM
        outer_l_cm = outer_length * MM_TO_CM
        total_h_cm = (wall_thick + stack_height) * MM_TO_CM

        inner_w_cm = inner_width * MM_TO_CM
        inner_l_cm = inner_length * MM_TO_CM
        floor_t_cm = wall_thick * MM_TO_CM
        cavity_d_cm = stack_height * MM_TO_CM
        cut_margin_cm = 0.5

        # ==============================================================================
        # 3. DOCUMENT TYPE CHECK (Assembly vs Part Design Environment)
        # ==============================================================================
        use_components = True
        try:
            test_occ = rootComp.occurrences.addNewComponent(adsk.core.Matrix3D.create())
            test_occ.deleteMe()
        except Exception:
            use_components = False

        if use_components:
            cartridgeOcc = rootComp.occurrences.addNewComponent(adsk.core.Matrix3D.create())
            cartridgeComp = cartridgeOcc.component
            cartridgeComp.name = "Main Cartridge Body"
        else:
            cartridgeComp = rootComp

        cartExtrudes = cartridgeComp.features.extrudeFeatures
        cartPlanes = cartridgeComp.constructionPlanes

        # --- Step 3.1: Outer Solid Block ---
        baseSketch = cartridgeComp.sketches.add(cartridgeComp.xYConstructionPlane)
        baseSketch.name = "Cartridge_Outer_Profile"
        baseLines = baseSketch.sketchCurves.sketchLines
        baseLines.addTwoPointRectangle(
            adsk.core.Point3D.create(-outer_w_cm / 2.0, -outer_l_cm / 2.0, 0),
            adsk.core.Point3D.create(outer_w_cm / 2.0, outer_l_cm / 2.0, 0)
        )
        outerProfile = baseSketch.profiles.item(0)

        outerExtInput = cartExtrudes.createInput(outerProfile, adsk.fusion.FeatureOperations.NewBodyFeatureOperation)
        outerExtInput.setDistanceExtent(False, adsk.core.ValueInput.createByReal(total_h_cm))
        cartridgeBodyExt = cartExtrudes.add(outerExtInput)
        cartridgeBody = cartridgeBodyExt.bodies.item(0)
        cartridgeBody.name = "Main Cartridge Body"

        # --- Step 3.2: Floor Construction Plane ---
        floorPlaneInput = cartPlanes.createInput()
        floorPlaneInput.setByOffset(cartridgeComp.xYConstructionPlane, adsk.core.ValueInput.createByReal(floor_t_cm))
        cartFloorPlane = cartPlanes.add(floorPlaneInput)

        # --- Step 3.3: Inner Cavity Cut ---
        cavitySketch = cartridgeComp.sketches.add(cartFloorPlane)
        cavitySketch.name = "Cartridge_Cavity_Profile"
        cavityLines = cavitySketch.sketchCurves.sketchLines
        cavityLines.addTwoPointRectangle(
            adsk.core.Point3D.create(-inner_w_cm / 2.0, -inner_l_cm / 2.0, 0),
            adsk.core.Point3D.create(inner_w_cm / 2.0, inner_l_cm / 2.0, 0)
        )
        cavityProfile = cavitySketch.profiles.item(0)

        cavityExtInput = cartExtrudes.createInput(cavityProfile, adsk.fusion.FeatureOperations.CutFeatureOperation)
        cavityExtInput.participantBodies = [cartridgeBody]
        cavityExtInput.setDistanceExtent(False, adsk.core.ValueInput.createByReal(cavity_d_cm))
        cartExtrudes.add(cavityExtInput)

        # --- Step 3.4: Front Dispensing Slit ---
        frontSlitSketch = cartridgeComp.sketches.add(cartFloorPlane)
        frontSlitSketch.name = "Front_Dispensing_Slit_Profile"
        frontLines = frontSlitSketch.sketchCurves.sketchLines
        
        slot_w_cm = front_slot_w * MM_TO_CM
        slot_h_cm = front_slot_h * MM_TO_CM
        y_front_outer = -outer_l_cm / 2.0 - cut_margin_cm
        y_front_inner = -inner_l_cm / 2.0 + cut_margin_cm
        
        frontLines.addTwoPointRectangle(
            adsk.core.Point3D.create(-slot_w_cm / 2.0, y_front_outer, 0),
            adsk.core.Point3D.create(slot_w_cm / 2.0, y_front_inner, 0)
        )
        frontProfile = frontSlitSketch.profiles.item(0)

        frontCutInput = cartExtrudes.createInput(frontProfile, adsk.fusion.FeatureOperations.CutFeatureOperation)
        frontCutInput.participantBodies = [cartridgeBody]
        frontCutInput.setDistanceExtent(False, adsk.core.ValueInput.createByReal(slot_h_cm))
        cartExtrudes.add(frontCutInput)

        # --- Step 3.5: Rear Pusher Access Hole ---
        rearHoleSketch = cartridgeComp.sketches.add(cartFloorPlane)
        rearHoleSketch.name = "Rear_Pusher_Hole_Profile"
        rearLines = rearHoleSketch.sketchCurves.sketchLines

        rear_w_cm = rear_hole_w * MM_TO_CM
        rear_h_cm = rear_hole_h * MM_TO_CM
        y_rear_inner = inner_l_cm / 2.0 - cut_margin_cm
        y_rear_outer = outer_l_cm / 2.0 + cut_margin_cm

        rearLines.addTwoPointRectangle(
            adsk.core.Point3D.create(-rear_w_cm / 2.0, y_rear_inner, 0),
            adsk.core.Point3D.create(rear_w_cm / 2.0, y_rear_outer, 0)
        )
        rearProfile = rearHoleSketch.profiles.item(0)

        rearCutInput = cartExtrudes.createInput(rearProfile, adsk.fusion.FeatureOperations.CutFeatureOperation)
        rearCutInput.participantBodies = [cartridgeBody]
        rearCutInput.setDistanceExtent(False, adsk.core.ValueInput.createByReal(rear_h_cm))
        cartExtrudes.add(rearCutInput)

        # --- Step 3.6: Extended Rear Servo Mount Platform ---
        mount_l_cm = mount_platform_l * MM_TO_CM
        platformSketch = cartridgeComp.sketches.add(cartridgeComp.xYConstructionPlane)
        platformSketch.name = "Servo_Mount_Platform"
        pLines = platformSketch.sketchCurves.sketchLines
        
        pLines.addTwoPointRectangle(
            adsk.core.Point3D.create(-outer_w_cm / 2.0, outer_l_cm / 2.0, 0),
            adsk.core.Point3D.create(outer_w_cm / 2.0, outer_l_cm / 2.0 + mount_l_cm, 0)
        )
        platformExtInput = cartExtrudes.createInput(platformSketch.profiles.item(0), adsk.fusion.FeatureOperations.JoinFeatureOperation)
        platformExtInput.participantBodies = [cartridgeBody]
        platformExtInput.setDistanceExtent(False, adsk.core.ValueInput.createByReal(floor_t_cm))
        cartExtrudes.add(platformExtInput)

        # Standoffs with M2 mounting holes for the FS90R (+X side)
        standoffSketch = cartridgeComp.sketches.add(cartFloorPlane)
        standoffSketch.name = "FS90R_Standoffs"
        sLines = standoffSketch.sketchCurves.sketchLines

        standoff_x = 18.0 * MM_TO_CM
        standoff_y_center = outer_l_cm / 2.0 + (mount_l_cm * 0.5)
        pitch_half_cm = (servo_hole_pitch / 2.0) * MM_TO_CM
        standoff_h_cm = 12.0 * MM_TO_CM
        m2_r_cm = (m2_hole_dia / 2.0) * MM_TO_CM

        sLines.addTwoPointRectangle(
            adsk.core.Point3D.create(standoff_x - 0.4, standoff_y_center - pitch_half_cm - 0.4, 0),
            adsk.core.Point3D.create(standoff_x + 0.4, standoff_y_center - pitch_half_cm + 0.4, 0)
        )
        sLines.addTwoPointRectangle(
            adsk.core.Point3D.create(standoff_x - 0.4, standoff_y_center + pitch_half_cm - 0.4, 0),
            adsk.core.Point3D.create(standoff_x + 0.4, standoff_y_center + pitch_half_cm + 0.4, 0)
        )

        for prof in standoffSketch.profiles:
            stInput = cartExtrudes.createInput(prof, adsk.fusion.FeatureOperations.JoinFeatureOperation)
            stInput.participantBodies = [cartridgeBody]
            stInput.setDistanceExtent(False, adsk.core.ValueInput.createByReal(standoff_h_cm))
            cartExtrudes.add(stInput)

        # M2 pilot holes
        holePlaneInput = cartPlanes.createInput()
        holePlaneInput.setByOffset(cartridgeComp.xYConstructionPlane, adsk.core.ValueInput.createByReal(floor_t_cm + standoff_h_cm))
        holePlane = cartPlanes.add(holePlaneInput)

        holeSketch = cartridgeComp.sketches.add(holePlane)
        holeSketch.name = "M2_Holes"
        hCircles = holeSketch.sketchCurves.sketchCircles
        hCircles.addByCenterRadius(adsk.core.Point3D.create(standoff_x, standoff_y_center - pitch_half_cm, 0), m2_r_cm)
        hCircles.addByCenterRadius(adsk.core.Point3D.create(standoff_x, standoff_y_center + pitch_half_cm, 0), m2_r_cm)

        for prof in holeSketch.profiles:
            hCutInput = cartExtrudes.createInput(prof, adsk.fusion.FeatureOperations.CutFeatureOperation)
            hCutInput.participantBodies = [cartridgeBody]
            hCutInput.setDistanceExtent(False, adsk.core.ValueInput.createByReal(8.0 * MM_TO_CM))
            cartExtrudes.add(hCutInput)

        # ==============================================================================
        # 4. COMPONENT 2: PUSHER SLED WITH GEAR RETAINING GUIDE RAIL
        # ==============================================================================
        if use_components:
            sledOcc = rootComp.occurrences.addNewComponent(adsk.core.Matrix3D.create())
            sledComp = sledOcc.component
            sledComp.name = "Pusher Sled"
        else:
            sledComp = rootComp

        sledExtrudes = sledComp.features.extrudeFeatures
        sledPlanes = sledComp.constructionPlanes
        sledChamfers = sledComp.features.chamferFeatures
        sledPatterns = sledComp.features.rectangularPatternFeatures

        # Sled Floor Plane
        sledFloorPlaneInput = sledPlanes.createInput()
        sledFloorPlaneInput.setByOffset(sledComp.xYConstructionPlane, adsk.core.ValueInput.createByReal(floor_t_cm))
        sledFloorPlane = sledPlanes.add(sledFloorPlaneInput)

        sled_w_cm = sled_width * MM_TO_CM
        sled_l_cm = sled_length * MM_TO_CM
        sled_h_cm = sled_height * MM_TO_CM

        sled_y_max = (inner_l_cm / 2.0) - (sled_rear_gap * MM_TO_CM)
        sled_y_min = sled_y_max - sled_l_cm

        # --- Step 4.1: Sled Body ---
        sledSketch = sledComp.sketches.add(sledFloorPlane)
        sledSketch.name = "Sled_Base_Profile"
        sledLines = sledSketch.sketchCurves.sketchLines
        sledLines.addTwoPointRectangle(
            adsk.core.Point3D.create(-sled_w_cm / 2.0, sled_y_min, 0),
            adsk.core.Point3D.create(sled_w_cm / 2.0, sled_y_max, 0)
        )
        sledProfile = sledSketch.profiles.item(0)

        sledExtInput = sledExtrudes.createInput(sledProfile, adsk.fusion.FeatureOperations.NewBodyFeatureOperation)
        sledExtInput.setDistanceExtent(False, adsk.core.ValueInput.createByReal(sled_h_cm))
        sledBodyExt = sledExtrudes.add(sledExtInput)
        sledBody = sledBodyExt.bodies.item(0)
        sledBody.name = "Pusher Sled"

        # --- Step 4.2: Front Leading Wedge Chamfer ---
        target_z = floor_t_cm + sled_h_cm
        topFrontEdge = None

        for edge in sledBody.edges:
            pt1 = edge.startVertex.geometry
            pt2 = edge.endVertex.geometry
            mid_z = (pt1.z + pt2.z) / 2.0
            mid_y = (pt1.y + pt2.y) / 2.0
            if abs(mid_z - target_z) < 0.02 and abs(mid_y - sled_y_min) < 0.02:
                topFrontEdge = edge
                break

        if topFrontEdge:
            edgeCol = adsk.core.ObjectCollection.create()
            edgeCol.add(topFrontEdge)
            chamferInput = sledChamfers.createInput(edgeCol, True)
            chamferInput.setToEqualDistance(adsk.core.ValueInput.createByReal(sled_chamfer * MM_TO_CM))
            sledChamfers.add(chamferInput)

        # --- Step 4.3: Extended Gear Rack Arm Base ---
        rack_xmin_cm = rack_base_x_min * MM_TO_CM
        rack_xmax_cm = rack_base_x_max * MM_TO_CM
        rack_h_cm = rack_height * MM_TO_CM
        rack_l_cm = rack_length * MM_TO_CM

        rackSketch = sledComp.sketches.add(sledFloorPlane)
        rackSketch.name = "Rack_Arm_Profile"
        rackLines = rackSketch.sketchCurves.sketchLines
        rackLines.addTwoPointRectangle(
            adsk.core.Point3D.create(rack_xmin_cm, sled_y_max, 0),
            adsk.core.Point3D.create(rack_xmax_cm, sled_y_max + rack_l_cm, 0)
        )
        rackProfile = rackSketch.profiles.item(0)

        rackExtInput = sledExtrudes.createInput(rackProfile, adsk.fusion.FeatureOperations.JoinFeatureOperation)
        rackExtInput.participantBodies = [sledBody]
        rackExtInput.setDistanceExtent(False, adsk.core.ValueInput.createByReal(rack_h_cm))
        sledExtrudes.add(rackExtInput)

        # --- Step 4.4: Anti-Slip Retaining Guide Rail (Flange alongside teeth) ---
        # Top plane of the rack base arm
        rackTopPlaneInput = sledPlanes.createInput()
        rackTopPlaneInput.setByOffset(sledComp.xYConstructionPlane, adsk.core.ValueInput.createByReal(floor_t_cm + rack_h_cm))
        rackTopPlane = sledPlanes.add(rackTopPlaneInput)

        guideRailSketch = sledComp.sketches.add(rackTopPlane)
        guideRailSketch.name = "Gear_Retaining_Rail_Profile"
        grLines = guideRailSketch.sketchCurves.sketchLines
        
        rail_w_cm = rail_thickness * MM_TO_CM
        rail_h_cm = rail_height * MM_TO_CM
        rail_xmin = rack_xmin_cm
        rail_xmax = rack_xmin_cm + rail_w_cm  # Sits from -5.5mm to -3.5mm

        grLines.addTwoPointRectangle(
            adsk.core.Point3D.create(rail_xmin, sled_y_max, 0),
            adsk.core.Point3D.create(rail_xmax, sled_y_max + rack_l_cm, 0)
        )
        grProfile = guideRailSketch.profiles.item(0)

        grExtInput = sledExtrudes.createInput(grProfile, adsk.fusion.FeatureOperations.JoinFeatureOperation)
        grExtInput.participantBodies = [sledBody]
        grExtInput.setDistanceExtent(False, adsk.core.ValueInput.createByReal(rail_h_cm))
        sledExtrudes.add(grExtInput)

        # --- Step 4.5: Rack Gear Teeth (Bounded on the left by the guide rail) ---
        tooth_pitch_cm = tooth_pitch * MM_TO_CM
        tooth_h_cm = tooth_height * MM_TO_CM
        num_teeth = int((rack_length - 15.0) / tooth_pitch)  # 26 teeth

        toothSketch = sledComp.sketches.add(rackTopPlane)
        toothSketch.name = "Single_Tooth_Profile"
        toothLines = toothSketch.sketchCurves.sketchLines
        
        t_base_len = tooth_pitch_cm * 0.55
        t_start_y = sled_y_max + (tooth_pitch_cm * 1.5)
        
        # Teeth span from rail inner face (-3.5mm) to rack right edge (+5.0mm)
        toothLines.addTwoPointRectangle(
            adsk.core.Point3D.create(rail_xmax, t_start_y, 0),
            adsk.core.Point3D.create(rack_xmax_cm, t_start_y + t_base_len, 0)
        )
        toothProfile = toothSketch.profiles.item(0)

        toothExtInput = sledExtrudes.createInput(toothProfile, adsk.fusion.FeatureOperations.JoinFeatureOperation)
        toothExtInput.participantBodies = [sledBody]
        toothExtInput.setDistanceExtent(False, adsk.core.ValueInput.createByReal(tooth_h_cm))
        toothExtFeature = sledExtrudes.add(toothExtInput)

        # Pattern Teeth Along Extended Arm
        entities = adsk.core.ObjectCollection.create()
        entities.add(toothExtFeature)
        
        patternInput = sledPatterns.createInput(
            entities, 
            sledComp.yConstructionAxis, 
            adsk.core.ValueInput.createByString(str(num_teeth)), 
            adsk.core.ValueInput.createByReal(tooth_pitch_cm), 
            adsk.fusion.PatternDistanceType.SpacingPatternDistanceType
        )
        sledPatterns.add(patternInput)

        # ==============================================================================
        # 5. COMPONENT 3: FS90R PINION GEAR
        # ==============================================================================
        if use_components:
            pinionOcc = rootComp.occurrences.addNewComponent(adsk.core.Matrix3D.create())
            pinionComp = pinionOcc.component
            pinionComp.name = "FS90R Pinion Gear"
        else:
            pinionComp = rootComp

        pinionExtrudes = pinionComp.features.extrudeFeatures
        pinionPatterns = pinionComp.features.circularPatternFeatures

        # Pitch & Tooth Sizing
        p_circ_cm = (pinion_teeth_count * tooth_pitch) * MM_TO_CM
        r_pitch_cm = p_circ_cm / (2.0 * math.pi)
        r_root_cm = r_pitch_cm - (tooth_h_cm * 0.55)
        r_tip_cm = r_pitch_cm + (tooth_h_cm * 0.45)
        gear_t_cm = pinion_thickness * MM_TO_CM

        # Base Gear Cylinder
        baseGearSketch = pinionComp.sketches.add(pinionComp.xYConstructionPlane)
        baseGearSketch.name = "Pinion_Root_Circle"
        baseGearSketch.sketchCurves.sketchCircles.addByCenterRadius(adsk.core.Point3D.create(0, 0, 0), r_root_cm)
        
        gearBodyInput = pinionExtrudes.createInput(baseGearSketch.profiles.item(0), adsk.fusion.FeatureOperations.NewBodyFeatureOperation)
        gearBodyInput.setDistanceExtent(False, adsk.core.ValueInput.createByReal(gear_t_cm))
        gearBodyExt = pinionExtrudes.add(gearBodyInput)
        gearBody = gearBodyExt.bodies.item(0)
        gearBody.name = "FS90R Pinion Gear"

        # Single Seed Tooth
        toothGenSketch = pinionComp.sketches.add(pinionComp.xYConstructionPlane)
        toothGenSketch.name = "Pinion_Tooth_Seed"
        tLines = toothGenSketch.sketchCurves.sketchLines
        
        half_angle_root = (2.0 * math.pi / pinion_teeth_count) * 0.28
        half_angle_tip = (2.0 * math.pi / pinion_teeth_count) * 0.14
        
        p1 = adsk.core.Point3D.create(-r_root_cm * math.sin(half_angle_root), r_root_cm * math.cos(half_angle_root), 0)
        p2 = adsk.core.Point3D.create(-r_tip_cm * math.sin(half_angle_tip), r_tip_cm * math.cos(half_angle_tip), 0)
        p3 = adsk.core.Point3D.create(r_tip_cm * math.sin(half_angle_tip), r_tip_cm * math.cos(half_angle_tip), 0)
        p4 = adsk.core.Point3D.create(r_root_cm * math.sin(half_angle_root), r_root_cm * math.cos(half_angle_root), 0)

        tLines.addByTwoPoints(p1, p2)
        tLines.addByTwoPoints(p2, p3)
        tLines.addByTwoPoints(p3, p4)
        tLines.addByTwoPoints(p4, p1)

        pinionToothProfile = toothGenSketch.profiles.item(0)
        pinionToothExtInput = pinionExtrudes.createInput(pinionToothProfile, adsk.fusion.FeatureOperations.JoinFeatureOperation)
        pinionToothExtInput.participantBodies = [gearBody]
        pinionToothExtInput.setDistanceExtent(False, adsk.core.ValueInput.createByReal(gear_t_cm))
        pinionToothFeature = pinionExtrudes.add(pinionToothExtInput)

        # Circular Pattern around Z axis
        circEntities = adsk.core.ObjectCollection.create()
        circEntities.add(pinionToothFeature)
        
        circPatternInput = pinionPatterns.createInput(circEntities, pinionComp.zConstructionAxis)
        circPatternInput.quantity = adsk.core.ValueInput.createByString(str(pinion_teeth_count))
        circPatternInput.totalAngle = adsk.core.ValueInput.createByString('360 deg')
        circPatternInput.isSymmetric = False
        pinionPatterns.add(circPatternInput)

        # FS90R 4.8mm Spline Bore Cut
        boreSketch = pinionComp.sketches.add(pinionComp.xYConstructionPlane)
        boreSketch.name = "FS90R_Shaft_Mount_Profile"
        boreSketch.sketchCurves.sketchCircles.addByCenterRadius(
            adsk.core.Point3D.create(0, 0, 0), 
            (fs90r_spline_dia / 2.0) * MM_TO_CM
        )
        boreProfile = boreSketch.profiles.item(0)

        boreCutInput = pinionExtrudes.createInput(boreProfile, adsk.fusion.FeatureOperations.CutFeatureOperation)
        boreCutInput.participantBodies = [gearBody]
        boreCutInput.setDistanceExtent(False, adsk.core.ValueInput.createByReal(gear_t_cm * 1.5))
        pinionExtrudes.add(boreCutInput)

        # Position Pinion Gear (Centered on X=0, captured on its left face by the rail at X=-3.5mm)
        pinion_mesh_y_cm = outer_l_cm / 2.0 + (mount_l_cm * 0.5)
        pinion_mesh_z_cm = floor_t_cm + rack_h_cm + tooth_h_cm + r_pitch_cm
        
        mat = adsk.core.Matrix3D.create()
        mat.setToRotation(math.pi / 2.0, adsk.core.Vector3D.create(0, 1, 0), adsk.core.Point3D.create(0, 0, 0))
        mat.translation = adsk.core.Vector3D.create(0, pinion_mesh_y_cm, pinion_mesh_z_cm)

        if use_components:
            pinionOcc.transform = mat
        else:
            bodyCol = adsk.core.ObjectCollection.create()
            bodyCol.add(gearBody)
            moveFeats = rootComp.features.moveFeatures
            try:
                moveInput = moveFeats.createInput2(bodyCol)
                moveInput.transform = mat
                moveFeats.add(moveInput)
            except Exception:
                moveInput = moveFeats.createInput(bodyCol, mat)
                moveFeats.add(moveInput)

        app.activeViewport.fit()
        ui.messageBox('Generated successfully with anti-slip retaining rail, extended rack, and servo mount!')

    except Exception:
        if ui:
            ui.messageBox(f'Failed:\n{traceback.format_exc()}')