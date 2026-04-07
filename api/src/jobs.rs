//! Job-Store: Geteilter Zustand für alle laufenden Optimierungsjobs
//!
//! Zugriffsmuster:
//!   - HTTP-Handler lesen/schreiben via Arc<JobStore>
//!   - Worker-Tasks schreiben Status-Updates
//!   - WebSocket-Handler lesen live per tokio::sync::watch

use std::{
    collections::HashMap,
    sync::Mutex,
};

use serde::{Deserialize, Serialize};
use tokio::sync::watch;
use uuid::Uuid;

use topo_core::{
    fem::BoundaryConditions,
    simp::{IterationResult, OptimizationConfig, OptimizationResult},
};

/// Was der Client beim Job-Start sendet
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JobRequest {
    /// Gittergröße horizontal
    pub nelx: usize,
    /// Gittergröße vertikal
    pub nely: usize,
    /// Optimierungsparameter
    pub config: OptimizationConfig,
    /// Vordefiniertes Lastfall-Szenario
    pub scenario: LoadScenario,
}

/// Vordefinierte Lastfälle – einfach auswählbar vom Frontend
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum LoadScenario {
    /// Klassischer Kragarm: links eingespannt, Kraft rechts mittig
    Cantilever,
    /// Brücke: beide Seiten unten gelagert, Last mittig oben
    Bridge,
    /// MBB-Beam: Standardbenchmark für Topologieoptimierung
    MbbBeam,
}

impl LoadScenario {
    /// Erzeugt die Randbedingungen für das gegebene Gitter
    pub fn build_bc(&self, mesh: &topo_core::mesh::Mesh) -> BoundaryConditions {
        let mut bc = BoundaryConditions::new();
        match self {
            LoadScenario::Cantilever => {
                for node_id in mesh.left_edge_nodes() {
                    bc.fix_node(node_id);
                }
                let tip = mesh.node_id(mesh.nelx, mesh.nely / 2);
                bc.apply_force_y(tip, -1.0);
            }
            LoadScenario::Bridge => {
                // Beide unteren Ecken gelagert
                bc.fix_node(mesh.node_id(0, 0));
                bc.fix_node(mesh.node_id(mesh.nelx, 0));
                // Last mittig oben
                let top_mid = mesh.node_id(mesh.nelx / 2, mesh.nely);
                bc.apply_force_y(top_mid, -1.0);
            }
            LoadScenario::MbbBeam => {
                // MBB: linke obere Ecke fixiert (nur y), rechte untere Ecke (nur y)
                bc.fix_node_y(mesh.node_id(0, mesh.nely));
                bc.fix_node_y(mesh.node_id(mesh.nelx, 0));
                // Kraft links oben nach unten
                bc.apply_force_y(mesh.node_id(0, mesh.nely), -1.0);
            }
        }
        bc
    }
}

/// Status eines Jobs
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Running,
    Done,
    Failed(String),
}

/// Ein einzelner Job im Store
#[derive(Debug, Clone, Serialize)]
pub struct Job {
    pub id: Uuid,
    pub status: JobStatus,
    pub request: JobRequest,
    /// Letzter Iterations-Update (für REST polling)
    pub last_iteration: Option<IterationResult>,
    /// Finales Ergebnis (nach Abschluss)
    pub result: Option<OptimizationResult>,
}

/// Geteilter Job-Store (thread-safe via Mutex)
pub struct JobStore {
    jobs: Mutex<HashMap<Uuid, Job>>,
    /// watch-Sender pro Job – WebSocket-Clients subscriben hierauf
    pub watchers: Mutex<HashMap<Uuid, watch::Sender<Option<IterationResult>>>>,
}

impl JobStore {
    pub fn new() -> Self {
        Self {
            jobs: Mutex::new(HashMap::new()),
            watchers: Mutex::new(HashMap::new()),
        }
    }

    /// Legt einen neuen Job an, gibt die ID zurück
    pub fn create_job(&self, request: JobRequest) -> Uuid {
        let id = Uuid::new_v4();
        let (tx, _rx) = watch::channel(None);

        let job = Job {
            id,
            status: JobStatus::Pending,
            request,
            last_iteration: None,
            result: None,
        };

        self.jobs.lock().unwrap().insert(id, job);
        self.watchers.lock().unwrap().insert(id, tx);
        id
    }

    pub fn get_job(&self, id: &Uuid) -> Option<Job> {
        self.jobs.lock().unwrap().get(id).cloned()
    }

    pub fn set_running(&self, id: &Uuid) {
        let mut jobs = self.jobs.lock().unwrap();
        if let Some(job) = jobs.get_mut(id) {
            job.status = JobStatus::Running;
        }
    }

    /// Wird vom Worker bei jeder Iteration aufgerufen
    pub fn update_iteration(&self, id: &Uuid, iter: IterationResult) {
        // Job-State updaten
        {
            let mut jobs = self.jobs.lock().unwrap();
            if let Some(job) = jobs.get_mut(id) {
                job.last_iteration = Some(iter.clone());
            }
        }
        // WebSocket-Clients benachrichtigen
        let watchers = self.watchers.lock().unwrap();
        if let Some(tx) = watchers.get(id) {
            let _ = tx.send(Some(iter));
        }
    }

    pub fn set_done(&self, id: &Uuid, result: OptimizationResult) {
        let mut jobs = self.jobs.lock().unwrap();
        if let Some(job) = jobs.get_mut(id) {
            job.status = JobStatus::Done;
            job.result = Some(result);
        }
    }

    pub fn set_failed(&self, id: &Uuid, error: String) {
        let mut jobs = self.jobs.lock().unwrap();
        if let Some(job) = jobs.get_mut(id) {
            job.status = JobStatus::Failed(error);
        }
    }

    /// Gibt einen watch::Receiver für Live-Updates zurück
    pub fn subscribe(&self, id: &Uuid) -> Option<watch::Receiver<Option<IterationResult>>> {
        let watchers = self.watchers.lock().unwrap();
        watchers.get(id).map(|tx| tx.subscribe())
    }
}

impl Default for JobStore {
    fn default() -> Self {
        Self::new()
    }
}