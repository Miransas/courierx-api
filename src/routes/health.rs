use axum::Json;
use serde_json::{Value, json};

/// `GET /health` — liveness probe. No auth required.
pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
