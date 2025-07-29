//! Metadata operations for buckets, objects, and users

use crate::{database::Database, models::*};
use object_io_core::{Bucket, Object, ObjectInfo, Result, StorageClass, VersioningStatus, AccessControl, User};
use object_io_database::{BucketInfo, ObjectInfo as DbObjectInfo, UserInfo};
use uuid::Uuid;

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
        let bucket_info = BucketInfo::new(
            name.to_string(),
            owner.to_string(),
            "us-east-1".to_string(), // Default region
        );

        self.db.connection()
            .create_bucket(bucket_info.clone())
            .await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to create bucket: {}", e),
            })?;

        Ok(Bucket {
            name: bucket_info.name,
            created_at: bucket_info.created_at,
            region: bucket_info.region,
            versioning: VersioningStatus::Unversioned,
            access_control: AccessControl {
                owner: User {
                    id: Uuid::new_v4(),
                    name: bucket_info.owner,
                    email: "owner@objectio.local".to_string(),
                    access_keys: vec![],
                    created_at: bucket_info.created_at,
                },
                acl: vec![],
                policy: None,
            },
        })
    }

    /// Get bucket by name
    pub async fn get_bucket(&self, name: &str) -> Result<Option<Bucket>> {
        match self.db.connection()
            .get_bucket(name)
            .await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to get bucket: {}", e),
            })? {
            Some(bucket_info) => Ok(Some(Bucket {
                name: bucket_info.name,
                created_at: bucket_info.created_at,
                region: bucket_info.region,
                versioning: VersioningStatus::Unversioned,
                access_control: AccessControl {
                    owner: User {
                        id: Uuid::new_v4(),
                        name: bucket_info.owner,
                        email: "owner@objectio.local".to_string(),
                        access_keys: vec![],
                        created_at: bucket_info.created_at,
                    },
                    acl: vec![],
                    policy: None,
                },
            })),
            None => Ok(None),
        }
    }

    /// Check if bucket exists
    pub async fn bucket_exists(&self, name: &str) -> Result<bool> {
        Ok(self.get_bucket(name).await?.is_some())
    }

    /// List buckets for owner
    pub async fn list_buckets(&self, owner: &str) -> Result<Vec<Bucket>> {
        let bucket_infos = self.db.connection()
            .list_buckets_by_owner(owner)
            .await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to list buckets: {}", e),
            })?;

        Ok(bucket_infos.into_iter().map(|info| Bucket {
            name: info.name,
            created_at: info.created_at,
            region: info.region,
            versioning: VersioningStatus::Unversioned,
            access_control: AccessControl {
                owner: User {
                    id: Uuid::new_v4(),
                    name: info.owner,
                    email: "owner@objectio.local".to_string(),
                    access_keys: vec![],
                    created_at: info.created_at,
                },
                acl: vec![],
                policy: None,
            },
        }).collect())
    }

    /// Delete bucket
    pub async fn delete_bucket(&self, name: &str) -> Result<bool> {
        // First delete all objects in the bucket
        let _deleted_objects = self.db.connection()
            .delete_all_objects_in_bucket(name)
            .await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to delete objects in bucket: {}", e),
            })?;

        // Then delete the bucket itself
        let deleted = self.db.connection()
            .delete_bucket(name)
            .await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to delete bucket: {}", e),
            })?;

        Ok(deleted)
    }

    // Object operations

    /// Store object metadata
    pub async fn put_object(&self, bucket: &str, key: &str, object_info: &ObjectInfo) -> Result<()> {
        let db_object_info = DbObjectInfo::new(
            key.to_string(),
            bucket.to_string(),
            object_info.size,
            "application/octet-stream".to_string(), // Default content type
            object_info.etag.clone(),
        );

        self.db.connection()
            .put_object(db_object_info)
            .await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to store object metadata: {}", e),
            })?;

        Ok(())
    }

    /// Get object metadata
    pub async fn get_object(&self, bucket: &str, key: &str) -> Result<Option<Object>> {
        match self.db.connection()
            .get_object(bucket, key)
            .await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to get object: {}", e),
            })? {
            Some(object_info) => Ok(Some(Object {
                key: object_info.key,
                bucket: object_info.bucket,
                size: object_info.size,
                etag: object_info.etag,
                last_modified: object_info.last_modified,
                content_type: object_info.content_type,
                content_encoding: object_info.content_encoding,
                metadata: object_info.metadata,
                storage_class: StorageClass::Standard,
            })),
            None => Ok(None),
        }
    }

    /// List objects in bucket
    pub async fn list_objects(&self, bucket: &str, prefix: Option<&str>, _max_keys: Option<u32>) -> Result<Vec<Object>> {
        let object_infos = self.db.connection()
            .list_objects(bucket, prefix)
            .await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to list objects: {}", e),
            })?;

        Ok(object_infos.into_iter().map(|info| Object {
            key: info.key,
            bucket: info.bucket,
            size: info.size,
            etag: info.etag,
            last_modified: info.last_modified,
            content_type: info.content_type,
            content_encoding: info.content_encoding,
            metadata: info.metadata,
            storage_class: StorageClass::Standard,
        }).collect())
    }

    /// Delete object
    pub async fn delete_object(&self, bucket: &str, key: &str) -> Result<bool> {
        self.db.connection()
            .delete_object(bucket, key)
            .await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to delete object: {}", e),
            })
    }

    /// Get object count for bucket
    pub async fn get_object_count(&self, bucket: &str) -> Result<u64> {
        self.db.connection()
            .get_object_count(bucket)
            .await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to get object count: {}", e),
            })
    }

    // User operations

    /// Create user
    pub async fn create_user(&self, access_key: &str, secret_key_hash: &str, display_name: &str) -> Result<()> {
        let user_info = UserInfo::new(
            uuid::Uuid::new_v4().to_string(),
            access_key.to_string(),
            secret_key_hash.to_string(),
            display_name.to_string(),
            format!("{}@objectio.local", access_key), // Default email
        );

        self.db.connection()
            .create_user(user_info)
            .await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to create user: {}", e),
            })?;

        Ok(())
    }

    /// Get user by access key
    pub async fn get_user_by_access_key(&self, access_key: &str) -> Result<Option<UserRecord>> {
        match self.db.connection()
            .get_user_by_access_key(access_key)
            .await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to get user: {}", e),
            })? {
            Some(user_info) => Ok(Some(UserRecord {
                id: Some(serde_json::Value::String(user_info.user_id)),
                access_key: user_info.access_key,
                secret_key: user_info.secret_key_hash,
                created_at: user_info.created_at.to_rfc3339(),
                is_admin: user_info.permissions.admin,
                permissions: vec![], // Convert from our permissions structure if needed
            })),
            None => Ok(None),
        }
    }

    /// Check if any admin users exist
    pub async fn admin_user_exists(&self) -> Result<bool> {
        let users = self.list_users().await?;
        Ok(users.iter().any(|user| user.is_admin))
    }

    /// Check if any users exist (for initial setup)
    pub async fn user_count(&self) -> Result<u64> {
        let users = self.db.connection()
            .list_users()
            .await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to count users: {}", e),
            })?;

        Ok(users.len() as u64)
    }

    /// List all users
    pub async fn list_users(&self) -> Result<Vec<UserRecord>> {
        let user_infos = self.db.connection()
            .list_users()
            .await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to list users: {}", e),
            })?;

        Ok(user_infos.into_iter().map(|info| UserRecord {
            id: Some(serde_json::Value::String(info.user_id)),
            access_key: info.access_key,
            secret_key: info.secret_key_hash,
            created_at: info.created_at.to_rfc3339(),
            is_admin: info.permissions.admin,
            permissions: vec![], // Convert from our permissions structure if needed
        }).collect())
    }

    /// Delete user
    pub async fn delete_user(&self, access_key: &str) -> Result<bool> {
        self.db.connection()
            .delete_user(access_key)
            .await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to delete user: {}", e),
            })
    }
}
