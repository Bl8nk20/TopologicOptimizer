//! Mesh-Modul: Gitter-Diskretisierung für 2D-Topologieoptimierung
//!
//! Wir arbeiten mit Q4-Elementen (bilineare Viereck-Elemente).
//! Jeder Knoten hat 2 Freiheitsgrade (DOF): u_x und u_y.

use serde::{Deserialize, Serialize};

use crate::error::{Result, TopoError};

/// Ein 2D-Knoten im FEM-Gitter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: usize,
    pub x: f64,
    pub y: f64,
}

impl Node {
    pub fn new(id: usize, x: f64, y: f64) -> Self {
        Self { id, x, y }
    }
}

/// Ein Q4-Element (4 Knoten, bilinear)
///
/// Knotenreihenfolge (counter-clockwise):
/// ```text
///  3 --- 2
///  |     |
///  0 --- 1
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Element {
    pub id: usize,
    /// Globale Knoten-IDs (4 Knoten)
    pub node_ids: [usize; 4],
    /// Aktuelle Dichte (0 = leer, 1 = voll)
    pub density: f64,
}

impl Element {
    pub fn new(id: usize, node_ids: [usize; 4]) -> Self {
        Self {
            id,
            node_ids,
            density: 1.0, // Anfangsdichte: voll
        }
    }

    /// Freiheitsgrade dieses Elements (8 DOFs: 2 pro Knoten × 4 Knoten)
    pub fn dofs(&self) -> [usize; 8] {
        let n = self.node_ids;
        [
            2 * n[0],
            2 * n[0] + 1,
            2 * n[1],
            2 * n[1] + 1,
            2 * n[2],
            2 * n[2] + 1,
            2 * n[3],
            2 * n[3] + 1,
        ]
    }
}

/// Das gesamte FEM-Gitter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    pub nodes: Vec<Node>,
    pub elements: Vec<Element>,
    /// Anzahl Elemente in x-Richtung
    pub nelx: usize,
    /// Anzahl Elemente in y-Richtung
    pub nely: usize,
    /// Elementgröße (quadratisch)
    pub element_size: f64,
}

impl Mesh {
    /// Erstellt ein regelmäßiges rechteckiges Gitter
    ///
    /// # Argumente
    /// * `nelx` - Anzahl Elemente horizontal
    /// * `nely` - Anzahl Elemente vertikal
    /// * `element_size` - Seitenlänge eines Elements
    pub fn regular_grid(nelx: usize, nely: usize, element_size: f64) -> Result<Self> {
        if nelx == 0 || nely == 0 {
            return Err(TopoError::Mesh(
                "Gitter muss mindestens 1×1 Element haben".into(),
            ));
        }
        if element_size <= 0.0 {
            return Err(TopoError::Mesh("Elementgröße muss positiv sein".into()));
        }

        let node_cols = nelx + 1;
        let node_rows = nely + 1;

        // Knoten erzeugen (Zeilen-Major)
        let mut nodes = Vec::with_capacity(node_cols * node_rows);
        for row in 0..node_rows {
            for col in 0..node_cols {
                nodes.push(Node::new(
                    row * node_cols + col,
                    col as f64 * element_size,
                    row as f64 * element_size,
                ));
            }
        }

        // Q4-Elemente erzeugen
        let mut elements = Vec::with_capacity(nelx * nely);
        for row in 0..nely {
            for col in 0..nelx {
                let id = row * nelx + col;
                // Knotenindizes für dieses Element
                let n0 = row * node_cols + col;
                let n1 = row * node_cols + col + 1;
                let n2 = (row + 1) * node_cols + col + 1;
                let n3 = (row + 1) * node_cols + col;
                elements.push(Element::new(id, [n0, n1, n2, n3]));
            }
        }

        Ok(Self {
            nodes,
            elements,
            nelx,
            nely,
            element_size,
        })
    }

    /// Gesamtzahl der Freiheitsgrade
    pub fn ndof(&self) -> usize {
        2 * self.nodes.len()
    }

    /// Knoten an Position (col, row) -> globale Node-ID
    pub fn node_id(&self, col: usize, row: usize) -> usize {
        row * (self.nelx + 1) + col
    }

    /// Element an Position (col, row) -> globale Element-ID
    pub fn element_id(&self, col: usize, row: usize) -> usize {
        row * self.nelx + col
    }

    /// Alle Knoten auf der linken Kante (x = 0)
    pub fn left_edge_nodes(&self) -> Vec<usize> {
        (0..=self.nely).map(|row| self.node_id(0, row)).collect()
    }

    /// Alle Knoten auf der rechten Kante
    pub fn right_edge_nodes(&self) -> Vec<usize> {
        (0..=self.nely)
            .map(|row| self.node_id(self.nelx, row))
            .collect()
    }

    /// Alle Knoten auf der unteren Kante (y = 0)
    pub fn bottom_edge_nodes(&self) -> Vec<usize> {
        (0..=self.nelx).map(|col| self.node_id(col, 0)).collect()
    }

    /// Physikalische Abmessungen des Gitters
    pub fn width(&self) -> f64 {
        self.nelx as f64 * self.element_size
    }

    pub fn height(&self) -> f64 {
        self.nely as f64 * self.element_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regular_grid_node_count() {
        let mesh = Mesh::regular_grid(4, 3, 1.0).unwrap();
        assert_eq!(mesh.nodes.len(), 5 * 4); // (nelx+1)*(nely+1)
        assert_eq!(mesh.elements.len(), 4 * 3);
    }

    #[test]
    fn test_element_dofs() {
        let mesh = Mesh::regular_grid(2, 2, 1.0).unwrap();
        let elem = &mesh.elements[0];
        // Element 0 hat Knoten 0,1,4,3 (bei 3×3 Knoten-Grid)
        let dofs = elem.dofs();
        assert_eq!(dofs.len(), 8);
    }

    #[test]
    fn test_invalid_mesh() {
        assert!(Mesh::regular_grid(0, 3, 1.0).is_err());
        assert!(Mesh::regular_grid(3, 0, 1.0).is_err());
        assert!(Mesh::regular_grid(3, 3, -1.0).is_err());
    }
}