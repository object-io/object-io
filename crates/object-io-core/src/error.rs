//! Error types for ObjectIO

use thiserror::Error;

/// Result type alias for ObjectIO operations
pub type Result<T> = std::result::Result<T, ObjectIOError>;

/// Main error type for ObjectIO operations
#[derive(Debug, Error)]
pub enum ObjectIOError {
    #[error("Bucket not found: {bucket}")]
    BucketNotFound { bucket: String },

    #[error("Object not found: {key} in bucket {bucket}")]
    ObjectNotFound { bucket: String, key: String },

    #[error("Bucket already exists: {bucket}")]
    BucketAlreadyExists { bucket: String },

    #[error("Invalid bucket name: {bucket}")]
    InvalidBucketName { bucket: String },

    #[error("Invalid object key: {key}")]
    InvalidObjectKey { key: String },

    #[error("Authentication failed: {reason}")]
    AuthenticationFailed { reason: String },

    #[error("Authorization failed: {reason}")]
    AuthorizationFailed { reason: String },

    #[error("Storage error: {message}")]
    StorageError { message: String },

    #[error("Database error: {message}")]
    DatabaseError { message: String },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("Invalid request: {message}")]
    InvalidRequest { message: String },

    #[error("Internal server error: {message}")]
    InternalError { message: String },

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl ObjectIOError {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            ObjectIOError::BucketNotFound { .. } => 404,
            ObjectIOError::ObjectNotFound { .. } => 404,
            ObjectIOError::BucketAlreadyExists { .. } => 409,
            ObjectIOError::InvalidBucketName { .. } => 400,
            ObjectIOError::InvalidObjectKey { .. } => 400,
            ObjectIOError::AuthenticationFailed { .. } => 401,
            ObjectIOError::AuthorizationFailed { .. } => 403,
            ObjectIOError::InvalidRequest { .. } => 400,
            ObjectIOError::StorageError { .. } => 500,
            ObjectIOError::DatabaseError { .. } => 500,
            ObjectIOError::ConfigurationError { .. } => 500,
            ObjectIOError::InternalError { .. } => 500,
            ObjectIOError::IO(_) => 500,
            ObjectIOError::Serialization(_) => 400,
            ObjectIOError::Other(_) => 500,
        }
    }

    /// Get the S3 error code for this error
    pub fn s3_error_code(&self) -> &'static str {
        match self {
            ObjectIOError::BucketNotFound { .. } => "NoSuchBucket",
            ObjectIOError::ObjectNotFound { .. } => "NoSuchKey",
            ObjectIOError::BucketAlreadyExists { .. } => "BucketAlreadyExists",
            ObjectIOError::InvalidBucketName { .. } => "InvalidBucketName",
            ObjectIOError::InvalidObjectKey { .. } => "InvalidKey",
            ObjectIOError::AuthenticationFailed { .. } => "InvalidAccessKeyId",
            ObjectIOError::AuthorizationFailed { .. } => "AccessDenied",
            ObjectIOError::InvalidRequest { .. } => "InvalidRequest",
            _ => "InternalError",
        }
    }
}
