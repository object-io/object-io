//! Filesystem storage backend implementation

use crate::traits::Storage;
use object_io_core::{Object, ObjectIOError, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};

/// Filesystem-based storage backend
pub struct FilesystemStorage {
    root_path: PathBuf,
}

impl FilesystemStorage {
    /// Create a new filesystem storage backend
    pub async fn new<P: AsRef<Path>>(root_path: P) -> Result<Self> {
        let root_path = root_path.as_ref().to_path_buf();
        
        // Create root directory if it doesn't exist
        if !root_path.exists() {
            fs::create_dir_all(&root_path).await.map_err(|e| {
                ObjectIOError::StorageError {
                    message: format!("Failed to create storage directory: {}", e),
                }
            })?;
        }

        Ok(Self { root_path })
    }

    /// Get the full path for a bucket
    fn bucket_path(&self, bucket: &str) -> PathBuf {
        self.root_path.join(bucket)
    }

    /// Get the full path for an object
    fn object_path(&self, bucket: &str, key: &str) -> PathBuf {
        self.bucket_path(bucket).join(key)
    }

    /// Get the metadata file path for an object
    fn metadata_path(&self, bucket: &str, key: &str) -> PathBuf {
        let object_path = self.object_path(bucket, key);
        object_path.with_extension("meta")
    }
}

#[async_trait::async_trait]
impl Storage for FilesystemStorage {
    async fn put_object(
        &self,
        bucket: &str,
        key: &str,
        mut data: Box<dyn AsyncRead + Send + Unpin>,
        metadata: HashMap<String, String>,
    ) -> Result<String> {
        let object_path = self.object_path(bucket, key);
        let metadata_path = self.metadata_path(bucket, key);

        // Create bucket directory if it doesn't exist
        if let Some(parent) = object_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                ObjectIOError::StorageError {
                    message: format!("Failed to create bucket directory: {}", e),
                }
            })?;
        }

        // Write object data
        let mut file = fs::File::create(&object_path).await.map_err(|e| {
            ObjectIOError::StorageError {
                message: format!("Failed to create object file: {}", e),
            }
        })?;

        let mut buffer = Vec::new();
        data.read_to_end(&mut buffer).await.map_err(|e| {
            ObjectIOError::StorageError {
                message: format!("Failed to read data: {}", e),
            }
        })?;

        file.write_all(&buffer).await.map_err(|e| {
            ObjectIOError::StorageError {
                message: format!("Failed to write object: {}", e),
            }
        })?;

        // Generate ETag
        let etag = object_io_core::utils::generate_etag(&buffer);

        // Write metadata
        let metadata_json = serde_json::to_string(&metadata).map_err(|e| {
            ObjectIOError::StorageError {
                message: format!("Failed to serialize metadata: {}", e),
            }
        })?;

        fs::write(&metadata_path, metadata_json).await.map_err(|e| {
            ObjectIOError::StorageError {
                message: format!("Failed to write metadata: {}", e),
            }
        })?;

        Ok(etag)
    }

    async fn get_object(&self, bucket: &str, key: &str) -> Result<Box<dyn AsyncRead + Send + Unpin>> {
        let object_path = self.object_path(bucket, key);

        if !object_path.exists() {
            return Err(ObjectIOError::ObjectNotFound {
                bucket: bucket.to_string(),
                key: key.to_string(),
            });
        }

        let file = fs::File::open(object_path).await.map_err(|e| {
            ObjectIOError::StorageError {
                message: format!("Failed to open object: {}", e),
            }
        })?;

        Ok(Box::new(file))
    }

    async fn delete_object(&self, bucket: &str, key: &str) -> Result<()> {
        let object_path = self.object_path(bucket, key);
        let metadata_path = self.metadata_path(bucket, key);

        if !object_path.exists() {
            return Err(ObjectIOError::ObjectNotFound {
                bucket: bucket.to_string(),
                key: key.to_string(),
            });
        }

        // Delete object file
        fs::remove_file(&object_path).await.map_err(|e| {
            ObjectIOError::StorageError {
                message: format!("Failed to delete object: {}", e),
            }
        })?;

        // Delete metadata file if it exists
        if metadata_path.exists() {
            fs::remove_file(&metadata_path).await.map_err(|e| {
                ObjectIOError::StorageError {
                    message: format!("Failed to delete metadata: {}", e),
                }
            })?;
        }

        Ok(())
    }

    async fn object_exists(&self, bucket: &str, key: &str) -> Result<bool> {
        let object_path = self.object_path(bucket, key);
        Ok(object_path.exists())
    }

    async fn get_object_metadata(&self, bucket: &str, key: &str) -> Result<HashMap<String, String>> {
        let metadata_path = self.metadata_path(bucket, key);

        if !metadata_path.exists() {
            return Ok(HashMap::new());
        }

        let metadata_content = fs::read_to_string(&metadata_path).await.map_err(|e| {
            ObjectIOError::StorageError {
                message: format!("Failed to read metadata: {}", e),
            }
        })?;

        let metadata: HashMap<String, String> = serde_json::from_str(&metadata_content).map_err(|e| {
            ObjectIOError::StorageError {
                message: format!("Failed to parse metadata: {}", e),
            }
        })?;

        Ok(metadata)
    }

    async fn list_objects(
        &self,
        bucket: &str,
        prefix: Option<&str>,
        _delimiter: Option<&str>,
        max_keys: Option<u32>,
    ) -> Result<Vec<Object>> {
        let bucket_path = self.bucket_path(bucket);

        if !bucket_path.exists() {
            return Ok(Vec::new());
        }

        let mut objects = Vec::new();
        let mut entries = fs::read_dir(&bucket_path).await.map_err(|e| {
            ObjectIOError::StorageError {
                message: format!("Failed to read bucket directory: {}", e),
            }
        })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            ObjectIOError::StorageError {
                message: format!("Failed to read directory entry: {}", e),
            }
        })? {
            let path = entry.path();
            
            // Skip metadata files
            if path.extension().and_then(|s| s.to_str()) == Some("meta") {
                continue;
            }

            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                    // Apply prefix filter
                    if let Some(prefix_str) = prefix {
                        if !file_name.starts_with(prefix_str) {
                            continue;
                        }
                    }

                    // Get file metadata
                    let metadata = entry.metadata().await.map_err(|e| {
                        ObjectIOError::StorageError {
                            message: format!("Failed to read file metadata: {}", e),
                        }
                    })?;

                    // Create object summary
                    let object = Object {
                        key: file_name.to_string(),
                        bucket: bucket.to_string(),
                        size: metadata.len(),
                        etag: "".to_string(), // Would need to read file to generate
                        last_modified: chrono::DateTime::<chrono::Utc>::from(metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH)),
                        content_type: "application/octet-stream".to_string(),
                        content_encoding: None,
                        metadata: HashMap::new(),
                        storage_class: object_io_core::StorageClass::Standard,
                    };

                    objects.push(object);

                    // Apply max_keys limit
                    if let Some(max) = max_keys {
                        if objects.len() >= max as usize {
                            break;
                        }
                    }
                }
            }
        }

        Ok(objects)
    }
}
