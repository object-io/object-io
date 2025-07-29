//! Database connection and management

use object_io_core::Result;
use object_io_database::ObjectDB;

/// Database connection wrapper
pub struct Database {
    db: ObjectDB,
}

impl Database {
    /// Create a new database connection
    pub async fn new(path: &str) -> Result<Self> {
        let db = ObjectDB::new(path).await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: e.to_string(),
            })?;

        Ok(Self { db })
    }

    /// Get a reference to the database connection
    pub fn connection(&self) -> &ObjectDB {
        &self.db
    }

    /// Initialize database schema
    pub async fn init_schema(&self) -> Result<()> {
        // With our embedded database, schema initialization is handled automatically
        // when we create buckets, objects, and users. No explicit schema setup needed.
        Ok(())
    }

    /// Flush database to disk
    pub async fn flush(&self) -> Result<()> {
        self.db.flush().await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: e.to_string(),
            })
    }
}
