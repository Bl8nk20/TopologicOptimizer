class BoundaryConditionManager:
    def __init__(self, doc, analysis, geometry):
        self.doc = doc
        self.analysis = analysis
        self.geometry = geometry

    def add_fixed_constraint(self, face_ids):
        for face in face_ids:
            constraint = self.doc.addObject("Fem::ConstraintFixed", f"FixedFace{face}")
            constraint.Reference = [(self.geometry, f"Face{face}")]
            self.analysis.addObject(constraint)

    def add_force_constraint(self, face_ids, force_value, direction=(1, 0, 0)):
        for face in face_ids:
            load = self.doc.addObject("Fem::ConstraintForce", f"ForceFace{face}")
            load.Reference = [(self.geometry, f"Face{face}")]
            load.Force = force_value
            load.Direction = direction
            self.analysis.addObject(load)
