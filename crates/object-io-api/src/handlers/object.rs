//! Object operation handlers

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use futures::StreamExt;
use serde::Deserialize;
use std::collections::HashMap;
use tokio::io::AsyncReadExt;
use crate::state::AppState;

/// Put object parameters
#[derive(Debug, Deserialize)]
pub struct PutObjectQuery {
    #[serde(rename = "Content-Type")]
    pub content_type: Option<String>,
    #[serde(rename = "x-amz-meta-")]
    pub metadata: Option<HashMap<String, String>>,
}

/// Get object parameters
#[derive(Debug, Deserialize)]
pub struct GetObjectQuery {
    #[serde(rename = "response-content-type")]
    pub response_content_type: Option<String>,
    #[serde(rename = "response-content-disposition")]
    pub response_content_disposition: Option<String>,
}

/// Put object handler (PUT /{bucket}/{key+})
pub async fn put_object(
    Path((bucket, key)): Path<(String, String)>,
    State(state): State<AppState>,
    Query(_params): Query<PutObjectQuery>,
    headers: HeaderMap,
    body: Body,
) -> std::result::Result<Response, StatusCode> {
    // Check if bucket exists
    match state.metadata.get_bucket(&bucket).await {
        Ok(Some(_)) => {},
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to check bucket '{}': {}", bucket, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    // Extract metadata from headers
    let mut metadata = HashMap::new();
    
    // Add content type
    if let Some(content_type) = headers.get("content-type") {
        if let Ok(ct_str) = content_type.to_str() {
            metadata.insert("content-type".to_string(), ct_str.to_string());
        }
    }

    // Add custom metadata (x-amz-meta-* headers)
    for (name, value) in headers.iter() {
        if let Some(name_str) = name.as_str().strip_prefix("x-amz-meta-") {
            if let Ok(value_str) = value.to_str() {
                metadata.insert(name_str.to_string(), value_str.to_string());
            }
        }
    }

    // Convert body to async reader
    let body_stream = tokio_util::io::StreamReader::new(
        body.into_data_stream().map(|result| {
            result.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))
        })
    );

    // Store object
    match state.storage.put_object(&bucket, &key, Box::new(body_stream), metadata).await {
        Ok(etag) => {
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("ETag", format!("\"{}\"", etag))
                .body(Body::empty())
                .unwrap();
            Ok(response)
        }
        Err(e) => {
            eprintln!("Failed to store object '{}/{}': {}", bucket, key, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get object handler (GET /{bucket}/{key+})
pub async fn get_object(
    Path((bucket, key)): Path<(String, String)>,
    State(state): State<AppState>,
    Query(_params): Query<GetObjectQuery>,
) -> std::result::Result<Response, StatusCode> {
    // Check if bucket exists
    match state.metadata.get_bucket(&bucket).await {
        Ok(Some(_)) => {},
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to check bucket '{}': {}", bucket, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    // Get object from storage
    match state.storage.get_object(&bucket, &key).await {
        Ok(mut reader) => {
            // Get object metadata for headers
            let metadata = match state.storage.get_object_metadata(&bucket, &key).await {
                Ok(meta) => meta,
                Err(_) => HashMap::new(),
            };

            // Create response with appropriate headers
            let mut response_builder = Response::builder().status(StatusCode::OK);

            // Set content type
            if let Some(content_type) = metadata.get("content-type") {
                response_builder = response_builder.header("content-type", content_type);
            } else {
                response_builder = response_builder.header("content-type", "application/octet-stream");
            }

            // Read the data to create body
            let mut buffer = Vec::new();
            if let Err(e) = reader.read_to_end(&mut buffer).await {
                eprintln!("Failed to read object data: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }

            let response = response_builder
                .body(Body::from(buffer))
                .unwrap();
            Ok(response)
        }
        Err(object_io_core::ObjectIOError::ObjectNotFound { .. }) => {
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            eprintln!("Failed to get object '{}/{}': {}", bucket, key, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Head object handler (HEAD /{bucket}/{key+})
pub async fn head_object(
    Path((bucket, key)): Path<(String, String)>,
    State(state): State<AppState>,
) -> std::result::Result<Response, StatusCode> {
    // Check if bucket exists
    match state.metadata.get_bucket(&bucket).await {
        Ok(Some(_)) => {},
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to check bucket '{}': {}", bucket, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    // Check if object exists and get metadata
    match state.storage.object_exists(&bucket, &key).await {
        Ok(true) => {
            // Get object metadata for headers
            let metadata = match state.storage.get_object_metadata(&bucket, &key).await {
                Ok(meta) => meta,
                Err(_) => HashMap::new(),
            };

            let mut response_builder = Response::builder().status(StatusCode::OK);

            // Set content type
            if let Some(content_type) = metadata.get("content-type") {
                response_builder = response_builder.header("content-type", content_type);
            } else {
                response_builder = response_builder.header("content-type", "application/octet-stream");
            }

            // Add custom metadata as x-amz-meta-* headers
            for (key, value) in metadata.iter() {
                if !key.starts_with("content-") {
                    response_builder = response_builder.header(
                        format!("x-amz-meta-{}", key),
                        value
                    );
                }
            }

            let response = response_builder
                .body(Body::empty())
                .unwrap();
            Ok(response)
        }
        Ok(false) => {
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            eprintln!("Failed to check object '{}/{}': {}", bucket, key, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete object handler (DELETE /{bucket}/{key+})
pub async fn delete_object(
    Path((bucket, key)): Path<(String, String)>,
    State(state): State<AppState>,
) -> std::result::Result<StatusCode, StatusCode> {
    // Check if bucket exists
    match state.metadata.get_bucket(&bucket).await {
        Ok(Some(_)) => {},
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to check bucket '{}': {}", bucket, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    // Delete object from storage
    match state.storage.delete_object(&bucket, &key).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(object_io_core::ObjectIOError::ObjectNotFound { .. }) => {
            // S3 returns 204 even if object doesn't exist
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            eprintln!("Failed to delete object '{}/{}': {}", bucket, key, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
