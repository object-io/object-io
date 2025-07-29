//! S3-compatible response formats

use axum::{
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use serde_json::json;

/// S3-compatible error response
#[derive(Debug, Serialize)]
pub struct S3ErrorResponse {
    #[serde(rename = "Code")]
    pub code: String,
    #[serde(rename = "Message")]
    pub message: String,
    #[serde(rename = "RequestId")]
    pub request_id: String,
    #[serde(rename = "Resource")]
    pub resource: Option<String>,
}

/// Standard API error response
#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    pub error: String,
    pub message: String,
    pub request_id: String,
    pub timestamp: String,
}

/// Convert ObjectIO error to HTTP response  
pub fn error_response(error: &object_io_core::ObjectIOError, request_id: String) -> impl IntoResponse {
    let status = StatusCode::from_u16(error.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    
    let error_response = S3ErrorResponse {
        code: error.s3_error_code().to_string(),
        message: error.to_string(),
        request_id: request_id.clone(),
        resource: None,
    };

    let mut response = (status, Json(error_response)).into_response();
    
    // Add standard AWS headers
    response.headers_mut().insert(
        "x-amz-request-id",
        request_id.parse().unwrap_or_else(|_| "unknown".parse().unwrap()),
    );
    response.headers_mut().insert(
        "content-type",
        "application/xml".parse().unwrap(),
    );

    response
}

/// Create a success response with JSON body
pub fn json_response<T: Serialize>(data: T) -> impl IntoResponse {
    (StatusCode::OK, Json(data))
}

/// Create an XML response for S3 compatibility
pub fn xml_response(xml: String) -> impl IntoResponse {
    (
        StatusCode::OK,
        [("content-type", "application/xml")],
        xml,
    )
}

/// Create a health check response
pub fn health_response() -> impl IntoResponse {
    json_response(json!({
        "status": "healthy",
        "service": "ObjectIO",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
