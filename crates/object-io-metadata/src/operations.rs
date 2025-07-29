//! Metadata operations for buckets, objects, and users

use crate::{database::Database, models::*};
use chrono::{DateTime, Utc};
use object_io_core::{Bucket, Object, ObjectInfo, Result};
use std::collections::HashMap;

/// Metadata operations interface
pub struct MetadataOperations {
    db: Database,
}

impl MetadataOperations {
    /// Create new metadata operations instance
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    // Bucket operations
    
    /// Create a new bucket
    pub async fn create_bucket(&self, name: &str, owner: &str) -> Result<Bucket> {
        let now = Utc::now();
        let bucket_record = BucketRecord {
            id: None,
            name: name.to_string(),
            created_at: now.to_rfc3339(),
            updated_at: now.to_rfc3339(),
            owner: owner.to_string(),
            acl: HashMap::new(),
        };

        let created: Option<serde_json::Value> = self.db.connection()
            .create(("bucket", uuid::Uuid::new_v4().to_string()))
            .content(bucket_record)
            .await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to create bucket: {}", e),
            })?;

        let record_value = created
            .ok_or_else(|| object_io_core::ObjectIOError::DatabaseError {
                message: "No bucket record returned from creation".to_string(),
            })?;

        let record: BucketRecord = serde_json::from_value(record_value)
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to deserialize bucket record: {}", e),
            })?;

        Ok(Bucket {
            name: record.name,
            created_at: DateTime::parse_from_rfc3339(&record.created_at)
                .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                    message: format!("Failed to parse created_at: {}", e),
                })?
                .with_timezone(&Utc),
            region: "us-east-1".to_string(), // Default region
            versioning: object_io_core::VersioningStatus::default(),
            access_control: object_io_core::AccessControl {
                owner: object_io_core::User {
                    id: uuid::Uuid::new_v4(),
                    name: record.owner.clone(),
                    email: format!("{}@localhost", record.owner),
                    access_keys: vec![],
                    created_at: DateTime::parse_from_rfc3339(&record.created_at)
                        .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                            message: format!("Failed to parse created_at: {}", e),
                        })?
                        .with_timezone(&Utc),
                },
                acl: vec![],
                policy: None,
            },
        })
    }

    /// Get bucket by name
    pub async fn get_bucket(&self, name: &str) -> Result<Option<Bucket>> {
        let result: Vec<BucketRecord> = self.db.connection()
            .query("SELECT * FROM bucket WHERE name = $name")
            .bind(("name", name))
            .await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to query bucket: {}", e),
            })?
            .take(0)
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to parse bucket query result: {}", e),
            })?;

        Ok(result.into_iter().next().map(|record| Bucket {
            name: record.name,
            created_at: DateTime::parse_from_rfc3339(&record.created_at)
                .unwrap_or_else(|_| Utc::now().into())
                .with_timezone(&Utc),
            region: "us-east-1".to_string(), // Default region
            versioning: object_io_core::VersioningStatus::default(),
            access_control: object_io_core::AccessControl {
                owner: object_io_core::User {
                    id: uuid::Uuid::new_v4(),
                    name: record.owner.clone(),
                    email: format!("{}@localhost", record.owner),
                    access_keys: vec![],
                    created_at: DateTime::parse_from_rfc3339(&record.created_at)
                        .unwrap_or_else(|_| Utc::now().into())
                        .with_timezone(&Utc),
                },
                acl: vec![],
                policy: None,
            },
        }))
    }

    /// List all buckets for a user
    pub async fn list_buckets(&self, owner: &str) -> Result<Vec<Bucket>> {
        let result: Vec<BucketRecord> = self.db.connection()
            .query("SELECT * FROM bucket WHERE owner = $owner ORDER BY created_at")
            .bind(("owner", owner))
            .await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to list buckets: {}", e),
            })?
            .take(0)
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to parse bucket list result: {}", e),
            })?;

        Ok(result.into_iter().map(|record| Bucket {
            name: record.name,
            created_at: DateTime::parse_from_rfc3339(&record.created_at)
                .unwrap_or_else(|_| Utc::now().into())
                .with_timezone(&Utc),
            region: "us-east-1".to_string(), // Default region
            versioning: object_io_core::VersioningStatus::default(),
            access_control: object_io_core::AccessControl {
                owner: object_io_core::User {
                    id: uuid::Uuid::new_v4(),
                    name: record.owner.clone(),
                    email: format!("{}@localhost", record.owner),
                    access_keys: vec![],
                    created_at: DateTime::parse_from_rfc3339(&record.created_at)
                        .unwrap_or_else(|_| Utc::now().into())
                        .with_timezone(&Utc),
                },
                acl: vec![],
                policy: None,
            },
        }).collect())
    }

    /// Delete a bucket
    pub async fn delete_bucket(&self, name: &str) -> Result<()> {
        self.db.connection()
            .query("DELETE FROM bucket WHERE name = $name")
            .bind(("name", name))
            .await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to delete bucket: {}", e),
            })?;

        Ok(())
    }

    // Object operations

    /// Store object metadata
    pub async fn put_object_metadata(
        &self,
        bucket: &str,
        key: &str,
        size: u64,
        content_type: &str,
        etag: &str,
        storage_path: &str,
        metadata: HashMap<String, String>,
    ) -> Result<ObjectInfo> {
        let object_record = ObjectRecord {
            id: None,
            key: key.to_string(),
            bucket: bucket.to_string(),
            size,
            content_type: content_type.to_string(),
            etag: etag.to_string(),
            last_modified: Utc::now().to_rfc3339(),
            storage_path: storage_path.to_string(),
            metadata,
        };

        let created: Option<serde_json::Value> = self.db.connection()
            .create(("object", uuid::Uuid::new_v4().to_string()))
            .content(object_record)
            .await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to store object metadata: {}", e),
            })?;

        let record_value = created
            .ok_or_else(|| object_io_core::ObjectIOError::DatabaseError {
                message: "No object record returned from creation".to_string(),
            })?;

        let record: ObjectRecord = serde_json::from_value(record_value)
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to deserialize object record: {}", e),
            })?;

        Ok(ObjectInfo {
            key: record.key,
            last_modified: DateTime::parse_from_rfc3339(&record.last_modified)
                .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                    message: format!("Failed to parse last_modified: {}", e),
                })?
                .with_timezone(&Utc),
            etag: record.etag,
            size: record.size,
            storage_class: "STANDARD".to_string(),
        })
    }

    /// Get object metadata
    pub async fn get_object_metadata(&self, bucket: &str, key: &str) -> Result<Option<ObjectInfo>> {
        let result: Vec<ObjectRecord> = self.db.connection()
            .query("SELECT * FROM object WHERE bucket = $bucket AND key = $key")
            .bind(("bucket", bucket))
            .bind(("key", key))
            .await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to query object metadata: {}", e),
            })?
            .take(0)
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to parse object metadata result: {}", e),
            })?;

        Ok(result.into_iter().next().map(|record| ObjectInfo {
            key: record.key,
            last_modified: DateTime::parse_from_rfc3339(&record.last_modified)
                .unwrap_or_else(|_| Utc::now().into())
                .with_timezone(&Utc),
            etag: record.etag,
            size: record.size,
            storage_class: "STANDARD".to_string(),
        }))
    }

    /// List objects in a bucket
    pub async fn list_objects(
        &self,
        bucket: &str,
        prefix: Option<&str>,
        max_keys: Option<u32>,
    ) -> Result<Vec<Object>> {
        let mut query = "SELECT * FROM object WHERE bucket = $bucket".to_string();
        let mut params = vec![("bucket", bucket.to_string())];

        if let Some(prefix) = prefix {
            query.push_str(" AND string::startsWith(key, $prefix)");
            params.push(("prefix", prefix.to_string()));
        }

        query.push_str(" ORDER BY key");

        if let Some(limit) = max_keys {
            query.push_str(" LIMIT $limit");
            params.push(("limit", limit.to_string()));
        }

        let mut query_builder = self.db.connection().query(&query);
        for (key, value) in params {
            query_builder = query_builder.bind((key, value));
        }

        let result: Vec<ObjectRecord> = query_builder
            .await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to list objects: {}", e),
            })?
            .take(0)
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to parse object list result: {}", e),
            })?;

        Ok(result.into_iter().map(|record| Object {
            key: record.key,
            bucket: record.bucket,
            size: record.size,
            etag: record.etag,
            last_modified: DateTime::parse_from_rfc3339(&record.last_modified)
                .unwrap_or_else(|_| Utc::now().into())
                .with_timezone(&Utc),
            content_type: record.content_type,
            content_encoding: None,
            metadata: record.metadata,
            storage_class: object_io_core::StorageClass::default(),
        }).collect())
    }

    /// Delete object metadata
    pub async fn delete_object(&self, bucket: &str, key: &str) -> Result<()> {
        self.db.connection()
            .query("DELETE FROM object WHERE bucket = $bucket AND key = $key")
            .bind(("bucket", bucket))
            .bind(("key", key))
            .await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to delete object metadata: {}", e),
            })?;

        Ok(())
    }

    // User operations

    /// Create a new user
    pub async fn create_user(&self, user: &User) -> Result<User> {
        let user_record = UserRecord::from(user.clone());
        
        let created: Option<serde_json::Value> = self.db.connection()
            .create(("user", uuid::Uuid::new_v4().to_string()))
            .content(user_record)
            .await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to create user: {}", e),
            })?;

        let record_value = created
            .ok_or_else(|| object_io_core::ObjectIOError::DatabaseError {
                message: "No user record returned from creation".to_string(),
            })?;

        let record: UserRecord = serde_json::from_value(record_value)
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to deserialize user record: {}", e),
            })?;

        Ok(User::from(record))
    }

    /// Get user by access key
    pub async fn get_user_by_access_key(&self, access_key: &str) -> Result<Option<User>> {
        let results: Vec<UserRecord> = self.db.connection()
            .query("SELECT * FROM user WHERE access_key = $access_key")
            .bind(("access_key", access_key))
            .await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to query user: {}", e),
            })?
            .take(0)
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to parse user query result: {}", e),
            })?;

        if results.is_empty() {
            return Ok(None);
        }

        Ok(Some(User::from(results[0].clone())))
    }

    /// Check if any admin users exist
    pub async fn admin_user_exists(&self) -> Result<bool> {
        let results: Vec<serde_json::Value> = self.db.connection()
            .query("SELECT COUNT() as count FROM user WHERE is_admin = true")
            .await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to check admin users: {}", e),
            })?
            .take(0)
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to parse admin check result: {}", e),
            })?;

        if let Some(result) = results.first() {
            if let Some(count) = result.get("count").and_then(|v| v.as_u64()) {
                return Ok(count > 0);
            }
        }

        Ok(false)
    }

    /// List all users (admin only)
    pub async fn list_users(&self) -> Result<Vec<User>> {
        let results: Vec<UserRecord> = self.db.connection()
            .query("SELECT * FROM user ORDER BY created_at DESC")
            .await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to list users: {}", e),
            })?
            .take(0)
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to parse user list result: {}", e),
            })?;

        Ok(results.into_iter().map(User::from).collect())
    }

    /// Delete user by access key
    pub async fn delete_user(&self, access_key: &str) -> Result<()> {
        self.db.connection()
            .query("DELETE FROM user WHERE access_key = $access_key")
            .bind(("access_key", access_key))
            .await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to delete user: {}", e),
            })?;

        Ok(())
    }
}
