//! Object operation handlers

use axum::{
    body::Body,
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
    response::{Json, Response},
    Extension,
};
use object_io_core::Result as CoreResult;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio_util::io::ReaderStream;

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

/// List objects parameters
#[derive(Debug, Deserialize)]
pub struct ListObjectsQuery {
    pub prefix: Option<String>,
    pub delimiter: Option<String>,
    pub marker: Option<String>,
    #[serde(rename = "max-keys")]
    pub max_keys: Option<u32>,
    #[serde(rename = "list-type")]
    pub list_type: Option<u32>,
    #[serde(rename = "continuation-token")]
    pub continuation_token: Option<String>,
}

/// List objects response
#[derive(Debug, Serialize)]
pub struct ListObjectsResponse {
    pub name: String,
    pub prefix: Option<String>,
    pub delimiter: Option<String>,
    pub max_keys: u32,
    pub is_truncated: bool,
    pub contents: Vec<ObjectSummary>,
    pub common_prefixes: Vec<CommonPrefix>,
    pub next_continuation_token: Option<String>,
}

/// Object summary for listing
#[derive(Debug, Serialize)]
pub struct ObjectSummary {
    pub key: String,
    pub last_modified: String,
    pub etag: String,
    pub size: u64,
    pub storage_class: String,
}

/// Common prefix for listing
#[derive(Debug, Serialize)]
pub struct CommonPrefix {
    pub prefix: String,
}

/// Put object handler (PUT /{bucket}/{key+})
pub async fn put_object(
    Path((bucket, key)): Path<(String, String)>,
    Query(params): Query<PutObjectQuery>,
    headers: HeaderMap,
    body: Body,
    Extension(storage): Extension<Arc<dyn object_io_storage::Storage>>,
) -> std::result::Result<Response, StatusCode> {
    // Convert body to bytes
    let bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    // Prepare metadata
    let mut metadata = HashMap::new();
    if let Some(content_type) = params.content_type {
        metadata.insert("content-type".to_string(), content_type);
    }

    // Extract additional metadata from headers
    for (name, value) in headers.iter() {
        let name_str = name.as_str();
        if name_str.starts_with("x-amz-meta-") {
            if let Ok(value_str) = value.to_str() {
                metadata.insert(name_str.to_string(), value_str.to_string());
            }
        }
    }

    // Create a reader from bytes
    let reader = std::io::Cursor::new(bytes.clone());
    let boxed_reader: Box<dyn tokio::io::AsyncRead + Send + Unpin> = Box::new(reader);

    // Store the object
    match storage.put_object(&bucket, &key, boxed_reader, metadata).await {
        Ok(etag) => {
            let mut response = Response::new(Body::empty());
            *response.status_mut() = StatusCode::OK;
            response.headers_mut().insert("ETag", etag.parse::<axum::http::HeaderValue>().unwrap());
            Ok(response)
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get object handler (GET /{bucket}/{key+})
pub async fn get_object(
    Path((bucket, key)): Path<(String, String)>,
    Query(params): Query<GetObjectQuery>,
    Extension(storage): Extension<Arc<dyn object_io_storage::Storage>>,
) -> std::result::Result<Response, StatusCode> {
    match storage.get_object(&bucket, &key).await {
        Ok(reader) => {
            // Convert the reader to a stream
            let stream = ReaderStream::new(reader);
            let body = Body::from_stream(stream);
            
            let mut response = Response::new(body);
            *response.status_mut() = StatusCode::OK;
            
            // Add custom response headers if specified
            if let Some(content_type) = params.response_content_type {
                response.headers_mut().insert("Content-Type", content_type.parse::<axum::http::HeaderValue>().unwrap());
            }
            if let Some(disposition) = params.response_content_disposition {
                response.headers_mut().insert("Content-Disposition", disposition.parse::<axum::http::HeaderValue>().unwrap());
            }
            
            Ok(response)
        }
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

/// Head object handler (HEAD /{bucket}/{key+})
pub async fn head_object(
    Path((bucket, key)): Path<(String, String)>,
    Extension(storage): Extension<Arc<dyn object_io_storage::Storage>>,
) -> std::result::Result<Response, StatusCode> {
    match storage.get_object_metadata(&bucket, &key).await {
        Ok(metadata) => {
            let mut response = Response::new(Body::empty());
            *response.status_mut() = StatusCode::OK;
            
            // Add metadata headers
            for (k, v) in metadata {
                if let (Ok(header_name), Ok(header_value)) = (
                    k.parse::<axum::http::HeaderName>(), 
                    v.parse::<axum::http::HeaderValue>()
                ) {
                    response.headers_mut().insert(header_name, header_value);
                }
            }
            
            Ok(response)
        }
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

/// Delete object handler (DELETE /{bucket}/{key+})
pub async fn delete_object(
    Path((bucket, key)): Path<(String, String)>,
    Extension(storage): Extension<Arc<dyn object_io_storage::Storage>>,
) -> std::result::Result<StatusCode, StatusCode> {
    match storage.delete_object(&bucket, &key).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

/// List objects handler (GET /{bucket})
pub async fn list_objects(
    Path(bucket): Path<String>,
    Query(params): Query<ListObjectsQuery>,
    Extension(storage): Extension<Arc<dyn object_io_storage::Storage>>,
) -> std::result::Result<Json<ListObjectsResponse>, StatusCode> {
    let max_keys = params.max_keys.unwrap_or(1000).min(1000);
    
    match storage.list_objects(
        &bucket,
        params.prefix.as_deref(),
        params.delimiter.as_deref(),
        Some(max_keys),
    ).await {
        Ok(objects) => {
            let contents: Vec<ObjectSummary> = objects
                .into_iter()
                .map(|obj| ObjectSummary {
                    key: obj.key,
                    last_modified: obj.last_modified.to_rfc3339(),
                    etag: obj.etag,
                    size: obj.size,
                    storage_class: "STANDARD".to_string(),
                })
                .collect();
            
            let response = ListObjectsResponse {
                name: bucket,
                prefix: params.prefix,
                delimiter: params.delimiter,
                max_keys,
                is_truncated: false, // TODO: Implement pagination
                contents,
                common_prefixes: vec![], // TODO: Implement common prefixes
                next_continuation_token: None,
            };
            
            Ok(Json(response))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
