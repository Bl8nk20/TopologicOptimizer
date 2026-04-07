//! FEM-Modul: Finite-Elemente-Methode für 2D linear-elastische Probleme
//!
//! ## Theorie
//!
//! Für ebene Spannungsprobleme (plane stress) lösen wir:
//!   K · u = f
//!
//! wobei:
//!   K = globale Steifigkeitsmatrix
//!   u = Verschiebungsvektor (DOFs)
//!   f = Kraftvektor
//!
//! Das Element-Steifigkeitsmatrix ergibt sich aus:
//!   Ke = ∫ Bᵀ · C · B dV
//!
//! mit B = Verzerrungs-Verschiebungs-Matrix (strain-displacement matrix)
//! und C = Elastizitätstensor (ebener Spannungszustand)

use serde::{Deserialize, Serialize};

use crate::{
    error::{Result, TopoError},
    mesh::Mesh,
};

/// Materialparameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    /// Elastizitätsmodul (Young's modulus) in Pa
    pub young_modulus: f64,
    /// Querkontraktionszahl (Poisson's ratio), typisch 0.3
    pub poisson_ratio: f64,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            young_modulus: 1.0,    // normiert
            poisson_ratio: 0.3,
        }
    }
}

impl Material {
    /// Elastizitätstensor C für ebenen Spannungszustand (3×3 Voigt-Notation)
    /// C = E/(1-ν²) · [[1, ν, 0], [ν, 1, 0], [0, 0, (1-ν)/2]]
    pub fn elasticity_tensor(&self) -> [[f64; 3]; 3] {
        let e = self.young_modulus;
        let v = self.poisson_ratio;
        let factor = e / (1.0 - v * v);
        [
            [factor * 1.0,       factor * v,       0.0],
            [factor * v,         factor * 1.0,     0.0],
            [0.0,                0.0,               factor * (1.0 - v) / 2.0],
        ]
    }
}

/// Gauss-Quadratur Punkte und Gewichte für Q4 (2×2 Integration)
const GAUSS_POINTS: [(f64, f64, f64); 4] = [
    (-0.577_350_269, -0.577_350_269, 1.0),
    ( 0.577_350_269, -0.577_350_269, 1.0),
    ( 0.577_350_269,  0.577_350_269, 1.0),
    (-0.577_350_269,  0.577_350_269, 1.0),
];

/// Berechnet die Element-Steifigkeitsmatrix für ein Q4-Element
///
/// Gibt eine 8×8 Matrix zurück (2 DOF × 4 Knoten)
pub fn element_stiffness_matrix(material: &Material, element_size: f64) -> [[f64; 8]; 8] {
    let c = material.elasticity_tensor();
    let a = element_size / 2.0; // halbe Elementgröße

    let mut ke = [[0.0f64; 8]; 8];

    for (xi, eta, weight) in &GAUSS_POINTS {
        // Formfunktions-Ableitungen im natürlichen Koordinatensystem
        // N1=(1-xi)(1-eta)/4, N2=(1+xi)(1-eta)/4, etc.
        let dndxi = [
            -(1.0 - eta) / 4.0,
             (1.0 - eta) / 4.0,
             (1.0 + eta) / 4.0,
            -(1.0 + eta) / 4.0,
        ];
        let dndeta = [
            -(1.0 - xi) / 4.0,
            -(1.0 + xi) / 4.0,
             (1.0 + xi) / 4.0,
             (1.0 - xi) / 4.0,
        ];

        // Jacobi-Matrix (für quadratisches Element vereinfacht)
        // J = [[a, 0], [0, a]]  →  det(J) = a²
        let det_j = a * a;

        // Inverse Jacobi: dN/dx = (1/a) * dN/dxi
        let dndx: Vec<f64> = dndxi.iter().map(|d| d / a).collect();
        let dndy: Vec<f64> = dndeta.iter().map(|d| d / a).collect();

        // B-Matrix (3×8): εxx = ∂u/∂x, εyy = ∂v/∂y, γxy = ∂u/∂y + ∂v/∂x
        let mut b = [[0.0f64; 8]; 3];
        for i in 0..4 {
            b[0][2 * i]     = dndx[i]; // εxx
            b[1][2 * i + 1] = dndy[i]; // εyy
            b[2][2 * i]     = dndy[i]; // γxy (u-Teil)
            b[2][2 * i + 1] = dndx[i]; // γxy (v-Teil)
        }

        // Ke += weight * det(J) * Bᵀ · C · B
        // Erst CB = C · B berechnen (3×8)
        let mut cb = [[0.0f64; 8]; 3];
        for row in 0..3 {
            for col in 0..8 {
                for k in 0..3 {
                    cb[row][col] += c[row][k] * b[k][col];
                }
            }
        }

        // Dann Ke += Bᵀ · CB
        for i in 0..8 {
            for j in 0..8 {
                let mut sum = 0.0;
                for k in 0..3 {
                    sum += b[k][i] * cb[k][j];
                }
                ke[i][j] += weight * det_j * sum;
            }
        }
    }

    ke
}

/// Randbedingungen für den FEM-Solver
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryConditions {
    /// Fixierte DOFs (Dirichlet-RB): (dof_index, wert)
    pub fixed_dofs: Vec<(usize, f64)>,
    /// Aufgebrachte Kräfte: (dof_index, kraftwert)
    pub forces: Vec<(usize, f64)>,
}

impl BoundaryConditions {
    pub fn new() -> Self {
        Self {
            fixed_dofs: Vec::new(),
            forces: Vec::new(),
        }
    }

    /// Fixiert alle DOFs eines Knotens (vollständige Einspannung)
    pub fn fix_node(&mut self, node_id: usize) {
        self.fixed_dofs.push((2 * node_id, 0.0));
        self.fixed_dofs.push((2 * node_id + 1, 0.0));
    }

    /// Fixiert nur die x-Verschiebung eines Knotens
    pub fn fix_node_x(&mut self, node_id: usize) {
        self.fixed_dofs.push((2 * node_id, 0.0));
    }

    /// Fixiert nur die y-Verschiebung eines Knotens
    pub fn fix_node_y(&mut self, node_id: usize) {
        self.fixed_dofs.push((2 * node_id + 1, 0.0));
    }

    /// Wendet eine Kraft in x-Richtung an einem Knoten an
    pub fn apply_force_x(&mut self, node_id: usize, force: f64) {
        self.forces.push((2 * node_id, force));
    }

    /// Wendet eine Kraft in y-Richtung an einem Knoten an
    pub fn apply_force_y(&mut self, node_id: usize, force: f64) {
        self.forces.push((2 * node_id + 1, force));
    }
}

impl Default for BoundaryConditions {
    fn default() -> Self {
        Self::new()
    }
}

/// Einfacher direkter Solver für K·u = f
///
/// Implementiert den Penalty-Ansatz für Dirichlet-RBs:
/// Fixierte DOFs werden mit einem großen Wert (Penalty) auf der Diagonale belegt.
pub struct FemSolver {
    pub material: Material,
}

impl FemSolver {
    pub fn new(material: Material) -> Self {
        Self { material }
    }

    /// Assembliert die globale Steifigkeitsmatrix
    ///
    /// Gibt eine ndof×ndof Matrix zurück (dichte Darstellung für kleine Probleme).
    /// TODO: Für große Probleme → Sparse-Matrix (CSR-Format)
    pub fn assemble_global_stiffness(
        &self,
        mesh: &Mesh,
        densities: &[f64],
        penalty: f64,
    ) -> Vec<Vec<f64>> {
        let ndof = mesh.ndof();
        let mut k_global = vec![vec![0.0f64; ndof]; ndof];

        // Element-Steifigkeitsmatrix (gleich für alle Elemente bei uniformem Gitter)
        let ke_base = element_stiffness_matrix(&self.material, mesh.element_size);

        for elem in &mesh.elements {
            // SIMP-Skalierung: E(ρ) = E_min + ρᵖ · (E₀ - E_min)
            let rho = densities[elem.id];
            let e_min = 1e-9 * self.material.young_modulus; // Vermeidet singuläre Matrix
            let e_scaled = e_min + rho.powf(penalty) * (self.material.young_modulus - e_min);
            let scale = e_scaled / self.material.young_modulus;

            let dofs = elem.dofs();

            // Assemblierung: Ke → K_global
            for (i, &dof_i) in dofs.iter().enumerate() {
                for (j, &dof_j) in dofs.iter().enumerate() {
                    k_global[dof_i][dof_j] += scale * ke_base[i][j];
                }
            }
        }

        k_global
    }

    /// Löst K·u = f mit Penalty-Methode für Dirichlet-RBs
    pub fn solve(
        &self,
        mesh: &Mesh,
        densities: &[f64],
        bc: &BoundaryConditions,
        simp_penalty: f64,
    ) -> Result<Vec<f64>> {
        let ndof = mesh.ndof();
        let mut k = self.assemble_global_stiffness(mesh, densities, simp_penalty);
        let mut f = vec![0.0f64; ndof];

        // Kräfte einsetzen
        for &(dof, force) in &bc.forces {
            if dof >= ndof {
                return Err(TopoError::InvalidBoundaryCondition(format!(
                    "Kraft-DOF {dof} liegt außerhalb des Gitters (max: {ndof})"
                )));
            }
            f[dof] += force;
        }

        // Penalty-Methode für fixierte DOFs
        let penalty_value = 1e20 * self.material.young_modulus;
        for &(dof, prescribed) in &bc.fixed_dofs {
            if dof >= ndof {
                return Err(TopoError::InvalidBoundaryCondition(format!(
                    "Fixierter DOF {dof} liegt außerhalb des Gitters (max: {ndof})"
                )));
            }
            k[dof][dof] += penalty_value;
            f[dof] += penalty_value * prescribed;
        }

        // Löse mit Gauss-Elimination (für kleine Systeme)
        // TODO: Für große Systeme → Conjugate Gradient
        gauss_elimination(k, f)
    }

    /// Berechnet die Compliance (= externe Arbeit = uᵀ·f)
    /// Compliance = Maß für die Flexibilität → minimieren = steifes Bauteil
    pub fn compute_compliance(&self, displacements: &[f64], forces: &[f64]) -> f64 {
        displacements
            .iter()
            .zip(forces.iter())
            .map(|(u, f)| u * f)
            .sum()
    }

    /// Berechnet Sensitivitäten: ∂C/∂ρe = -p · ρe^(p-1) · ue · Ke · ue
    pub fn compute_sensitivities(
        &self,
        mesh: &Mesh,
        densities: &[f64],
        displacements: &[f64],
        simp_penalty: f64,
    ) -> Vec<f64> {
        let ke_base = element_stiffness_matrix(&self.material, mesh.element_size);
        let mut sensitivities = vec![0.0f64; mesh.elements.len()];

        for elem in &mesh.elements {
            let rho = densities[elem.id];
            let dofs = elem.dofs();

            // Lokaler Verschiebungsvektor für dieses Element
            let ue: Vec<f64> = dofs.iter().map(|&d| displacements[d]).collect();

            // ue · Ke · ue (Elementenergy)
            let mut uke = [0.0f64; 8];
            for i in 0..8 {
                for j in 0..8 {
                    uke[i] += ke_base[i][j] * ue[j];
                }
            }
            let ue_ke_ue: f64 = ue.iter().zip(uke.iter()).map(|(a, b)| a * b).sum();

            // ∂C/∂ρe = -p · ρe^(p-1) · ueᵀ·Ke·ue
            sensitivities[elem.id] =
                -simp_penalty * rho.powf(simp_penalty - 1.0) * ue_ke_ue;
        }

        sensitivities
    }
}

/// Gauss-Elimination mit Pivoting
fn gauss_elimination(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Result<Vec<f64>> {
    let n = b.len();

    for col in 0..n {
        // Partial Pivoting: finde größten Wert in dieser Spalte
        let mut max_row = col;
        let mut max_val = a[col][col].abs();
        for row in (col + 1)..n {
            if a[row][col].abs() > max_val {
                max_val = a[row][col].abs();
                max_row = row;
            }
        }

        if max_val < 1e-14 {
            return Err(TopoError::SingularMatrix { dof: col });
        }

        // Zeilen tauschen
        if max_row != col {
            a.swap(col, max_row);
            b.swap(col, max_row);
        }

        // Elimination
        for row in (col + 1)..n {
            let factor = a[row][col] / a[col][col];
            for k in col..n {
                let delta = factor * a[col][k];
                a[row][k] -= delta;
            }
            b[row] -= factor * b[col];
        }
    }

    // Rücksubstitution
    let mut x = vec![0.0f64; n];
    for i in (0..n).rev() {
        let mut sum = b[i];
        for j in (i + 1)..n {
            sum -= a[i][j] * x[j];
        }
        x[i] = sum / a[i][i];
    }

    Ok(x)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::Mesh;

    #[test]
    fn test_element_stiffness_symmetry() {
        let mat = Material::default();
        let ke = element_stiffness_matrix(&mat, 1.0);
        // Steifigkeitsmatrix muss symmetrisch sein
        for i in 0..8 {
            for j in 0..8 {
                let diff = (ke[i][j] - ke[j][i]).abs();
                assert!(diff < 1e-10, "Ke nicht symmetrisch bei [{i}][{j}]: {diff}");
            }
        }
    }

    #[test]
    fn test_simple_cantilever() {
        // 2×1 Gitter: 2 Elemente horizontal, 1 vertikal
        let mesh = Mesh::regular_grid(2, 1, 1.0).unwrap();
        let mat = Material::default();
        let solver = FemSolver::new(mat);
        let densities = vec![1.0; mesh.elements.len()];

        let mut bc = BoundaryConditions::new();
        // Linke Kante einspannen
        for &node_id in &mesh.left_edge_nodes() {
            bc.fix_node(node_id);
        }
        // Kraft rechts oben in -y Richtung
        let top_right_node = mesh.node_id(mesh.nelx, mesh.nely);
        bc.apply_force_y(top_right_node, -1.0);

        let u = solver.solve(&mesh, &densities, &bc, 3.0).unwrap();

        // Linke Knoten sollten keine Verschiebung haben
        for &node_id in &mesh.left_edge_nodes() {
            assert!(u[2 * node_id].abs() < 1e-6);
            assert!(u[2 * node_id + 1].abs() < 1e-6);
        }

        // Rechter Knoten sollte sich nach unten bewegen
        assert!(u[2 * top_right_node + 1] < 0.0, "Kein erwartetes Durchbiegen");
    }

    #[test]
    fn test_gauss_elimination() {
        // Einfaches 2×2 System: 2x + y = 5, x + 2y = 4 → x=2, y=1
        let a = vec![vec![2.0, 1.0], vec![1.0, 2.0]];
        let b = vec![5.0, 4.0];
        let x = gauss_elimination(a, b).unwrap();
        assert!((x[0] - 2.0).abs() < 1e-10);
        assert!((x[1] - 1.0).abs() < 1e-10);
    }
}