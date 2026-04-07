use thiserror::Error;

#[derive(Debug, Error)]
pub enum TopoError {
    #[error("Mesh-Fehler: {0}")]
    Mesh(String),

    #[error("FEM-Solver-Fehler: {0}")]
    Solver(String),

    #[error("Optimierung nicht konvergiert nach {iterations} Iterationen")]
    NotConverged { iterations: usize },

    #[error("Ungültige Randbedingung: {0}")]
    InvalidBoundaryCondition(String),

    #[error("Singular matrix: Freiheitsgrad {dof} ist nicht fixiert")]
    SingularMatrix { dof: usize },

    #[error("IO-Fehler: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialisierungsfehler: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, TopoError>;