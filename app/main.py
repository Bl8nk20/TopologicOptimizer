import FreeCAD
from fem.geometry import GeometryManager
from fem.boundary_conditions import BoundaryConditionManager
from fem.solver import SolverManager
from fem.results import ResultsManager


def main():
    doc = FreeCAD.newDocument("TopOptModular")

    # Geometrie erstellen
    box = doc.addObject("Part::Box", "Box")
    box.Length = 100
    box.Width = 50
    box.Height = 10
    doc.recompute()

    # FEM Setup
    geo = GeometryManager(doc)
    analysis = geo.create_analysis()
    geo.add_geometry(box)
    geo.add_material()
    mesh = geo.create_mesh()

    # Randbedingungen
    bc = BoundaryConditionManager(doc, analysis, box)
    bc.add_fixed_constraint(face_ids=[1])
    bc.add_force_constraint(face_ids=[2], force_value=1000)

    # Solver
    solver = SolverManager(doc, analysis)
    solver.setup_calculix_solver()
    solver.run_solver()

    # Ergebnisse
    results = ResultsManager(mesh, analysis)
    coords, connectivity, _, _ = results.get_mesh_data()
    displacements = results.get_displacement_field()

    print("Displacements:", displacements)


if __name__ == "__main__":
    main()
