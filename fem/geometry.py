import FreeCAD
import Fem


class GeometryManager:
    def __init__(self, doc):
        self.doc = doc
        self.analysis = None
        self.mesh = None
        self.geometry = None

    def create_analysis(self):
        self.analysis = self.doc.addObject("Fem::FemAnalysis", "Analysis")
        return self.analysis

    def add_geometry(self, geometry):
        self.geometry = geometry
        return geometry

    def create_mesh(self):
        mesh = self.doc.addObject('Fem::FemMeshShapeNetgenObject', 'FEMMesh')
        mesh.Shape = self.geometry
        self.mesh = mesh
        self.analysis.addObject(mesh)
        self.doc.recompute()
        return mesh

    def add_material(self, young=210000, poisson=0.3):
        material = self.doc.addObject("Fem::Material", "Material")
        material.Material = {
            'Name': 'Steel-Generic',
            'YoungsModulus': f'{young} MPa',
            'PoissonRatio': f'{poisson}'
        }
        self.analysis.addObject(material)
        return material
