//! Authentication and authorization for S3 API

pub mod sigv4;

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use chrono::{DateTime, Utc};
use object_io_core::{ObjectIOError, Result};
use object_io_metadata::MetadataOperations;
use std::sync::Arc;

use crate::state::AppState;
use sigv4::{AuthorizationHeader, SignatureRequest, SigV4Validator};

/// Authentication middleware for S3 API requests
pub async fn auth_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> std::result::Result<Response, StatusCode> {
    // Skip authentication for health checks and CORS preflight
    let path = request.uri().path();
    if path == "/health" || request.method() == "OPTIONS" {
        return Ok(next.run(request).await);
    }

    // Extract authentication information from headers
    let headers = request.headers().clone();
    let auth_result = authenticate_request(&headers, &request, &state.metadata).await;

    match auth_result {
        Ok(auth_context) => {
            // Add auth context to request extensions for use in handlers
            let (mut parts, body) = request.into_parts();
            parts.extensions.insert(auth_context);
            let new_request = Request::from_parts(parts, body);
            Ok(next.run(new_request).await)
        }
        Err(ObjectIOError::AuthError { message }) => {
            eprintln!("Authentication failed: {}", message);
            
            // Return appropriate S3 error response
            let error_response = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<Error>
    <Code>AccessDenied</Code>
    <Message>{}</Message>
    <RequestId>00000000-0000-0000-0000-000000000000</RequestId>
</Error>"#,
                message
            );

            let response = Response::builder()
                .status(StatusCode::FORBIDDEN)
                .header("content-type", "application/xml")
                .body(error_response.into())
                .unwrap();
            
            Ok(response)
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Authentication context for requests
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub access_key: String,
    pub user_id: String,
    pub is_admin: bool,
}

/// Authenticate S3 API request
async fn authenticate_request(
    headers: &HeaderMap,
    request: &Request,
    metadata: &Arc<MetadataOperations>,
) -> Result<AuthContext> {
    // Check for Authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| ObjectIOError::AuthError {
            message: "Missing Authorization header".to_string(),
        })?;

    // Parse authorization header
    let parsed_auth = AuthorizationHeader::parse(auth_header)?;
    let access_key = parsed_auth.access_key()?;

    // Look up user by access key
    let user = metadata
        .get_user_by_access_key(&access_key)
        .await?
        .ok_or_else(|| ObjectIOError::AuthError {
            message: "Invalid access key".to_string(),
        })?;

    // Extract timestamp from x-amz-date header
    let timestamp = extract_timestamp(headers)?;

    // Get payload hash
    let payload_hash = headers
        .get("x-amz-content-sha256")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("UNSIGNED-PAYLOAD");

    // Create signature request
    let sig_request = SignatureRequest {
        method: request.method(),
        uri: request.uri().path(),
        query_string: request.uri().query().unwrap_or(""),
        headers,
        payload_hash,
        timestamp,
    };

    // Validate signature
    let validator = SigV4Validator::new(
        "us-east-1".to_string(), // TODO: Get from config
        "s3".to_string(),
    );

    let is_valid = validator.validate_signature(&sig_request, &parsed_auth, &user.secret_key)?;

    if !is_valid {
        return Err(ObjectIOError::AuthError {
            message: "Signature verification failed".to_string(),
        });
    }

    Ok(AuthContext {
        access_key: user.access_key,
        user_id: user.id.unwrap_or_default().to_string(),
        is_admin: user.is_admin,
    })
}

/// Extract timestamp from request headers
fn extract_timestamp(headers: &HeaderMap) -> Result<DateTime<Utc>> {
    // Try x-amz-date first, then Date header
    let timestamp_str = headers
        .get("x-amz-date")
        .or_else(|| headers.get("date"))
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| ObjectIOError::AuthError {
            message: "Missing timestamp header (x-amz-date or Date)".to_string(),
        })?;

    // Parse timestamp (x-amz-date format: 20230101T120000Z)
    if timestamp_str.ends_with('Z') && timestamp_str.contains('T') {
        DateTime::parse_from_str(timestamp_str, "%Y%m%dT%H%M%SZ")
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|_| ObjectIOError::AuthError {
                message: "Invalid timestamp format".to_string(),
            })
    } else {
        // Try RFC 2822 format for Date header
        DateTime::parse_from_rfc2822(timestamp_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|_| ObjectIOError::AuthError {
                message: "Invalid timestamp format".to_string(),
            })
    }
}

/// Create initial admin user if none exists
pub async fn ensure_admin_user(metadata: &Arc<MetadataOperations>) -> Result<()> {
    // Check if any admin users exist
    let admin_exists = metadata.admin_user_exists().await?;
    
    if !admin_exists {
        // Create default admin user
        let access_key = "AKIAOBJECTIO12345678";
        let secret_key = "wJalrXUtnFEMI/K7MDENG+bPxRfiCYzEXAMPLEKEY";
        let display_name = "Admin User";

        metadata.create_user(access_key, secret_key, display_name).await?;
        
        println!("✅ Created default admin user:");
        println!("   Access Key: {}", access_key);
        println!("   Secret Key: {}", secret_key);
        println!("   ⚠️  Please change these credentials in production!");
    }

    Ok(())
}
