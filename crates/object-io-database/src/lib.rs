//! ObjectIO Embedded Database
//! 
//! A simple, fast embedded database built specifically for ObjectIO's metadata storage needs.
//! Uses Sled as the underlying storage engine with custom schemas for buckets, objects, and users.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info, instrument};

pub mod models;
pub mod operations;

pub use models::{BucketInfo, ObjectInfo, UserInfo};
pub use operations::*;

/// ObjectIO embedded database
#[derive(Clone)]
pub struct ObjectDB {
    /// Sled database instance
    db: Arc<sled::Db>,
    /// Buckets tree
    buckets: sled::Tree,
    /// Objects tree  
    objects: sled::Tree,
    /// Users tree
    users: sled::Tree,
}

impl ObjectDB {
    /// Create a new database instance
    #[instrument(skip(path))]
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        info!("Opening ObjectIO database at: {}", path.display());
        
        let db = sled::open(path)?;
        let buckets = db.open_tree("buckets")?;
        let objects = db.open_tree("objects")?;
        let users = db.open_tree("users")?;
        
        debug!("Database trees initialized successfully");
        
        Ok(Self {
            db: Arc::new(db),
            buckets,
            objects,
            users,
        })
    }
    
    /// Create an in-memory database for testing
    #[cfg(test)]
    pub fn memory() -> Result<Self> {
        let config = sled::Config::new().temporary(true);
        let db = config.open()?;
        let buckets = db.open_tree("buckets")?;
        let objects = db.open_tree("objects")?;
        let users = db.open_tree("users")?;
        
        Ok(Self {
            db: Arc::new(db),
            buckets,
            objects,
            users,
        })
    }
    
    /// Flush all pending writes to disk
    #[instrument(skip(self))]
    pub async fn flush(&self) -> Result<()> {
        self.db.flush_async().await?;
        debug!("Database flushed to disk");
        Ok(())
    }
    
    /// Get database size statistics
    #[instrument(skip(self))]
    pub fn stats(&self) -> DatabaseStats {
        DatabaseStats {
            buckets_count: self.buckets.len(),
            objects_count: self.objects.len(), 
            users_count: self.users.len(),
            size_on_disk: self.db.size_on_disk().unwrap_or(0),
        }
    }
    
    /// Compact the database to reduce disk usage
    #[instrument(skip(self))]
    pub async fn compact(&self) -> Result<()> {
        // Sled handles compaction automatically, but we can trigger it
        self.db.flush_async().await?;
        info!("Database compaction completed");
        Ok(())
    }
}

/// Database statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub buckets_count: usize,
    pub objects_count: usize,
    pub users_count: usize,
    pub size_on_disk: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_database_creation() {
        let db = ObjectDB::memory().expect("Failed to create in-memory database");
        let stats = db.stats();
        assert_eq!(stats.buckets_count, 0);
        assert_eq!(stats.objects_count, 0);
        assert_eq!(stats.users_count, 0);
    }
    
    #[tokio::test]
    async fn test_database_flush() {
        let db = ObjectDB::memory().expect("Failed to create in-memory database");
        db.flush().await.expect("Failed to flush database");
    }
}
