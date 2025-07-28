//! Core types for ObjectIO

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Represents an S3 bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bucket {
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub region: String,
    pub versioning: VersioningStatus,
    pub access_control: AccessControl,
}

/// Represents an S3 object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Object {
    pub key: String,
    pub bucket: String,
    pub size: u64,
    pub etag: String,
    pub last_modified: DateTime<Utc>,
    pub content_type: String,
    pub content_encoding: Option<String>,
    pub metadata: HashMap<String, String>,
    pub storage_class: StorageClass,
}

/// Object metadata summary (for listings)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectSummary {
    pub key: String,
    pub size: u64,
    pub etag: String,
    pub last_modified: DateTime<Utc>,
    pub storage_class: StorageClass,
}

/// Object information for head requests and metadata operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectInfo {
    pub key: String,
    pub size: u64,
    pub etag: String,
    pub last_modified: DateTime<Utc>,
    pub storage_class: String,
}

/// Bucket versioning status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VersioningStatus {
    Unversioned,
    Enabled,
    Suspended,
}

impl Default for VersioningStatus {
    fn default() -> Self {
        Self::Unversioned
    }
}

/// Storage class for objects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageClass {
    Standard,
    ReducedRedundancy,
    StandardIA,
    OneZoneIA,
    Glacier,
    DeepArchive,
}

impl Default for StorageClass {
    fn default() -> Self {
        Self::Standard
    }
}

/// Access control configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControl {
    pub owner: User,
    pub acl: Vec<Grant>,
    pub policy: Option<BucketPolicy>,
}

/// User representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub access_keys: Vec<AccessKey>,
    pub created_at: DateTime<Utc>,
}

/// Access key for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessKey {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub status: AccessKeyStatus,
}

/// Access key status
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AccessKeyStatus {
    Active,
    Inactive,
}

/// Access control grant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grant {
    pub grantee: Grantee,
    pub permission: Permission,
}

/// Grant recipient
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Grantee {
    User(Uuid),
    Group(String),
    AllUsers,
    AuthenticatedUsers,
}

/// Access permission
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Permission {
    Read,
    Write,
    ReadAcp,
    WriteAcp,
    FullControl,
}

/// Bucket policy document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketPolicy {
    pub version: String,
    pub statements: Vec<PolicyStatement>,
}

/// Policy statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyStatement {
    pub sid: Option<String>,
    pub effect: PolicyEffect,
    pub principal: Principal,
    pub action: Vec<String>,
    pub resource: Vec<String>,
    pub condition: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
}

/// Policy effect
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PolicyEffect {
    Allow,
    Deny,
}

/// Policy principal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Principal {
    AWS(Vec<String>),
    All,
}

/// Multipart upload information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipartUpload {
    pub upload_id: String,
    pub bucket: String,
    pub key: String,
    pub initiated: DateTime<Utc>,
    pub parts: Vec<UploadPart>,
}

/// Individual part of a multipart upload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadPart {
    pub part_number: u32,
    pub etag: String,
    pub size: u64,
    pub last_modified: DateTime<Utc>,
}

/// List objects request parameters
#[derive(Debug, Clone, Default)]
pub struct ListObjectsRequest {
    pub bucket: String,
    pub prefix: Option<String>,
    pub delimiter: Option<String>,
    pub marker: Option<String>,
    pub max_keys: Option<u32>,
}

/// List objects response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListObjectsResponse {
    pub bucket: String,
    pub prefix: Option<String>,
    pub delimiter: Option<String>,
    pub marker: Option<String>,
    pub next_marker: Option<String>,
    pub max_keys: u32,
    pub is_truncated: bool,
    pub objects: Vec<ObjectSummary>,
    pub common_prefixes: Vec<String>,
}
