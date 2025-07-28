//! Metadata models for database operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Database representation of a bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketRecord {
    pub id: Option<String>,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub owner: String,
    pub acl: HashMap<String, String>,
}

/// Database representation of an object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectRecord {
    pub id: Option<String>,
    pub key: String,
    pub bucket: String,
    pub size: u64,
    pub content_type: String,
    pub etag: String,
    pub last_modified: DateTime<Utc>,
    pub storage_path: String,
    pub metadata: HashMap<String, String>,
}

/// Database representation of a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRecord {
    pub id: Option<String>,
    pub access_key: String,
    pub secret_key: String,
    pub created_at: DateTime<Utc>,
    pub is_admin: bool,
    pub permissions: Vec<String>,
}

/// Multipart upload tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipartUpload {
    pub id: Option<String>,
    pub upload_id: String,
    pub bucket: String,
    pub key: String,
    pub created_at: DateTime<Utc>,
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
