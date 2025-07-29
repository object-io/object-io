//! Metadata models for database operations

use chrono::{DateTime, Utc};
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

/// Public User type for API operations
#[derive(Debug, Clone)]
pub struct User {
    pub id: Option<String>,
    pub access_key: String,
    pub secret_key: String,
    pub created_at: DateTime<Utc>,
    pub is_admin: bool,
    pub permissions: Vec<String>,
}

impl From<UserRecord> for User {
    fn from(record: UserRecord) -> Self {
        Self {
            id: record.id.map(|v| v.to_string()),
            access_key: record.access_key,
            secret_key: record.secret_key,
            created_at: DateTime::parse_from_rfc3339(&record.created_at)
                .unwrap_or_else(|_| chrono::Utc::now().into())
                .with_timezone(&Utc),
            is_admin: record.is_admin,
            permissions: record.permissions,
        }
    }
}

impl From<User> for UserRecord {
    fn from(user: User) -> Self {
        Self {
            id: user.id.map(|id| serde_json::Value::String(id)),
            access_key: user.access_key,
            secret_key: user.secret_key,
            created_at: user.created_at.to_rfc3339(),
            is_admin: user.is_admin,
            permissions: user.permissions,
        }
    }
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
