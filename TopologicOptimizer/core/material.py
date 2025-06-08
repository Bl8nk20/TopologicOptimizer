from dataclasses import dataclass

@dataclass
class Material:
    name : str
    density : float
    youngs_modulus : float
    poisson_ratio : float