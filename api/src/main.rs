mod jobs;
mod routes;
mod ws;

use std::{net::SocketAddr, sync::Arc};

use axum::{Router, routing::{get, post}};
use tower_http::{cors::{Any, CorsLayer}, services::ServeDir};
use tracing::info;
use tracing_subscriber::EnvFilter;

use jobs::JobStore;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive("api=debug".parse().unwrap())
            .add_directive("topo_core=info".parse().unwrap()))
        .init();

    let store = Arc::new(JobStore::new());

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/jobs",         post(routes::create_job))
        .route("/api/jobs/{id}",     get(routes::get_job))
        .route("/api/jobs/{id}/vtk", get(routes::download_vtk))
        .route("/api/jobs/{id}/csv", get(routes::download_csv))
        .route("/api/jobs/{id}/ws",  get(ws::job_ws_handler))
        .route("/health", get(routes::health))
        .fallback_service(ServeDir::new("frontend"))
        .layer(cors)
        .with_state(store);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Server läuft auf http://{addr}");
    info!("Frontend: http://localhost:3000/");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}