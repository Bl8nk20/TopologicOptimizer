# Gets the Mesh from the FreeCAD - Python Endpoint.
# ? Stores the mesh in a graph ?

# Mesh needs following Properties fullfilled:
# * Geometry itself
# * Material definition
# * Forces on Geometry applied

# Design Patterns: Singleton
from design_patterns import SingletonMeta


class MeshReader(metaclass=SingletonMeta):
    def __init__():
        pass
