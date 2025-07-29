//! API Routes for ObjectIO

use axum::{
    middleware,
    routing::{delete, get, head, put},
    Router,
};
use object_io_core::Result;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::{
    handlers::{bucket, object},
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
    
    // Ensure admin user exists
    // TODO: Re-enable after fixing authentication system
    // crate::auth::ensure_admin_user(&state.metadata).await?;
    
    info!("Application state initialized successfully");
    
    info!("Setting up routes and middleware...");
    let app = Router::new()
        // Health check endpoint
        .route("/health", get(health::health_check))
        
        // S3 API routes
        // Root endpoint - List buckets
        .route("/", get(bucket::list_buckets))
        
        // Bucket operations
        .route("/:bucket", put(bucket::create_bucket))
        .route("/:bucket", delete(bucket::delete_bucket))
        .route("/:bucket", head(bucket::head_bucket))
        .route("/:bucket", get(bucket::get_bucket_location))
        
        // Object operations
        .route("/:bucket/:key", put(object::put_object))
        .route("/:bucket/:key", get(object::get_object))
        .route("/:bucket/:key", delete(object::delete_object))
        .route("/:bucket/:key", head(object::head_object))
        
        // Add application state
        .with_state(state.clone())
        
        // Add middleware layers (applied in reverse order)
        // TODO: Re-enable authentication middleware after fixing trait bounds
        // .layer(middleware::from_fn_with_state(state.clone(), crate::auth::auth_middleware))
        .layer(middleware::from_fn(security_headers_middleware))
        .layer(middleware::from_fn(request_id_middleware))
        .layer(cors_layer())
        .layer(timeout_layer())
        .layer(body_limit_layer())
        .layer(TraceLayer::new_for_http());

    info!("Application router configured successfully");
    Ok(app)
}
