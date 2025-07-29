//! Metadata models for database operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Database representation of a bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub owner: String,
    pub acl: HashMap<String, String>,
}

/// Database representation of an object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    pub key: String,
    pub bucket: String,
    pub size: u64,
    pub content_type: String,
    pub etag: String,
    pub last_modified: String,
    pub storage_path: String,
    pub metadata: HashMap<String, String>,
}

/// Database representation of a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    pub access_key: String,
    pub secret_key: String,
    pub created_at: String,
    pub is_admin: bool,
    pub permissions: Vec<String>,
}

/// Multipart upload tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipartUpload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    pub upload_id: String,
    pub bucket: String,
    pub key: String,
    pub created_at: String,
    pub parts: Vec<PartInfo>,
}

/// Individual part information for multipart uploads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartInfo {
    pub part_number: i32,
    pub etag: String,
    pub size: u64,
    pub storage_path: String,
}
