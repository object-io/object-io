//! Database connection and management

use object_io_core::Result;
use surrealdb::{Surreal, engine::local::{Db, RocksDb}};

/// Database connection wrapper
pub struct Database {
    db: Surreal<Db>,
}

impl Database {
    /// Create a new database connection
    pub async fn new(path: &str) -> Result<Self> {
        let db = Surreal::new::<RocksDb>(path).await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: e.to_string(),
            })?;

        db.use_ns("objectio").use_db("main").await
            .map_err(|e| object_io_core::ObjectIOError::DatabaseError {
                message: format!("Failed to select namespace/database: {}", e),
            })?;

        // Convert to the Db type for compatibility
        let db: Surreal<Db> = db.into();
        Ok(Self { db })
    }

    /// Get a reference to the database connection
    pub fn connection(&self) -> &Surreal<Db> {
        &self.db
    }

    /// Initialize database schema
    pub async fn init_schema(&self) -> Result<()> {
        // Define bucket table
        self.db.query("
            DEFINE TABLE bucket SCHEMAFULL;
            DEFINE FIELD name ON TABLE bucket TYPE string;
            DEFINE FIELD created_at ON TABLE bucket TYPE datetime;
            DEFINE FIELD updated_at ON TABLE bucket TYPE datetime;
            DEFINE FIELD owner ON TABLE bucket TYPE string;
            DEFINE FIELD acl ON TABLE bucket TYPE object;
            DEFINE INDEX bucket_name ON TABLE bucket COLUMNS name UNIQUE;
        ").await.map_err(|e| object_io_core::ObjectIOError::DatabaseError {
            message: format!("Failed to create bucket schema: {}", e),
        })?;

        // Define object table
        self.db.query("
            DEFINE TABLE object SCHEMAFULL;
            DEFINE FIELD key ON TABLE object TYPE string;
            DEFINE FIELD bucket ON TABLE object TYPE string;
            DEFINE FIELD size ON TABLE object TYPE int;
            DEFINE FIELD content_type ON TABLE object TYPE string;
            DEFINE FIELD etag ON TABLE object TYPE string;
            DEFINE FIELD last_modified ON TABLE object TYPE datetime;
            DEFINE FIELD storage_path ON TABLE object TYPE string;
            DEFINE FIELD metadata ON TABLE object TYPE object;
            DEFINE INDEX object_bucket_key ON TABLE object COLUMNS bucket, key UNIQUE;
        ").await.map_err(|e| object_io_core::ObjectIOError::DatabaseError {
            message: format!("Failed to create object schema: {}", e),
        })?;

        // Define user table
        self.db.query("
            DEFINE TABLE user SCHEMAFULL;
            DEFINE FIELD access_key ON TABLE user TYPE string;
            DEFINE FIELD secret_key ON TABLE user TYPE string;
            DEFINE FIELD created_at ON TABLE user TYPE datetime;
            DEFINE FIELD is_admin ON TABLE user TYPE bool;
            DEFINE FIELD permissions ON TABLE user TYPE array;
            DEFINE INDEX user_access_key ON TABLE user COLUMNS access_key UNIQUE;
        ").await.map_err(|e| object_io_core::ObjectIOError::DatabaseError {
            message: format!("Failed to create user schema: {}", e),
        })?;

        Ok(())
    }
}
