//! WebSocket Handler: Live-Streaming von Optimierungsiterationen
//!
//! Jeder Client subscribed auf einen watch::Receiver.
//! Pro Iteration sendet der Worker eine JSON-Nachricht.

use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::Response,
};
use tracing::{debug, info};
use uuid::Uuid;

use crate::jobs::JobStore;

/// GET /api/jobs/:id/ws — WebSocket Upgrade
pub async fn job_ws_handler(
    ws: WebSocketUpgrade,
    State(store): State<Arc<JobStore>>,
    Path(id): Path<Uuid>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, store, id))
}

async fn handle_socket(mut socket: WebSocket, store: Arc<JobStore>, id: Uuid) {
    info!("WS verbunden für Job {id}");

    // Prüfen ob Job existiert
    if store.get_job(&id).is_none() {
        let _ = socket
            .send(Message::Text(r#"{"error":"Job nicht gefunden"}"#.into()))
            .await;
        return;
    }

    // Watch-Receiver subscriben
    let Some(mut rx) = store.subscribe(&id) else {
        let _ = socket
            .send(Message::Text(r#"{"error":"Kein Stream verfügbar"}"#.into()))
            .await;
        return;
    };

    // Ersten Status senden
    let initial_status = serde_json::json!({
        "type": "connected",
        "job_id": id,
    });
    let _ = socket
        .send(Message::Text(initial_status.to_string().into()))
        .await;

    loop {
        tokio::select! {
            // Neue Iteration vom Worker
            changed = rx.changed() => {
                if changed.is_err() {
                    // Sender wurde gedroppt → Job beendet
                    break;
                }

                let iter_data = rx.borrow_and_update().clone();
                if let Some(iter) = iter_data {
                    let msg = serde_json::json!({
                        "type": "iteration",
                        "iteration": iter.iteration,
                        "compliance": iter.compliance,
                        "volume_fraction": iter.volume_fraction,
                        "density_change": iter.density_change,
                        // Dichten als flaches Array (nelx*nely Werte)
                        "densities": iter.densities,
                    });

                    debug!("WS → Iter {}", iter.iteration);

                    if socket.send(Message::Text(msg.to_string().into())).await.is_err() {
                        break; // Client getrennt
                    }
                }

                // Prüfen ob Job fertig ist
                if let Some(job) = store.get_job(&id) {
                    use crate::jobs::JobStatus;
                    match &job.status {
                        JobStatus::Done => {
                            let done_msg = serde_json::json!({
                                "type": "done",
                                "converged": job.result.as_ref().map(|r| r.converged),
                                "final_compliance": job.result.as_ref().map(|r| r.final_compliance),
                                "iterations": job.result.as_ref().map(|r| r.iterations),
                            });
                            let _ = socket.send(Message::Text(done_msg.to_string().into())).await;
                            break;
                        }
                        JobStatus::Failed(err) => {
                            let fail_msg = serde_json::json!({
                                "type": "failed",
                                "error": err,
                            });
                            let _ = socket.send(Message::Text(fail_msg.to_string().into())).await;
                            break;
                        }
                        _ => {}
                    }
                }
            }

            // Client sendet Ping oder schließt Verbindung
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(data))) => {
                        let _ = socket.send(Message::Pong(data)).await;
                    }
                    _ => {}
                }
            }
        }
    }

    info!("WS getrennt für Job {id}");
}