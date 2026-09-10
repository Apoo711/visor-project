import adsk.core
import adsk.fusion
import traceback

def run(context):
    ui = None
    try:
        app = adsk.core.Application.get()
        ui = app.userInterface
        design = app.activeProduct
        if not design:
            ui.messageBox('No active Fusion design found.')
            return

        root_comp = design.rootComponent
        sketches = root_comp.sketches
        extrudes = root_comp.features.extrudeFeatures
        xy_plane = root_comp.xYConstructionPlane

        # -------------------------------------------------------------
        # Dimensions (Internal Fusion API units are in CENTIMETERS)
        # -------------------------------------------------------------
        # PCB board allowance: 25.0 mm x 24.0 mm (+ 0.6 mm print tolerance)
        inner_w = 2.56
        inner_h = 2.46
        wall_t = 0.20        # 2.0 mm wall thickness
        floor_t = 0.16       # 1.6 mm bottom thickness
        pocket_d = 0.50      # 5.0 mm depth to clear board + SMD components

        outer_w = inner_w + (2 * wall_t)
        outer_h = inner_h + (2 * wall_t)
        total_h = floor_t + pocket_d

        # Cable slot (bottom edge): 17.0 mm wide x 2.0 mm high
        slot_w = 1.70
        slot_h = 0.20

        # Lens aperture cutout: 10.0 mm x 10.0 mm
        lens_size = 1.00

        # Mounting hole standoffs (center spacing: 21.0 mm x 12.5 mm)
        hole_pitch_x = 2.10
        hole_pitch_y = 1.25
        standoff_od = 0.35   # 3.5 mm outer peg/post
        standoff_h = 0.18    # 1.8 mm standoff height

        # -------------------------------------------------------------
        # 1. Base Outer Shell Extrusion
        # -------------------------------------------------------------
        base_sketch = sketches.add(xy_plane)
        lines = base_sketch.sketchCurves.sketchLines
        lines.addTwoPointRectangle(
            adsk.core.Point3D.create(-outer_w / 2, -outer_h / 2, 0),
            adsk.core.Point3D.create(outer_w / 2, outer_h / 2, 0)
        )

        base_prof = base_sketch.profiles.item(0)
        ext_input = extrudes.createInput(base_prof, adsk.fusion.FeatureOperations.NewBodyFeatureOperation)
        ext_input.setDistanceExtent(False, adsk.core.ValueInput.createByReal(total_h))
        outer_body = extrudes.add(ext_input).bodies.item(0)

        # -------------------------------------------------------------
        # 2. Cut Inner Pocket
        # -------------------------------------------------------------
        pocket_sketch = sketches.add(xy_plane)
        pocket_lines = pocket_sketch.sketchCurves.sketchLines
        pocket_lines.addTwoPointRectangle(
            adsk.core.Point3D.create(-inner_w / 2, -inner_h / 2, 0),
            adsk.core.Point3D.create(inner_w / 2, inner_h / 2, 0)
        )

        pocket_prof = pocket_sketch.profiles.item(0)
        cut_input = extrudes.createInput(pocket_prof, adsk.fusion.FeatureOperations.CutFeatureOperation)
        
        # Start pocket cut at floor thickness, cut through to top
        start_from = adsk.fusion.OffsetStartDefinition.create(adsk.core.ValueInput.createByReal(floor_t))
        cut_input.startDefinition = start_from
        cut_input.setDistanceExtent(False, adsk.core.ValueInput.createByReal(pocket_d + 0.1))
        extrudes.add(cut_input)

        # -------------------------------------------------------------
        # 3. Cut Front Lens Aperture (Through Floor)
        # -------------------------------------------------------------
        lens_sketch = sketches.add(xy_plane)
        lens_lines = lens_sketch.sketchCurves.sketchLines
        lens_lines.addTwoPointRectangle(
            adsk.core.Point3D.create(-lens_size / 2, -lens_size / 2, 0),
            adsk.core.Point3D.create(lens_size / 2, lens_size / 2, 0)
        )

        lens_prof = lens_sketch.profiles.item(0)
        lens_cut = extrudes.createInput(lens_prof, adsk.fusion.FeatureOperations.CutFeatureOperation)
        lens_cut.setDistanceExtent(False, adsk.core.ValueInput.createByReal(floor_t + 0.1))
        extrudes.add(lens_cut)

        # -------------------------------------------------------------
        # 4. Cut Ribbon Cable Slot (Bottom Wall)
        # -------------------------------------------------------------
        slot_sketch = sketches.add(xy_plane)
        slot_lines = slot_sketch.sketchCurves.sketchLines
        slot_lines.addTwoPointRectangle(
            adsk.core.Point3D.create(-slot_w / 2, -outer_h / 2 - 0.1, 0),
            adsk.core.Point3D.create(slot_w / 2, -inner_h / 2 + 0.05, 0)
        )

        slot_prof = slot_sketch.profiles.item(0)
        slot_cut = extrudes.createInput(slot_prof, adsk.fusion.FeatureOperations.CutFeatureOperation)
        slot_start = adsk.fusion.OffsetStartDefinition.create(adsk.core.ValueInput.createByReal(floor_t))
        slot_cut.startDefinition = slot_start
        slot_cut.setDistanceExtent(False, adsk.core.ValueInput.createByReal(slot_h))
        extrudes.add(slot_cut)

        # -------------------------------------------------------------
        # 5. Add Internal Standoffs (Support Pins)
        # -------------------------------------------------------------
        standoff_sketch = sketches.add(xy_plane)
        circles = standoff_sketch.sketchCurves.sketchCircles

        dx = hole_pitch_x / 2
        dy = hole_pitch_y / 2
        positions = [(-dx, -dy), (dx, -dy), (-dx, dy), (dx, dy)]

        for px, py in positions:
            circles.addByCenterRadius(adsk.core.Point3D.create(px, py, 0), standoff_od / 2)

        for i in range(standoff_sketch.profiles.count):
            prof = standoff_sketch.profiles.item(i)
            peg_input = extrudes.createInput(prof, adsk.fusion.FeatureOperations.JoinFeatureOperation)
            peg_start = adsk.fusion.OffsetStartDefinition.create(adsk.core.ValueInput.createByReal(floor_t))
            peg_input.startDefinition = peg_start
            peg_input.setDistanceExtent(False, adsk.core.ValueInput.createByReal(standoff_h))
            extrudes.add(peg_input)

        ui.messageBox('Camera mount generated successfully.')

    except:
        if ui:
            ui.messageBox('Failed:\n{}'.format(traceback.format_exc()))