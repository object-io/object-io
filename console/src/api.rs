use crate::types::{Bucket, ObjectInfo, CreateBucketRequest, SystemStats};
use gloo_net::http::Request;
use serde::Deserialize;
use chrono::{DateTime, Utc};

const API_BASE: &str = "http://localhost:5500";

/// S3-compatible list buckets response
#[derive(Debug, Deserialize)]
pub struct ListBucketsResponse {
    pub buckets: Vec<BucketInfo>,
    pub owner: OwnerInfo,
}

#[derive(Debug, Deserialize)]
pub struct BucketInfo {
    pub name: String,
    pub creation_date: String,
}

#[derive(Debug, Deserialize)]
pub struct OwnerInfo {
    pub id: String,
    pub display_name: String,
}

pub async fn get_system_stats() -> Result<SystemStats, String> {
    // For now, return mock stats since we don't have a stats endpoint yet
    Ok(SystemStats {
        total_buckets: 0,
        total_objects: 0,
        total_size_bytes: 0,
        storage_usage_percent: 0.0,
    })
}

pub async fn list_buckets() -> Result<Vec<Bucket>, String> {
    let response = Request::get(API_BASE)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        let s3_response: ListBucketsResponse = response
            .json()
            .await
            .map_err(|e| format!("JSON parse error: {}", e))?;
            
        let buckets = s3_response.buckets.into_iter().map(|bucket_info| Bucket {
            name: bucket_info.name,
            created_at: chrono::Utc::now(), // Use current time for now
            objects_count: 0,
            size_bytes: 0,
            region: "us-east-1".to_string(),
            versioning_enabled: false,
        }).collect();
        
        Ok(buckets)
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}

pub async fn create_bucket(request: CreateBucketRequest) -> Result<Bucket, String> {
    let response = Request::put(&format!("{}/{}", API_BASE, request.name))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "location_constraint": "us-east-1"
        }))
        .map_err(|e| format!("JSON serialize error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        Ok(Bucket {
            name: request.name,
            created_at: chrono::Utc::now(),
            objects_count: 0,
            size_bytes: 0,
            region: "us-east-1".to_string(),
            versioning_enabled: false,
        })
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}

pub async fn delete_bucket(bucket_name: &str) -> Result<(), String> {
    let response = Request::delete(&format!("{}/{}", API_BASE, bucket_name))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        Ok(())
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}

pub async fn list_objects(_bucket_name: &str) -> Result<Vec<ObjectInfo>, String> {
    // For now, return empty list since we need to implement S3 list objects endpoint
    // TODO: Implement GET /{bucket}?list-type=2 endpoint
    Ok(vec![])
}

pub async fn upload_object(bucket_name: &str, key: &str, data: Vec<u8>) -> Result<ObjectInfo, String> {
    let response = Request::put(&format!("{}/{}/{}", API_BASE, bucket_name, key))
        .header("Content-Type", "application/octet-stream")
        .body(data)
        .map_err(|e| format!("Request error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        Ok(ObjectInfo {
            key: key.to_string(),
            size: 0, // TODO: Get actual size
            last_modified: chrono::Utc::now(),
            etag: "\"example-etag\"".to_string(),
            content_type: "application/octet-stream".to_string(),
            storage_class: "STANDARD".to_string(),
        })
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}

pub async fn delete_object(bucket_name: &str, key: &str) -> Result<(), String> {
    let response = Request::delete(&format!("{}/{}/{}", API_BASE, bucket_name, key))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        Ok(())
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}
