//! API Routes for ObjectIO

use axum::{
    routing::{get, delete, put},
    Router,
};
use object_io_core::Result;
use tower_http::trace::TraceLayer;

pub mod health;

/// Create the main application router
pub async fn create_app() -> Result<Router> {
    let app = Router::new()
        // Health check endpoint
        .route("/health", get(health::health_check))
        
        // S3 API routes (to be implemented)
        .route("/", get(|| async { "ObjectIO S3-Compatible Storage" }))
        
        // Add middleware
        .layer(TraceLayer::new_for_http());

    Ok(app)
}
