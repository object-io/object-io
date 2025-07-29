//! API Routes for ObjectIO

use axum::{
    middleware,
    routing::get,
    Router,
};
use object_io_core::Result;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::{
    middleware::{
        cors_layer, timeout_layer, body_limit_layer,
        request_id_middleware, security_headers_middleware
    },
    state::AppState,
};

pub mod health;

/// Create the main application router
pub async fn create_app() -> Result<Router> {
    info!("Initializing application state...");
    let state = AppState::new().await?;
    info!("Application state initialized successfully");
    
    info!("Setting up routes and middleware...");
    let app = Router::new()
        // Health check endpoint
        .route("/health", get(health::health_check))
        
        // Root endpoint - S3 service description
        .route("/", get(|| async { "ObjectIO S3-Compatible Storage Server" }))
        
        // Add application state
        .with_state(state)
        
        // Add middleware layers (applied in reverse order)
        .layer(middleware::from_fn(security_headers_middleware))
        .layer(middleware::from_fn(request_id_middleware))
        .layer(cors_layer())
        .layer(timeout_layer())
        .layer(body_limit_layer())
        .layer(TraceLayer::new_for_http());

    info!("Application router configured successfully");
    Ok(app)
}
