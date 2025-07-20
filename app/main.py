import Mesh, Part, MeshPart
import FreeCAD as App


cyl = App.ActiveDocument.addObject("Part::Cylinder","Cylinder")
App.ActiveDocument.recompute()

msh = App.ActiveDocument.addObject("Mesh::Feature", "Mesh")
msh.Mesh = MeshPart.meshFromShape(Shape=cyl.Shape, MaxLength=1)
msh.ViewObject.DisplayMode = "Flat Lines"


new_mesh = msh.Mesh.copy()

print(new_mesh)