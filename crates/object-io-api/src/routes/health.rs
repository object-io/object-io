//! Health check endpoint

use axum::{http::StatusCode, Json};
use serde_json::{json, Value};

/// Health check handler
pub async fn health_check() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "status": "healthy",
            "service": "ObjectIO",
            "version": env!("CARGO_PKG_VERSION"),
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    )
}
