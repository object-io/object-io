//! Utility functions for ObjectIO

use crate::error::{ObjectIOError, Result};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Validate S3 bucket name according to AWS naming rules
pub fn validate_bucket_name(name: &str) -> Result<()> {
    // Basic validation - can be expanded
    if name.is_empty() || name.len() > 63 {
        return Err(ObjectIOError::InvalidBucketName {
            bucket: name.to_string(),
        });
    }

    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.') {
        return Err(ObjectIOError::InvalidBucketName {
            bucket: name.to_string(),
        });
    }

    if name.starts_with('-') || name.ends_with('-') || name.starts_with('.') || name.ends_with('.') {
        return Err(ObjectIOError::InvalidBucketName {
            bucket: name.to_string(),
        });
    }

    Ok(())
}

/// Validate S3 object key
pub fn validate_object_key(key: &str) -> Result<()> {
    if key.is_empty() || key.len() > 1024 {
        return Err(ObjectIOError::InvalidObjectKey {
            key: key.to_string(),
        });
    }

    // Check for invalid characters (simplified)
    if key.contains('\0') {
        return Err(ObjectIOError::InvalidObjectKey {
            key: key.to_string(),
        });
    }

    Ok(())
}

/// Generate ETag for content
pub fn generate_etag(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Parse query parameters from URL
pub fn parse_query_params(query: &str) -> HashMap<String, String> {
    query
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.split('=');
            match (parts.next(), parts.next()) {
                (Some(key), Some(value)) => Some((
                    urlencoding::decode(key).unwrap_or_default().to_string(),
                    urlencoding::decode(value).unwrap_or_default().to_string(),
                )),
                _ => None,
            }
        })
        .collect()
}

/// Format timestamp for S3 responses
pub fn format_s3_timestamp(timestamp: &chrono::DateTime<chrono::Utc>) -> String {
    timestamp.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

/// Parse content range header
pub fn parse_content_range(range: &str) -> Option<(u64, Option<u64>)> {
    if !range.starts_with("bytes=") {
        return None;
    }

    let range = &range[6..]; // Remove "bytes="
    let mut parts = range.split('-');
    
    match (parts.next(), parts.next()) {
        (Some(start), Some(end)) => {
            let start = start.parse().ok()?;
            let end = if end.is_empty() {
                None
            } else {
                Some(end.parse().ok()?)
            };
            Some((start, end))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_bucket_name() {
        assert!(validate_bucket_name("valid-bucket-name").is_ok());
        assert!(validate_bucket_name("test123").is_ok());
        
        assert!(validate_bucket_name("").is_err());
        assert!(validate_bucket_name("InvalidName").is_err());
        assert!(validate_bucket_name("-invalid").is_err());
        assert!(validate_bucket_name("invalid-").is_err());
    }

    #[test]
    fn test_validate_object_key() {
        assert!(validate_object_key("valid/object/key.txt").is_ok());
        assert!(validate_object_key("test-file").is_ok());
        
        assert!(validate_object_key("").is_err());
        assert!(validate_object_key("invalid\0key").is_err());
    }

    #[test]
    fn test_generate_etag() {
        let content = b"test content";
        let etag = generate_etag(content);
        assert!(!etag.is_empty());
        assert_eq!(etag.len(), 64); // SHA256 hex length
    }

    #[test]
    fn test_parse_content_range() {
        assert_eq!(parse_content_range("bytes=0-499"), Some((0, Some(499))));
        assert_eq!(parse_content_range("bytes=500-"), Some((500, None)));
        assert_eq!(parse_content_range("invalid"), None);
    }
}
