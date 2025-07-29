//! Data models for ObjectIO database

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Bucket information stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketInfo {
    /// Bucket name (unique identifier)
    pub name: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last modification timestamp
    pub updated_at: DateTime<Utc>,
    /// Bucket owner (user ID or access key)
    pub owner: String,
    /// Access Control List
    pub acl: BucketAcl,
    /// Bucket region (for S3 compatibility)
    pub region: String,
    /// Versioning enabled
    pub versioning_enabled: bool,
    /// Total object count in bucket
    pub object_count: u64,
    /// Total size of all objects in bytes
    pub total_size: u64,
}

impl BucketInfo {
    /// Create a new bucket
    pub fn new(name: String, owner: String, region: String) -> Self {
        let now = Utc::now();
        Self {
            name,
            created_at: now,
            updated_at: now,
            owner,
            acl: BucketAcl::default(),
            region,
            versioning_enabled: false,
            object_count: 0,
            total_size: 0,
        }
    }
}

/// Bucket Access Control List
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketAcl {
    /// Is bucket publicly readable
    pub public_read: bool,
    /// Is bucket publicly writable
    pub public_write: bool,
    /// Specific user permissions
    pub user_permissions: HashMap<String, BucketPermission>,
}

impl Default for BucketAcl {
    fn default() -> Self {
        Self {
            public_read: false,
            public_write: false,
            user_permissions: HashMap::new(),
        }
    }
}

/// Bucket permissions for a specific user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketPermission {
    pub read: bool,
    pub write: bool,
    pub delete: bool,
    pub admin: bool,
}

/// Object information stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectInfo {
    /// Object key (full path within bucket)
    pub key: String,
    /// Bucket name containing this object
    pub bucket: String,
    /// Object size in bytes
    pub size: u64,
    /// Content type (MIME type)
    pub content_type: String,
    /// ETag for the object
    pub etag: String,
    /// Last modification timestamp
    pub last_modified: DateTime<Utc>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Object metadata (custom headers)
    pub metadata: HashMap<String, String>,
    /// Storage class
    pub storage_class: StorageClass,
    /// Version ID (for versioned buckets)
    pub version_id: Option<String>,
    /// Is this a delete marker (for versioned buckets)
    pub is_delete_marker: bool,
    /// Content encoding
    pub content_encoding: Option<String>,
    /// Content language
    pub content_language: Option<String>,
    /// Cache control
    pub cache_control: Option<String>,
    /// Content disposition
    pub content_disposition: Option<String>,
}

impl ObjectInfo {
    /// Create new object info
    pub fn new(
        key: String,
        bucket: String,
        size: u64,
        content_type: String,
        etag: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            key,
            bucket,
            size,
            content_type,
            etag,
            last_modified: now,
            created_at: now,
            metadata: HashMap::new(),
            storage_class: StorageClass::Standard,
            version_id: None,
            is_delete_marker: false,
            content_encoding: None,
            content_language: None,
            cache_control: None,
            content_disposition: None,
        }
    }
}

/// Storage class for objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageClass {
    Standard,
    ReducedRedundancy,
    Glacier,
    DeepArchive,
}

impl Default for StorageClass {
    fn default() -> Self {
        StorageClass::Standard
    }
}

/// User information for authentication and authorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    /// User ID (unique identifier)
    pub user_id: String,
    /// Access key for S3 API
    pub access_key: String,
    /// Secret key for S3 API (hashed)
    pub secret_key_hash: String,
    /// User display name
    pub display_name: String,
    /// User email
    pub email: String,
    /// Is user active
    pub active: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last access timestamp
    pub last_access: Option<DateTime<Utc>>,
    /// User permissions
    pub permissions: UserPermissions,
}

impl UserInfo {
    /// Create a new user
    pub fn new(
        user_id: String,
        access_key: String,
        secret_key_hash: String,
        display_name: String,
        email: String,
    ) -> Self {
        Self {
            user_id,
            access_key,
            secret_key_hash,
            display_name,
            email,
            active: true,
            created_at: Utc::now(),
            last_access: None,
            permissions: UserPermissions::default(),
        }
    }
}

/// User permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissions {
    /// Can create buckets
    pub create_bucket: bool,
    /// Can delete buckets
    pub delete_bucket: bool,
    /// Can list all buckets
    pub list_all_buckets: bool,
    /// Is system administrator
    pub admin: bool,
}

impl Default for UserPermissions {
    fn default() -> Self {
        Self {
            create_bucket: true,
            delete_bucket: true,
            list_all_buckets: false,
            admin: false,
        }
    }
}
