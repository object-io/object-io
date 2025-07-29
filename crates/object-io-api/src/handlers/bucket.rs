//! Bucket operation handlers

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use crate::state::AppState;

/// List buckets request parameters
#[derive(Debug, Deserialize)]
pub struct ListBucketsQuery {
    #[serde(rename = "max-buckets")]
    pub max_buckets: Option<u32>,
}

/// List buckets response
#[derive(Debug, Serialize)]
pub struct ListBucketsResponse {
    pub buckets: Vec<BucketInfo>,
    pub owner: OwnerInfo,
}

/// Bucket information for listing
#[derive(Debug, Serialize)]
pub struct BucketInfo {
    pub name: String,
    pub creation_date: String,
}

/// Owner information
#[derive(Debug, Serialize)]
pub struct OwnerInfo {
    pub id: String,
    pub display_name: String,
}

/// Create bucket request
#[derive(Debug, Deserialize)]
pub struct CreateBucketRequest {
    pub location_constraint: Option<String>,
}

/// List buckets handler (GET /)
pub async fn list_buckets(
    Query(_params): Query<ListBucketsQuery>,
    State(state): State<AppState>,
) -> std::result::Result<Json<ListBucketsResponse>, StatusCode> {
    // TODO: Get actual owner from authentication context
    let owner = "default-owner";
    
    match state.metadata.list_buckets(owner).await {
        Ok(buckets) => {
            let bucket_infos: Vec<BucketInfo> = buckets
                .into_iter()
                .map(|bucket| BucketInfo {
                    name: bucket.name,
                    creation_date: bucket.created_at.to_rfc3339(),
                })
                .collect();

            let response = ListBucketsResponse {
                buckets: bucket_infos,
                owner: OwnerInfo {
                    id: owner.to_string(),
                    display_name: "Default Owner".to_string(),
                },
            };

            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Failed to list buckets: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create bucket handler (PUT /{bucket})
pub async fn create_bucket(
    Path(bucket_name): Path<String>,
    State(state): State<AppState>,
    Json(_request): Json<CreateBucketRequest>,
) -> std::result::Result<StatusCode, StatusCode> {
    // Validate bucket name
    if let Err(_) = object_io_core::validate_bucket_name(&bucket_name) {
        return Err(StatusCode::BAD_REQUEST);
    }

    // TODO: Get actual owner from authentication context
    let owner = "default-owner";
    
    match state.metadata.create_bucket(&bucket_name, owner).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            eprintln!("Failed to create bucket '{}': {}", bucket_name, e);
            
            // Check if it's a conflict (bucket already exists)
            if e.to_string().contains("already exists") {
                Err(StatusCode::CONFLICT)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// Delete bucket handler (DELETE /{bucket})
pub async fn delete_bucket(
    Path(bucket_name): Path<String>,
    State(state): State<AppState>,
) -> std::result::Result<StatusCode, StatusCode> {
    match state.metadata.delete_bucket(&bucket_name).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            eprintln!("Failed to delete bucket '{}': {}", bucket_name, e);
            
            // Check if it's a not found error
            if e.to_string().contains("not found") {
                Err(StatusCode::NOT_FOUND)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// Get bucket location handler (GET /{bucket}?location)
pub async fn get_bucket_location(
    Path(_bucket_name): Path<String>,
    Extension(_storage): Extension<Arc<dyn object_io_storage::Storage>>,
) -> std::result::Result<Json<HashMap<String, String>>, StatusCode> {
    let mut response = HashMap::new();
    response.insert("LocationConstraint".to_string(), "us-east-1".to_string());
    Ok(Json(response))
}

/// Head bucket handler (HEAD /{bucket})
pub async fn head_bucket(
    Path(bucket_name): Path<String>,
    State(state): State<AppState>,
) -> std::result::Result<StatusCode, StatusCode> {
    match state.metadata.get_bucket(&bucket_name).await {
        Ok(Some(_)) => Ok(StatusCode::OK),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to check bucket '{}': {}", bucket_name, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
