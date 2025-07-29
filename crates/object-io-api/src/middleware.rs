//! HTTP middleware for the API

use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue, Method},
    middleware::Next,
    response::Response,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::limit::RequestBodyLimitLayer;
use std::time::Duration;

/// Create CORS middleware for S3 API compatibility
pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::HEAD,
            Method::OPTIONS,
        ])
        .allow_headers([
            HeaderName::from_static("authorization"),
            HeaderName::from_static("content-type"),
            HeaderName::from_static("content-length"),
            HeaderName::from_static("x-amz-content-sha256"),
            HeaderName::from_static("x-amz-date"),
            HeaderName::from_static("x-amz-security-token"),
            HeaderName::from_static("x-amz-user-agent"),
            HeaderName::from_static("x-amz-target"),
            HeaderName::from_static("x-amz-acl"),
            HeaderName::from_static("x-amz-version-id"),
            HeaderName::from_static("x-amz-copy-source"),
            HeaderName::from_static("x-amz-copy-source-range"),
            HeaderName::from_static("x-amz-metadata-directive"),
            HeaderName::from_static("x-amz-tagging-directive"),
            HeaderName::from_static("x-amz-server-side-encryption"),
            HeaderName::from_static("x-amz-server-side-encryption-aws-kms-key-id"),
            HeaderName::from_static("x-amz-server-side-encryption-context"),
            HeaderName::from_static("x-amz-request-payer"),
            HeaderName::from_static("x-amz-expected-bucket-owner"),
            HeaderName::from_static("range"),
            HeaderName::from_static("if-match"),
            HeaderName::from_static("if-none-match"),
            HeaderName::from_static("if-modified-since"),
            HeaderName::from_static("if-unmodified-since"),
        ])
        .expose_headers([
            HeaderName::from_static("etag"),
            HeaderName::from_static("x-amz-version-id"),
            HeaderName::from_static("x-amz-server-side-encryption"),
            HeaderName::from_static("x-amz-server-side-encryption-aws-kms-key-id"),
            HeaderName::from_static("x-amz-server-side-encryption-context"),
            HeaderName::from_static("x-amz-request-id"),
            HeaderName::from_static("x-amz-id-2"),
            HeaderName::from_static("content-range"),
            HeaderName::from_static("accept-ranges"),
        ])
}

/// Create timeout middleware (30 second timeout)
pub fn timeout_layer() -> TimeoutLayer {
    TimeoutLayer::new(Duration::from_secs(30))
}

/// Create request body size limit middleware (5GB max for S3 compatibility)
pub fn body_limit_layer() -> RequestBodyLimitLayer {
    RequestBodyLimitLayer::new(5 * 1024 * 1024 * 1024) // 5GB limit
}

/// Add request ID header for tracking
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    let request_id = uuid::Uuid::new_v4().to_string();
    
    // Add request ID to request extensions for handlers to use
    request.extensions_mut().insert(RequestId(request_id.clone()));
    
    let mut response = next.run(request).await;
    
    // Add request ID to response headers
    response.headers_mut().insert(
        "x-amz-request-id",
        HeaderValue::from_str(&request_id).unwrap_or_else(|_| HeaderValue::from_static("unknown")),
    );
    
    response
}

/// Add basic security headers
pub async fn security_headers_middleware(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    
    let headers = response.headers_mut();
    
    // Add security headers
    headers.insert("x-content-type-options", HeaderValue::from_static("nosniff"));
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
    headers.insert("x-xss-protection", HeaderValue::from_static("1; mode=block"));
    headers.insert("referrer-policy", HeaderValue::from_static("strict-origin-when-cross-origin"));
    
    response
}

/// Request ID wrapper for tracking requests
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

impl RequestId {
    pub fn get(&self) -> &str {
        &self.0
    }
}
