//! Application state and configuration

use object_io_metadata::{Database, MetadataOperations};
use object_io_storage::{filesystem::FilesystemStorage, Storage};
use std::sync::Arc;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    /// Database operations
    pub metadata: Arc<MetadataOperations>,
    /// Storage backend
    pub storage: Arc<dyn Storage>,
    /// Server configuration
    pub config: Arc<ServerConfig>,
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Database path
    pub database_path: String,
    /// Storage root path
    pub storage_path: String,
    /// Default region
    pub default_region: String,
    /// Maximum request body size
    pub max_body_size: usize,
    /// Request timeout in seconds
    pub request_timeout: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            database_path: std::env::var("DATABASE_PATH")
                .unwrap_or_else(|_| "./data/objectio.db".to_string()),
            storage_path: std::env::var("STORAGE_PATH")
                .unwrap_or_else(|_| "./data/storage".to_string()),
            default_region: std::env::var("DEFAULT_REGION")
                .unwrap_or_else(|_| "us-east-1".to_string()),
            max_body_size: std::env::var("MAX_BODY_SIZE")
                .unwrap_or_else(|_| "5368709120".to_string()) // 5GB
                .parse()
                .unwrap_or(5 * 1024 * 1024 * 1024),
            request_timeout: std::env::var("REQUEST_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
        }
    }
}

impl AppState {
    /// Create new application state
    pub async fn new() -> object_io_core::Result<Self> {
        let config = Arc::new(ServerConfig::default());
        
        // Ensure storage directory exists
        tokio::fs::create_dir_all(&config.storage_path).await
            .map_err(|e| object_io_core::ObjectIOError::IO(e))?;
        
        // Ensure database directory exists
        if let Some(parent) = std::path::Path::new(&config.database_path).parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| object_io_core::ObjectIOError::IO(e))?;
        }
        
        // Initialize database
        let database = Database::new(&config.database_path).await?;
        database.init_schema().await?;
        
        let metadata = Arc::new(MetadataOperations::new(database));
        
        // Initialize filesystem storage backend
        let storage = Arc::new(FilesystemStorage::new(&config.storage_path).await?) as Arc<dyn Storage>;
        
        Ok(Self {
            metadata,
            storage,
            config,
        })
    }
}
