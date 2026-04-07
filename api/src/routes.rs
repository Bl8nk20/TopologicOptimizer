//! HTTP Route Handler

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response, Html},
    Json,
};
use serde_json::json;
use tracing::{error, info};
use uuid::Uuid;

use topo_core::{
    export::{export_csv, export_vtk},
    fem::Material,
    mesh::Mesh,
    simp::Optimizer,
};

use crate::jobs::{Job, JobRequest, JobStore};

pub type AppState = Arc<JobStore>;

pub async fn health() -> Html<&'static str> {
    // Lädt die HTML-Datei aus dem Frontend-Ordner
    Html(include_str!("../../frontend/health.html"))
}

/// POST /api/jobs — neuen Optimierungsjob starten
pub async fn create_job(
    State(store): State<AppState>,
    Json(request): Json<JobRequest>,
) -> Response {
    // Eingaben validieren
    if request.nelx == 0 || request.nely == 0 {
        return (StatusCode::BAD_REQUEST, Json(json!({
            "error": "nelx und nely müssen > 0 sein"
        }))).into_response();
    }
    if request.nelx > 200 || request.nely > 200 {
        return (StatusCode::BAD_REQUEST, Json(json!({
            "error": "Maximale Gittergröße: 200×200"
        }))).into_response();
    }

    let id = store.create_job(request.clone());
    info!("Job {id} erstellt: {}×{} {:?}", request.nelx, request.nely, request.scenario);

    // Worker-Task im Hintergrund starten
    let store_clone = store.clone();
    tokio::spawn(async move {
        run_optimization(store_clone, id, request).await;
    });

    (StatusCode::CREATED, Json(json!({
        "id": id,
        "status": "pending",
        "ws_url": format!("/api/jobs/{id}/ws"),
    }))).into_response()
}

/// GET /api/jobs/:id — Job-Status abfragen
pub async fn get_job(
    State(store): State<AppState>,
    Path(id): Path<Uuid>,
) -> Response {
    match store.get_job(&id) {
        Some(job) => Json(job_to_json(&job)).into_response(),
        None => (StatusCode::NOT_FOUND, Json(json!({ "error": "Job nicht gefunden" }))).into_response(),
    }
}

/// GET /api/jobs/:id/vtk — Ergebnis als VTK herunterladen
pub async fn download_vtk(
    State(store): State<AppState>,
    Path(id): Path<Uuid>,
) -> Response {
    let Some(job) = store.get_job(&id) else {
        return (StatusCode::NOT_FOUND, "Job nicht gefunden").into_response();
    };
    let Some(result) = job.result else {
        return (StatusCode::CONFLICT, "Job noch nicht abgeschlossen").into_response();
    };

    let mesh = build_mesh(&job.request);
    let mut buf = Vec::new();
    if let Err(e) = export_vtk(&mesh, &result, &mut buf, 0.5) {
        error!("VTK Export fehlgeschlagen: {e}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    (
        [(header::CONTENT_TYPE, "application/octet-stream"),
         (header::CONTENT_DISPOSITION, &format!("attachment; filename=\"job_{id}.vtk\"") as &str)],
        buf,
    ).into_response()
}

/// GET /api/jobs/:id/csv — Ergebnis als CSV herunterladen
pub async fn download_csv(
    State(store): State<AppState>,
    Path(id): Path<Uuid>,
) -> Response {
    let Some(job) = store.get_job(&id) else {
        return (StatusCode::NOT_FOUND, "Job nicht gefunden").into_response();
    };
    let Some(result) = job.result else {
        return (StatusCode::CONFLICT, "Job noch nicht abgeschlossen").into_response();
    };

    let mesh = build_mesh(&job.request);
    let mut buf = Vec::new();
    if let Err(e) = export_csv(&mesh, &result, &mut buf) {
        error!("CSV Export fehlgeschlagen: {e}");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    (
        [(header::CONTENT_TYPE, "text/csv"),
         (header::CONTENT_DISPOSITION, &format!("attachment; filename=\"job_{id}.csv\"") as &str)],
        buf,
    ).into_response()
}

// ── Interne Hilfsfunktionen ──────────────────────────────────────────────────

fn build_mesh(req: &JobRequest) -> Mesh {
    Mesh::regular_grid(req.nelx, req.nely, 1.0).expect("Mesh-Erstellung fehlgeschlagen")
}

fn job_to_json(job: &Job) -> serde_json::Value {
    json!({
        "id": job.id,
        "status": job.status,
        "last_iteration": job.last_iteration,
        "converged": job.result.as_ref().map(|r| r.converged),
        "final_compliance": job.result.as_ref().map(|r| r.final_compliance),
        "iterations": job.result.as_ref().map(|r| r.iterations),
    })
}

/// Der eigentliche Optimierungs-Worker (läuft als tokio Task)
async fn run_optimization(store: Arc<JobStore>, id: Uuid, request: JobRequest) {
    store.set_running(&id);
    info!("Job {id}: Optimierung startet");

    // Mesh + Randbedingungen aufbauen
    let mesh = match Mesh::regular_grid(request.nelx, request.nely, 1.0) {
        Ok(m) => m,
        Err(e) => {
            store.set_failed(&id, e.to_string());
            return;
        }
    };

    let bc = request.scenario.build_bc(&mesh);
    let material = Material::default();

    // Optimizer mit Fortschritts-Callback
    let store_iter = store.clone();
    let callback = Box::new(move |iter_result| {
        store_iter.update_iteration(&id, iter_result);
    });

    // Blockierenden Optimizer in eigenem Thread laufen lassen
    // (FEM-Solver ist CPU-intensiv → nicht den tokio-Thread blockieren)
    let result = tokio::task::spawn_blocking(move || {
        let optimizer = Optimizer::new(mesh, material, request.config, bc);
        optimizer.optimize(Some(callback))
    })
    .await;

    match result {
        Ok(Ok(opt_result)) => {
            info!(
                "Job {id}: fertig nach {} Iterationen, C={:.4e}",
                opt_result.iterations, opt_result.final_compliance
            );
            store.set_done(&id, opt_result);
        }
        Ok(Err(e)) => {
            error!("Job {id}: Optimierungsfehler: {e}");
            store.set_failed(&id, e.to_string());
        }
        Err(e) => {
            error!("Job {id}: Task panic: {e}");
            store.set_failed(&id, format!("Internal error: {e}"));
        }
    }
}