//! Database operations for ObjectIO

use crate::{models::*, ObjectDB};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

/// Bucket operations
impl ObjectDB {
    /// Create a new bucket
    #[instrument(skip(self))]
    pub async fn create_bucket(&self, bucket_info: BucketInfo) -> Result<()> {
        let key = bucket_info.name.as_bytes();
        let value = bincode::serialize(&bucket_info)?;
        
        // Check if bucket already exists
        if self.buckets.contains_key(key)? {
            return Err(anyhow!("Bucket '{}' already exists", bucket_info.name));
        }
        
        self.buckets.insert(key, value)?;
        debug!("Created bucket: {}", bucket_info.name);
        Ok(())
    }
    
    /// Get bucket information
    #[instrument(skip(self))]
    pub async fn get_bucket(&self, name: &str) -> Result<Option<BucketInfo>> {
        let key = name.as_bytes();
        match self.buckets.get(key)? {
            Some(value) => {
                let bucket_info: BucketInfo = bincode::deserialize(&value)?;
                debug!("Retrieved bucket: {}", name);
                Ok(Some(bucket_info))
            }
            None => {
                debug!("Bucket not found: {}", name);
                Ok(None)
            }
        }
    }
    
    /// Update bucket information
    #[instrument(skip(self))]
    pub async fn update_bucket(&self, bucket_info: BucketInfo) -> Result<()> {
        let key = bucket_info.name.as_bytes();
        let value = bincode::serialize(&bucket_info)?;
        
        // Check if bucket exists
        if !self.buckets.contains_key(key)? {
            return Err(anyhow!("Bucket '{}' does not exist", bucket_info.name));
        }
        
        self.buckets.insert(key, value)?;
        debug!("Updated bucket: {}", bucket_info.name);
        Ok(())
    }
    
    /// Delete a bucket
    #[instrument(skip(self))]
    pub async fn delete_bucket(&self, name: &str) -> Result<bool> {
        let key = name.as_bytes();
        match self.buckets.remove(key)? {
            Some(_) => {
                debug!("Deleted bucket: {}", name);
                Ok(true)
            }
            None => {
                debug!("Bucket not found for deletion: {}", name);
                Ok(false)
            }
        }
    }
    
    /// List all buckets
    #[instrument(skip(self))]
    pub async fn list_buckets(&self) -> Result<Vec<BucketInfo>> {
        let mut buckets = Vec::new();
        for result in self.buckets.iter() {
            let (_key, value) = result?;
            let bucket_info: BucketInfo = bincode::deserialize(&value)?;
            buckets.push(bucket_info);
        }
        debug!("Listed {} buckets", buckets.len());
        Ok(buckets)
    }
    
    /// List buckets owned by a specific user
    #[instrument(skip(self))]
    pub async fn list_buckets_by_owner(&self, owner: &str) -> Result<Vec<BucketInfo>> {
        let mut buckets = Vec::new();
        for result in self.buckets.iter() {
            let (_key, value) = result?;
            let bucket_info: BucketInfo = bincode::deserialize(&value)?;
            if bucket_info.owner == owner {
                buckets.push(bucket_info);
            }
        }
        debug!("Listed {} buckets for owner: {}", buckets.len(), owner);
        Ok(buckets)
    }
}

/// Object operations
impl ObjectDB {
    /// Store object information
    #[instrument(skip(self))]
    pub async fn put_object(&self, object_info: ObjectInfo) -> Result<()> {
        let key = format!("{}:{}", object_info.bucket, object_info.key);
        let value = bincode::serialize(&object_info)?;
        
        self.objects.insert(key.as_bytes(), value)?;
        
        // Update bucket statistics
        if let Ok(Some(mut bucket)) = self.get_bucket(&object_info.bucket).await {
            bucket.object_count += 1;
            bucket.total_size += object_info.size;
            bucket.updated_at = chrono::Utc::now();
            let _ = self.update_bucket(bucket).await;
        }
        
        debug!("Stored object: {}/{}", object_info.bucket, object_info.key);
        Ok(())
    }
    
    /// Get object information
    #[instrument(skip(self))]
    pub async fn get_object(&self, bucket: &str, key: &str) -> Result<Option<ObjectInfo>> {
        let object_key = format!("{}:{}", bucket, key);
        match self.objects.get(object_key.as_bytes())? {
            Some(value) => {
                let object_info: ObjectInfo = bincode::deserialize(&value)?;
                debug!("Retrieved object: {}/{}", bucket, key);
                Ok(Some(object_info))
            }
            None => {
                debug!("Object not found: {}/{}", bucket, key);
                Ok(None)
            }
        }
    }
    
    /// Delete object
    #[instrument(skip(self))]
    pub async fn delete_object(&self, bucket: &str, key: &str) -> Result<bool> {
        let object_key = format!("{}:{}", bucket, key);
        
        // Get object info for statistics update
        let object_size = if let Some(obj) = self.get_object(bucket, key).await? {
            obj.size
        } else {
            return Ok(false);
        };
        
        match self.objects.remove(object_key.as_bytes())? {
            Some(_) => {
                // Update bucket statistics
                if let Ok(Some(mut bucket_info)) = self.get_bucket(bucket).await {
                    bucket_info.object_count = bucket_info.object_count.saturating_sub(1);
                    bucket_info.total_size = bucket_info.total_size.saturating_sub(object_size);
                    bucket_info.updated_at = chrono::Utc::now();
                    let _ = self.update_bucket(bucket_info).await;
                }
                
                debug!("Deleted object: {}/{}", bucket, key);
                Ok(true)
            }
            None => {
                debug!("Object not found for deletion: {}/{}", bucket, key);
                Ok(false)
            }
        }
    }
    
    /// List objects in a bucket with optional prefix filter
    #[instrument(skip(self))]
    pub async fn list_objects(&self, bucket: &str, prefix: Option<&str>) -> Result<Vec<ObjectInfo>> {
        let bucket_prefix = format!("{}:", bucket);
        let mut objects = Vec::new();
        
        for result in self.objects.scan_prefix(bucket_prefix.as_bytes()) {
            let (_key, value) = result?;
            let object_info: ObjectInfo = bincode::deserialize(&value)?;
            
            // Apply prefix filter if specified
            if let Some(prefix) = prefix {
                if !object_info.key.starts_with(prefix) {
                    continue;
                }
            }
            
            objects.push(object_info);
        }
        
        debug!("Listed {} objects in bucket: {}", objects.len(), bucket);
        Ok(objects)
    }
    
    /// Get object count for a bucket
    #[instrument(skip(self))]
    pub async fn get_object_count(&self, bucket: &str) -> Result<u64> {
        let bucket_prefix = format!("{}:", bucket);
        let mut count = 0u64;
        
        for result in self.objects.scan_prefix(bucket_prefix.as_bytes()) {
            let _ = result?;
            count += 1;
        }
        
        debug!("Counted {} objects in bucket: {}", count, bucket);
        Ok(count)
    }
}

/// User operations
impl ObjectDB {
    /// Create a new user
    #[instrument(skip(self, user_info), fields(user_id = %user_info.user_id))]
    pub async fn create_user(&self, user_info: UserInfo) -> Result<()> {
        let key = user_info.access_key.as_bytes();
        let value = bincode::serialize(&user_info)?;
        
        // Check if user already exists
        if self.users.contains_key(key)? {
            return Err(anyhow!("User with access key '{}' already exists", user_info.access_key));
        }
        
        self.users.insert(key, value)?;
        debug!("Created user: {}", user_info.user_id);
        Ok(())
    }
    
    /// Get user by access key
    #[instrument(skip(self))]
    pub async fn get_user_by_access_key(&self, access_key: &str) -> Result<Option<UserInfo>> {
        let key = access_key.as_bytes();
        match self.users.get(key)? {
            Some(value) => {
                let user_info: UserInfo = bincode::deserialize(&value)?;
                debug!("Retrieved user by access key: {}", access_key);
                Ok(Some(user_info))
            }
            None => {
                debug!("User not found by access key: {}", access_key);
                Ok(None)
            }
        }
    }
    
    /// Update user information
    #[instrument(skip(self, user_info), fields(user_id = %user_info.user_id))]
    pub async fn update_user(&self, user_info: UserInfo) -> Result<()> {
        let key = user_info.access_key.as_bytes();
        let value = bincode::serialize(&user_info)?;
        
        // Check if user exists
        if !self.users.contains_key(key)? {
            return Err(anyhow!("User with access key '{}' does not exist", user_info.access_key));
        }
        
        self.users.insert(key, value)?;
        debug!("Updated user: {}", user_info.user_id);
        Ok(())
    }
    
    /// Delete user
    #[instrument(skip(self))]
    pub async fn delete_user(&self, access_key: &str) -> Result<bool> {
        let key = access_key.as_bytes();
        match self.users.remove(key)? {
            Some(_) => {
                debug!("Deleted user with access key: {}", access_key);
                Ok(true)
            }
            None => {
                debug!("User not found for deletion: {}", access_key);
                Ok(false)
            }
        }
    }
    
    /// List all users
    #[instrument(skip(self))]
    pub async fn list_users(&self) -> Result<Vec<UserInfo>> {
        let mut users = Vec::new();
        for result in self.users.iter() {
            let (_key, value) = result?;
            let user_info: UserInfo = bincode::deserialize(&value)?;
            users.push(user_info);
        }
        debug!("Listed {} users", users.len());
        Ok(users)
    }
}

/// Bulk operations
impl ObjectDB {
    /// Delete all objects in a bucket (for bucket deletion)
    #[instrument(skip(self))]
    pub async fn delete_all_objects_in_bucket(&self, bucket: &str) -> Result<u64> {
        let bucket_prefix = format!("{}:", bucket);
        let mut deleted_count = 0u64;
        
        // Collect keys to delete
        let mut keys_to_delete = Vec::new();
        for result in self.objects.scan_prefix(bucket_prefix.as_bytes()) {
            let (key, _value) = result?;
            keys_to_delete.push(key.to_vec());
        }
        
        // Delete collected keys
        for key in keys_to_delete {
            if self.objects.remove(&key)?.is_some() {
                deleted_count += 1;
            }
        }
        
        debug!("Deleted {} objects from bucket: {}", deleted_count, bucket);
        Ok(deleted_count)
    }
    
    /// Get database health check information
    #[instrument(skip(self))]
    pub async fn health_check(&self) -> Result<HealthCheck> {
        let stats = self.stats();
        
        Ok(HealthCheck {
            database_accessible: true,
            buckets_count: stats.buckets_count,
            objects_count: stats.objects_count,
            users_count: stats.users_count,
            size_on_disk: stats.size_on_disk,
            last_checked: chrono::Utc::now(),
        })
    }
}

/// Health check information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub database_accessible: bool,
    pub buckets_count: usize,
    pub objects_count: usize,
    pub users_count: usize,
    pub size_on_disk: u64,
    pub last_checked: chrono::DateTime<chrono::Utc>,
}
