//! SIMP-Optimizer: Hauptschleife der Topologieoptimierung
//!
//! ## SIMP-Methode (Solid Isotropic Material with Penalization)
//!
//! Grundidee: Jedes Element hat eine Dichte ρe ∈ [0, 1].
//! Das Elastizitätsmodul wird mit einer Penalty-Funktion interpoliert:
//!
//!   E(ρe) = E_min + ρe^p · (E₀ - E_min)
//!
//! Das Optimierungsproblem:
//!
//!   minimize:   C(u, ρ) = f·u  (Compliance = Flexibilität)
//!   subject to: K(ρ)·u = f     (Gleichgewicht)
//!               V(ρ)/V₀ = vf   (Volumenbeschränkung)
//!               0 < ρ_min ≤ ρe ≤ 1
//!
//! Referenz: Sigmund & Maute (2013), Topology optimization approaches

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::{
    error::Result,
    fem::{BoundaryConditions, FemSolver, Material},
    filter::{optimality_criteria_update, SensitivityFilter},
    mesh::Mesh,
};

/// Konfiguration für den Optimizer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    /// Zielmaterialanteil (0 < vf ≤ 1), z.B. 0.4 = 40% des Volumens wird Material
    pub volume_fraction: f64,

    /// SIMP Strafexponent (typisch 3.0)
    pub penalty: f64,

    /// Filterradius in Elementgrößen (verhindert Schachbrettmuster)
    pub filter_radius: f64,

    /// Maximale Iterationen
    pub max_iterations: usize,

    /// Konvergenztoleranz (Änderung der Dichte zwischen Iterationen)
    pub convergence_tolerance: f64,

    /// Move-Limit für OC-Update (typisch 0.2)
    pub move_limit: f64,

    /// OC Dämpfungsexponent (typisch 0.5)
    pub oc_eta: f64,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            volume_fraction: 0.4,
            penalty: 3.0,
            filter_radius: 1.5,
            max_iterations: 100,
            convergence_tolerance: 0.01,
            move_limit: 0.2,
            oc_eta: 0.5,
        }
    }
}

/// Ergebnis einer abgeschlossenen Optimierungsiteration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationResult {
    pub iteration: usize,
    pub compliance: f64,
    pub volume_fraction: f64,
    pub density_change: f64,
    pub densities: Vec<f64>,
}

/// Finales Optimierungsergebnis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    /// Enddichten (0 = leer, 1 = voll)
    pub densities: Vec<f64>,
    /// Compliance-Verlauf über Iterationen
    pub compliance_history: Vec<f64>,
    /// Anzahl durchgeführter Iterationen
    pub iterations: usize,
    /// Ob die Optimierung konvergiert ist
    pub converged: bool,
    /// Finale Compliance
    pub final_compliance: f64,
    /// Gitter-Metadaten
    pub mesh_nelx: usize,
    pub mesh_nely: usize,
}

/// Callback-Typ für Fortschrittsberichte (z.B. für WebSocket-Streaming)
pub type ProgressCallback = Box<dyn Fn(IterationResult) + Send>;

/// Der Hauptoptimizer
pub struct Optimizer {
    mesh: Mesh,
    material: Material,
    config: OptimizationConfig,
    bc: BoundaryConditions,
}

impl Optimizer {
    pub fn new(
        mesh: Mesh,
        material: Material,
        config: OptimizationConfig,
        bc: BoundaryConditions,
    ) -> Self {
        Self {
            mesh,
            material,
            config,
            bc,
        }
    }

    /// Startet die Optimierung mit optionalem Fortschritts-Callback
    pub fn optimize(
        &self,
        callback: Option<ProgressCallback>,
    ) -> Result<OptimizationResult> {
        let n_elem = self.mesh.elements.len();
        let config = &self.config;

        // Sensitivitätsfilter aufbauen
        let filter = SensitivityFilter::new(
            &self.mesh,
            config.filter_radius * self.mesh.element_size,
        );

        let solver = FemSolver::new(self.material.clone());

        // Anfangsdichte: gleichmäßig mit Ziel-Volumenfraktion
        let mut densities = vec![config.volume_fraction; n_elem];
        let mut compliance_history = Vec::new();

        // Kraftvektor für spätere Compliance-Berechnung
        let ndof = self.mesh.ndof();
        let mut force_vector = vec![0.0f64; ndof];
        for &(dof, force) in &self.bc.forces {
            force_vector[dof] += force;
        }

        info!(
            "Starte SIMP-Optimierung: {}×{} Gitter, VF={}, p={}",
            self.mesh.nelx, self.mesh.nely, config.volume_fraction, config.penalty
        );

        let mut converged = false;
        let mut iter = 0;

        for iteration in 0..config.max_iterations {
            iter = iteration + 1;
            let old_densities = densities.clone();

            // ── 1. FEM lösen ────────────────────────────────────────────
            let displacements = solver.solve(
                &self.mesh,
                &densities,
                &self.bc,
                config.penalty,
            )?;

            // ── 2. Compliance und Sensitivitäten berechnen ──────────────
            let compliance = solver.compute_compliance(&displacements, &force_vector);
            let raw_sensitivities = solver.compute_sensitivities(
                &self.mesh,
                &densities,
                &displacements,
                config.penalty,
            );

            // ── 3. Sensitivitätsfilter anwenden ─────────────────────────
            let filtered_sensitivities = filter.apply(&densities, &raw_sensitivities);

            // ── 4. OC-Update: neue Dichten berechnen ────────────────────
            densities = optimality_criteria_update(
                &densities,
                &filtered_sensitivities,
                config.volume_fraction,
                config.move_limit,
                config.oc_eta,
            );

            // ── 5. Konvergenz prüfen ────────────────────────────────────
            let density_change = max_density_change(&old_densities, &densities);
            let actual_vf: f64 = densities.iter().sum::<f64>() / n_elem as f64;

            compliance_history.push(compliance);

            debug!(
                "Iter {:3}: C = {:.4e}, VF = {:.4}, Δρ = {:.4}",
                iter, compliance, actual_vf, density_change
            );

            // Callback für Live-Updates
            if let Some(cb) = &callback {
                cb(IterationResult {
                    iteration: iter,
                    compliance,
                    volume_fraction: actual_vf,
                    density_change,
                    densities: densities.clone(),
                });
            }

            if density_change < config.convergence_tolerance {
                info!(
                    "Konvergenz nach {} Iterationen (Δρ = {:.6})",
                    iter, density_change
                );
                converged = true;
                break;
            }
        }

        if !converged {
            info!(
                "Maximale Iterationen ({}) erreicht ohne Konvergenz",
                config.max_iterations
            );
        }

        let final_compliance = *compliance_history.last().unwrap_or(&f64::INFINITY);

        Ok(OptimizationResult {
            densities,
            compliance_history,
            iterations: iter,
            converged,
            final_compliance,
            mesh_nelx: self.mesh.nelx,
            mesh_nely: self.mesh.nely,
        })
    }

    /// Gibt das Gitter zurück
    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }
}

/// Maximale Dichteänderung zwischen zwei Iterationen
fn max_density_change(old: &[f64], new: &[f64]) -> f64 {
    old.iter()
        .zip(new.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f64, f64::max)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_cantilever() -> (Mesh, BoundaryConditions) {
        // Klassisches Benchmark: Kragarm 4×2 Elemente
        let mesh = Mesh::regular_grid(4, 2, 1.0).unwrap();
        let mut bc = BoundaryConditions::new();

        // Linke Kante einspannen
        for node_id in mesh.left_edge_nodes() {
            bc.fix_node(node_id);
        }

        // Kraft mittig rechts nach unten
        let mid_right = mesh.node_id(mesh.nelx, mesh.nely / 2);
        bc.apply_force_y(mid_right, -1.0);

        (mesh, bc)
    }

    #[test]
    fn test_optimization_runs() {
        let (mesh, bc) = simple_cantilever();
        let config = OptimizationConfig {
            max_iterations: 5, // Nur kurz für Tests
            ..Default::default()
        };
        let optimizer = Optimizer::new(mesh, Material::default(), config, bc);
        let result = optimizer.optimize(None).unwrap();

        assert_eq!(result.mesh_nelx, 4);
        assert_eq!(result.mesh_nely, 2);
        assert!(!result.densities.is_empty());
        assert!(result.final_compliance > 0.0);
    }

    #[test]
    fn test_volume_constraint_respected() {
        let (mesh, bc) = simple_cantilever();
        let vf = 0.5;
        let config = OptimizationConfig {
            volume_fraction: vf,
            max_iterations: 20,
            ..Default::default()
        };
        let optimizer = Optimizer::new(mesh, Material::default(), config, bc);
        let result = optimizer.optimize(None).unwrap();

        let actual_vf: f64 =
            result.densities.iter().sum::<f64>() / result.densities.len() as f64;
        // Volumenfraktion sollte nah am Ziel sein (±5%)
        assert!(
            (actual_vf - vf).abs() < 0.05,
            "VF {actual_vf:.3} weicht von Ziel {vf} ab"
        );
    }
}