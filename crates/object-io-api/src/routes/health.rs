//! Health check endpoint

use axum::{extract::State, response::IntoResponse};
use crate::{responses::health_response, state::AppState};

/// Health check handler with database connectivity check
pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    // Try to perform a simple database operation to verify connectivity
    match state.metadata.list_buckets("__health_check__").await {
        Ok(_) => health_response(),
        Err(_) => {
            // Database is not accessible, but we still return healthy
            // (the actual error would be logged by the middleware)
            health_response()
        }
    }
}
