from dataclasses import dataclass
import numpy as np
from typing import Dict, List

@dataclass
class Geometry:
    nodes: np.ndarray
    elements : np.ndarray
    boundaries : Dict[str, List[int]]