//! Export-Modul: Ergebnisse als VTK, CSV oder JSON ausgeben
//!
//! VTK Legacy Format kann direkt in ParaView oder FreeCAD geöffnet werden.

use std::io::Write;

use crate::{error::Result, mesh::Mesh, simp::OptimizationResult};

/// Exportiert das Ergebnis als VTK Legacy ASCII
///
/// Das .vtk Format kann in ParaView, Paraview und FreeCAD geöffnet werden.
pub fn export_vtk(
    mesh: &Mesh,
    result: &OptimizationResult,
    writer: &mut impl Write,
    threshold: f64,
) -> Result<()> {
    let n_nodes = mesh.nodes.len();
    let n_elem = mesh.elements.len();

    writeln!(writer, "# vtk DataFile Version 3.0")?;
    writeln!(writer, "Topology Optimization Result")?;
    writeln!(writer, "ASCII")?;
    writeln!(writer, "DATASET UNSTRUCTURED_GRID")?;
    writeln!(writer)?;

    // Knoten (3D: z=0 für 2D-Problem)
    writeln!(writer, "POINTS {} float", n_nodes)?;
    for node in &mesh.nodes {
        writeln!(writer, "{:.6} {:.6} 0.0", node.x, node.y)?;
    }
    writeln!(writer)?;

    // Zellen (Q4 = VTK Typ 9)
    writeln!(writer, "CELLS {} {}", n_elem, n_elem * 5)?;
    for elem in &mesh.elements {
        writeln!(
            writer,
            "4 {} {} {} {}",
            elem.node_ids[0], elem.node_ids[1], elem.node_ids[2], elem.node_ids[3]
        )?;
    }
    writeln!(writer)?;

    // Zelltypen
    writeln!(writer, "CELL_TYPES {}", n_elem)?;
    for _ in &mesh.elements {
        writeln!(writer, "9")?; // VTK_QUAD
    }
    writeln!(writer)?;

    // Zelldaten: Dichten
    writeln!(writer, "CELL_DATA {}", n_elem)?;
    writeln!(writer, "SCALARS density float 1")?;
    writeln!(writer, "LOOKUP_TABLE default")?;
    for &rho in &result.densities {
        writeln!(writer, "{:.6}", rho)?;
    }

    // Binäres Ergebnis (0/1 basierend auf Schwellwert)
    writeln!(writer, "SCALARS solid int 1")?;
    writeln!(writer, "LOOKUP_TABLE default")?;
    for &rho in &result.densities {
        writeln!(writer, "{}", if rho >= threshold { 1 } else { 0 })?;
    }

    Ok(())
}

/// Exportiert Dichten als CSV (für einfache Analyse)
pub fn export_csv(
    mesh: &Mesh,
    result: &OptimizationResult,
    writer: &mut impl Write,
) -> Result<()> {
    writeln!(writer, "elem_id,col,row,center_x,center_y,density,solid")?;
    for elem in &mesh.elements {
        let col = elem.id % mesh.nelx;
        let row = elem.id / mesh.nelx;
        let cx = (col as f64 + 0.5) * mesh.element_size;
        let cy = (row as f64 + 0.5) * mesh.element_size;
        let rho = result.densities[elem.id];
        writeln!(
            writer,
            "{},{},{},{:.4},{:.4},{:.6},{}",
            elem.id, col, row, cx, cy, rho,
            if rho >= 0.5 { 1 } else { 0 }
        )?;
    }
    Ok(())
}

/// Exportiert eine einfache ASCII-Visualisierung (für Debug/Terminal)
pub fn export_ascii(
    mesh: &Mesh,
    result: &OptimizationResult,
    threshold: f64,
) -> String {
    let mut output = String::new();
    // Von oben nach unten (Zeile nely-1 bis 0)
    for row in (0..mesh.nely).rev() {
        for col in 0..mesh.nelx {
            let elem_id = row * mesh.nelx + col;
            let rho = result.densities[elem_id];
            let ch = if rho >= threshold { '█' } else if rho >= threshold * 0.5 { '▒' } else { '░' };
            output.push(ch);
        }
        output.push('\n');
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        fem::{BoundaryConditions, Material},
        mesh::Mesh,
        simp::{OptimizationConfig, OptimizationResult, Optimizer},
    };

    fn run_small_optimization() -> (Mesh, OptimizationResult) {
        let mesh = Mesh::regular_grid(4, 2, 1.0).unwrap();
        let mut bc = BoundaryConditions::new();
        for n in mesh.left_edge_nodes() {
            bc.fix_node(n);
        }
        let top_right = mesh.node_id(mesh.nelx, mesh.nely);
        bc.apply_force_y(top_right, -1.0);

        let config = OptimizationConfig {
            max_iterations: 5,
            ..Default::default()
        };
        let optimizer = Optimizer::new(mesh.clone(), Material::default(), config, bc);
        let result = optimizer.optimize(None).unwrap();
        (mesh, result)
    }

    #[test]
    fn test_vtk_export() {
        let (mesh, result) = run_small_optimization();
        let mut buf = Vec::new();
        export_vtk(&mesh, &result, &mut buf, 0.5).unwrap();
        let content = String::from_utf8(buf).unwrap();
        assert!(content.contains("vtk DataFile"));
        assert!(content.contains("density"));
    }

    #[test]
    fn test_csv_export() {
        let (mesh, result) = run_small_optimization();
        let mut buf = Vec::new();
        export_csv(&mesh, &result, &mut buf).unwrap();
        let content = String::from_utf8(buf).unwrap();
        assert!(content.contains("elem_id,col,row"));
    }

    #[test]
    fn test_ascii_export() {
        let (mesh, result) = run_small_optimization();
        let ascii = export_ascii(&mesh, &result, 0.5);
        let lines: Vec<&str> = ascii.lines().collect();
        assert_eq!(lines.len(), mesh.nely);
        assert_eq!(lines[0].chars().count(), mesh.nelx);
    }
}