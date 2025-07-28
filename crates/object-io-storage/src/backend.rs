//! Storage backend factory and configuration

use crate::traits::Storage;
use crate::filesystem::FilesystemStorage;
use object_io_core::Result;
use std::sync::Arc;

/// Storage backend configuration
#[derive(Debug, Clone)]
pub enum StorageConfig {
    Filesystem {
        root_path: String,
    },
    // Future backends can be added here
    // S3 { endpoint: String, region: String },
    // GCS { project_id: String },
}

/// Storage backend factory
pub struct StorageBackend;

impl StorageBackend {
    /// Create a new storage backend from configuration
    pub async fn new(config: StorageConfig) -> Result<Arc<dyn Storage>> {
        match config {
            StorageConfig::Filesystem { root_path } => {
                let storage = FilesystemStorage::new(root_path).await?;
                Ok(Arc::new(storage))
            }
        }
    }

    /// Create a filesystem storage backend
    pub async fn filesystem(root_path: String) -> Result<Arc<dyn Storage>> {
        let storage = FilesystemStorage::new(root_path).await?;
        Ok(Arc::new(storage))
    }
}
