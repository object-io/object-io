//! Storage trait definitions

use object_io_core::{Object, Result};
use std::collections::HashMap;
use tokio::io::AsyncRead;

/// Core storage trait for object operations
#[async_trait::async_trait]
pub trait Storage: Send + Sync {
    /// Store an object with the given key and data stream
    async fn put_object(
        &self,
        bucket: &str,
        key: &str,
        data: Box<dyn AsyncRead + Send + Unpin>,
        metadata: HashMap<String, String>,
    ) -> Result<String>;

    /// Retrieve an object by key
    async fn get_object(&self, bucket: &str, key: &str) -> Result<Box<dyn AsyncRead + Send + Unpin>>;

    /// Delete an object by key
    async fn delete_object(&self, bucket: &str, key: &str) -> Result<()>;

    /// Check if an object exists
    async fn object_exists(&self, bucket: &str, key: &str) -> Result<bool>;

    /// Get object metadata
    async fn get_object_metadata(&self, bucket: &str, key: &str) -> Result<HashMap<String, String>>;

    /// List objects in a bucket with optional prefix
    async fn list_objects(
        &self,
        bucket: &str,
        prefix: Option<&str>,
        delimiter: Option<&str>,
        max_keys: Option<u32>,
    ) -> Result<Vec<Object>>;
}
