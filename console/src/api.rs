use crate::types::{Bucket, ObjectInfo, CreateBucketRequest, SystemStats, ApiResponse};
use gloo_net::http::Request;

const API_BASE: &str = "/api/v1";

pub async fn get_system_stats() -> Result<SystemStats, String> {
    let response = Request::get(&format!("{}/stats", API_BASE))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        let api_response: ApiResponse<SystemStats> = response
            .json()
            .await
            .map_err(|e| format!("JSON parse error: {}", e))?;
            
        api_response.data.ok_or_else(|| {
            api_response.error.unwrap_or_else(|| "Unknown error".to_string())
        })
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}

pub async fn list_buckets() -> Result<Vec<Bucket>, String> {
    let response = Request::get(&format!("{}/buckets", API_BASE))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        let api_response: ApiResponse<Vec<Bucket>> = response
            .json()
            .await
            .map_err(|e| format!("JSON parse error: {}", e))?;
            
        api_response.data.ok_or_else(|| {
            api_response.error.unwrap_or_else(|| "Unknown error".to_string())
        })
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}

pub async fn create_bucket(request: CreateBucketRequest) -> Result<Bucket, String> {
    let response = Request::post(&format!("{}/buckets", API_BASE))
        .json(&request)
        .map_err(|e| format!("JSON serialize error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        let api_response: ApiResponse<Bucket> = response
            .json()
            .await
            .map_err(|e| format!("JSON parse error: {}", e))?;
            
        api_response.data.ok_or_else(|| {
            api_response.error.unwrap_or_else(|| "Unknown error".to_string())
        })
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}

pub async fn delete_bucket(bucket_name: &str) -> Result<(), String> {
    let response = Request::delete(&format!("{}/buckets/{}", API_BASE, bucket_name))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        Ok(())
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}

pub async fn list_objects(bucket_name: &str) -> Result<Vec<ObjectInfo>, String> {
    let response = Request::get(&format!("{}/buckets/{}/objects", API_BASE, bucket_name))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        let api_response: ApiResponse<Vec<ObjectInfo>> = response
            .json()
            .await
            .map_err(|e| format!("JSON parse error: {}", e))?;
            
        api_response.data.ok_or_else(|| {
            api_response.error.unwrap_or_else(|| "Unknown error".to_string())
        })
    } else {
        Err(format!("HTTP error: {}", response.status()))
    }
}
