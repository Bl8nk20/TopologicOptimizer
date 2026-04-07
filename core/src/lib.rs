//! # topo-core
//!
//! Topologie-Optimierungsengine von Grund auf in Rust.
//!
//! ## Architektur
//!
//! ```text
//! mesh      → Geometrie einlesen, diskretisieren
//! fem       → Finite-Elemente-Methode: Steifigkeitsmatrix + Solver
//! simp      → SIMP-Algorithmus: iterative Dichteoptimierung
//! filter    → Sensitivitäts- und Dichtefilter
//! export    → Ergebnis als STL / VTK ausgeben
//! ```

pub mod error;
pub mod export;
pub mod fem;
pub mod filter;
pub mod mesh;
pub mod simp;

// Häufig benötigte Typen direkt re-exportieren
pub use error::TopoError;
pub use mesh::{Element, Mesh, Node};
pub use simp::{OptimizationConfig, OptimizationResult, Optimizer};
