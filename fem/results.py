import numpy as np


class ResultsManager:
    def __init__(self, mesh, analysis):
        self.mesh = mesh
        self.analysis = analysis

    def get_mesh_data(self):
        mesh_obj = self.mesh.FemMesh
        nodes = mesh_obj.Nodes
        elements = mesh_obj.Volumes  # Tetraeder für 3D

        node_ids = list(nodes.keys())
        node_coords = np.array([nodes[i] for i in node_ids])

        element_ids = list(elements.keys())
        connectivity = np.array([elements[i] for i in element_ids])

        return node_coords, connectivity, node_ids, element_ids

    def get_displacement_field(self):
        result = None
        for obj in self.analysis.Group:
            if obj.isDerivedFrom("Fem::FemResultObject"):
                result = obj
                break
        if result is None:
            raise RuntimeError("No result object found.")

        displacements = {
            node_id: (vector.x, vector.y, vector.z)
            for node_id, vector in result.DisplacementVectors.items()
        }

        return displacements
