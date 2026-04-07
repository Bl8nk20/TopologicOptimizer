//! Filter-Modul: Sensitivitäts- und Dichtefilter
//!
//! ## Warum Filter?
//!
//! Ohne Filter zeigen Topologieoptimierungen "Schachbrettmuster" (checkerboard patterns)
//! und sind gittermaschenabhängig. Filter glätten die Sensitivitätsfelder und erzwingen
//! eine Mindest-Feature-Größe.
//!
//! ## Implementierte Filter
//!
//! 1. **Sensitivitätsfilter**: Gewichtet Sensitivitäten der Nachbarelemente
//! 2. **Dichtefilter** (PDE-Filter): Glättet die Dichteverteilung

use crate::mesh::Mesh;

/// Berechnet den gewichteten Abstand zwischen zwei Elementen
fn element_center(mesh: &Mesh, elem_id: usize) -> (f64, f64) {
    let col = elem_id % mesh.nelx;
    let row = elem_id / mesh.nelx;
    let cx = (col as f64 + 0.5) * mesh.element_size;
    let cy = (row as f64 + 0.5) * mesh.element_size;
    (cx, cy)
}

/// Sensitivitätsfilter nach Sigmund (1994)
///
/// Dc̃e = (1 / max(γ, ρe) · sum_Hf) · Σ_f H_ef · ρf · Dcf
///
/// wobei H_ef = max(0, r_min - dist(e, f)) (Hut-Funktion)
pub struct SensitivityFilter {
    /// Mindest-Filterradius (in Anzahl Elemente)
    pub radius: f64,
    /// Vorberechnete Nachbarschaftsliste: [(element_id, weight)]
    neighbors: Vec<Vec<(usize, f64)>>,
}

impl SensitivityFilter {
    pub fn new(mesh: &Mesh, radius: f64) -> Self {
        let n_elem = mesh.elements.len();
        let mut neighbors = vec![Vec::new(); n_elem];

        // Für jedes Element: alle Elemente im Filterradius finden
        for e in 0..n_elem {
            let (cx_e, cy_e) = element_center(mesh, e);

            // Nur Elemente im rechteckigen Suchfenster betrachten (effizienter)
            let col_e = e % mesh.nelx;
            let row_e = e / mesh.nelx;
            let r_elem = (radius / mesh.element_size).ceil() as isize;

            let col_min = (col_e as isize - r_elem).max(0) as usize;
            let col_max = (col_e as isize + r_elem).min(mesh.nelx as isize - 1) as usize;
            let row_min = (row_e as isize - r_elem).max(0) as usize;
            let row_max = (row_e as isize + r_elem).min(mesh.nely as isize - 1) as usize;

            for row_f in row_min..=row_max {
                for col_f in col_min..=col_max {
                    let f = row_f * mesh.nelx + col_f;
                    let (cx_f, cy_f) = element_center(mesh, f);
                    let dist = ((cx_e - cx_f).powi(2) + (cy_e - cy_f).powi(2)).sqrt();
                    let weight = (radius - dist).max(0.0);
                    if weight > 0.0 {
                        neighbors[e].push((f, weight));
                    }
                }
            }
        }

        Self { radius, neighbors }
    }

    /// Wendet den Sensitivitätsfilter an
    pub fn apply(&self, densities: &[f64], sensitivities: &[f64]) -> Vec<f64> {
        let n = sensitivities.len();
        let mut filtered = vec![0.0f64; n];

        for e in 0..n {
            let mut sum_h = 0.0;
            let mut sum_h_rho_dc = 0.0;

            for &(f, h_ef) in &self.neighbors[e] {
                sum_h += h_ef;
                sum_h_rho_dc += h_ef * densities[f] * sensitivities[f];
            }

            let rho_e = densities[e].max(1e-3);
            filtered[e] = sum_h_rho_dc / (rho_e * sum_h);
        }

        filtered
    }
}

/// Optimality Criteria (OC) Update
///
/// Berechnet neue Dichten basierend auf dem Bisektionsverfahren,
/// das den Lagrange-Multiplikator λ sucht der die Volumenbeschränkung erfüllt.
pub fn optimality_criteria_update(
    densities: &[f64],
    sensitivities: &[f64],
    volume_fraction: f64,
    move_limit: f64,
    eta: f64, // Dämpfungsexponent, typisch 0.5
) -> Vec<f64> {
    let n = densities.len();

    // Bisection für Lagrange-Multiplikator λ
    let mut lambda_low = 0.0;
    let mut lambda_high = 1e9;
    let mut new_densities = vec![0.0f64; n];

    for _ in 0..50 {
        let lambda_mid = (lambda_low + lambda_high) / 2.0;

        let mut vol = 0.0;
        for i in 0..n {
            // B_e = -Dc_e / lambda
            let b_e = (-sensitivities[i] / lambda_mid).max(0.0);
            // ρ_new = ρ_old · B_e^η (begrenzt durch Move-Limit)
            let rho_new = (densities[i] * b_e.powf(eta))
                .max(densities[i] - move_limit)  // untere Move-Limit
                .min(densities[i] + move_limit)  // obere Move-Limit
                .clamp(0.001, 1.0);              // physikalische Grenzen
            new_densities[i] = rho_new;
            vol += rho_new;
        }

        let actual_vf = vol / n as f64;
        if actual_vf > volume_fraction {
            lambda_low = lambda_mid;
        } else {
            lambda_high = lambda_mid;
        }

        if (lambda_high - lambda_low) / lambda_high < 1e-4 {
            break;
        }
    }

    new_densities
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::Mesh;

    #[test]
    fn test_filter_construction() {
        let mesh = Mesh::regular_grid(10, 5, 1.0).unwrap();
        let filter = SensitivityFilter::new(&mesh, 1.5);
        // Alle Elemente sollten mindestens sich selbst als Nachbar haben
        for nbrs in &filter.neighbors {
            assert!(!nbrs.is_empty());
        }
    }

    #[test]
    fn test_oc_update_volume_constraint() {
        let n = 100;
        let densities = vec![0.5f64; n];
        let sensitivities = vec![-1.0f64; n]; // gleiche Sensitivität überall
        let vf = 0.4;

        let new_d = optimality_criteria_update(&densities, &sensitivities, vf, 0.2, 0.5);

        let actual_vf: f64 = new_d.iter().sum::<f64>() / n as f64;
        assert!(
            (actual_vf - vf).abs() < 0.02,
            "Volumenfraktion {actual_vf:.3} weicht zu stark von {vf} ab"
        );
    }
}