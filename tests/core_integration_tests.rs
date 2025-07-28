//! Integration tests for ObjectIO Core
//! 
//! This module contains comprehensive tests for the core types,
//! error handling, and validation functions.

use object_io_core::*;
use std::collections::HashMap;
use chrono::Utc;

#[cfg(test)]
mod bucket_tests {
    use super::*;

    #[test]
    fn test_bucket_creation() {
        let bucket = Bucket {
            name: "test-bucket".to_string(),
            created_at: Utc::now(),
            owner: "test-user".to_string(),
            region: "us-east-1".to_string(),
            versioning_enabled: false,
            public_read: false,
            tags: HashMap::new(),
        };

        assert_eq!(bucket.name, "test-bucket");
        assert_eq!(bucket.owner, "test-user");
        assert_eq!(bucket.region, "us-east-1");
        assert!(!bucket.versioning_enabled);
        assert!(!bucket.public_read);
    }

    #[test]
    fn test_bucket_with_tags() {
        let mut tags = HashMap::new();
        tags.insert("Environment".to_string(), "Production".to_string());
        tags.insert("Team".to_string(), "Backend".to_string());

        let bucket = Bucket {
            name: "prod-bucket".to_string(),
            created_at: Utc::now(),
            owner: "admin".to_string(),
            region: "us-west-2".to_string(),
            versioning_enabled: true,
            public_read: true,
            tags,
        };

        assert_eq!(bucket.tags.len(), 2);
        assert_eq!(bucket.tags.get("Environment"), Some(&"Production".to_string()));
        assert!(bucket.versioning_enabled);
        assert!(bucket.public_read);
    }

    #[test]
    fn test_bucket_serialization() {
        let bucket = Bucket {
            name: "serialize-test".to_string(),
            created_at: Utc::now(),
            owner: "user".to_string(),
            region: "eu-west-1".to_string(),
            versioning_enabled: false,
            public_read: false,
            tags: HashMap::new(),
        };

        let json = serde_json::to_string(&bucket).expect("Failed to serialize bucket");
        let deserialized: Bucket = serde_json::from_str(&json).expect("Failed to deserialize bucket");

        assert_eq!(bucket.name, deserialized.name);
        assert_eq!(bucket.owner, deserialized.owner);
        assert_eq!(bucket.region, deserialized.region);
    }
}

#[cfg(test)]
mod object_info_tests {
    use super::*;

    #[test]
    fn test_object_info_creation() {
        let object = ObjectInfo {
            key: "documents/report.pdf".to_string(),
            bucket: "my-bucket".to_string(),
            size: 1048576, // 1MB
            etag: "d41d8cd98f00b204e9800998ecf8427e".to_string(),
            content_type: "application/pdf".to_string(),
            created_at: Utc::now(),
            last_modified: Utc::now(),
            owner: "user123".to_string(),
            metadata: HashMap::new(),
            storage_class: StorageClass::Standard,
        };

        assert_eq!(object.key, "documents/report.pdf");
        assert_eq!(object.bucket, "my-bucket");
        assert_eq!(object.size, 1048576);
        assert_eq!(object.content_type, "application/pdf");
        assert_eq!(object.storage_class, StorageClass::Standard);
    }

    #[test]
    fn test_object_info_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("uploaded-by".to_string(), "john.doe".to_string());
        metadata.insert("department".to_string(), "finance".to_string());

        let object = ObjectInfo {
            key: "file.txt".to_string(),
            bucket: "data-bucket".to_string(),
            size: 2048,
            etag: "abc123def456".to_string(),
            content_type: "text/plain".to_string(),
            created_at: Utc::now(),
            last_modified: Utc::now(),
            owner: "admin".to_string(),
            metadata,
            storage_class: StorageClass::ReducedRedundancy,
        };

        assert_eq!(object.metadata.len(), 2);
        assert_eq!(object.metadata.get("uploaded-by"), Some(&"john.doe".to_string()));
        assert_eq!(object.storage_class, StorageClass::ReducedRedundancy);
    }

    #[test]
    fn test_storage_class_variants() {
        let standard = StorageClass::Standard;
        let reduced = StorageClass::ReducedRedundancy;
        let glacier = StorageClass::Glacier;
        let deep = StorageClass::DeepArchive;

        // Test default
        assert_eq!(StorageClass::default(), StorageClass::Standard);

        // Test serialization
        let json = serde_json::to_string(&glacier).unwrap();
        let deserialized: StorageClass = serde_json::from_str(&json).unwrap();
        assert_eq!(glacier, deserialized);
    }
}

#[cfg(test)]
mod validation_tests {
    use super::*;

    #[test]
    fn test_valid_bucket_names() {
        let valid_names = vec![
            "my-bucket",
            "test123",
            "a.b.c",
            "abc",
            "bucket-with-numbers-123",
            "a1b2c3",
            "test.bucket.name",
        ];

        for name in valid_names {
            assert!(validate_bucket_name(name).is_ok(), "Failed for: {}", name);
        }
    }

    #[test]
    fn test_invalid_bucket_names() {
        let invalid_cases = vec![
            ("", "empty name"),
            ("ab", "too short"),
            (&"a".repeat(64), "too long"),
            ("My-Bucket", "uppercase letters"),
            ("test_bucket", "underscore"),
            ("test bucket", "space"),
            ("-bucket", "starts with hyphen"),
            ("bucket-", "ends with hyphen"),
            (".bucket", "starts with period"),
            ("bucket.", "ends with period"),
            ("test..bucket", "consecutive periods"),
            ("test--bucket", "consecutive hyphens"),
            ("bucket@example", "at symbol"),
            ("bucket#test", "hash symbol"),
        ];

        for (name, reason) in invalid_cases {
            assert!(validate_bucket_name(name).is_err(), "Should fail for {}: {}", name, reason);
        }
    }

    #[test]
    fn test_valid_object_keys() {
        let valid_keys = vec![
            "file.txt",
            "path/to/file.pdf",
            "documents/reports/2024/annual.docx",
            "a",
            "1234567890",
            "file-with-dashes.txt",
            "file_with_underscores.txt",
            "file with spaces.txt",
            "special-chars!@#$%^&*().txt",
            "unicode-文件.txt",
        ];

        for key in valid_keys {
            assert!(validate_object_key(key).is_ok(), "Failed for: {}", key);
        }
    }

    #[test]
    fn test_invalid_object_keys() {
        let invalid_cases = vec![
            ("", "empty key"),
            (&"a".repeat(1025), "too long"),
            ("/file.txt", "starts with slash"),
            ("//file.txt", "starts with double slash"),
        ];

        for (key, reason) in invalid_cases {
            assert!(validate_object_key(key).is_err(), "Should fail for {}: {}", key, reason);
        }
    }

    #[test]
    fn test_edge_case_validations() {
        // Bucket name edge cases
        assert!(validate_bucket_name("abc").is_ok()); // minimum length
        assert!(validate_bucket_name(&"a".repeat(63)).is_ok()); // maximum length
        
        // Object key edge cases
        assert!(validate_object_key("a").is_ok()); // minimum length
        assert!(validate_object_key(&"a".repeat(1024)).is_ok()); // maximum length
    }
}

#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let bucket_error = ObjectIOError::BucketNotFound {
            bucket: "test-bucket".to_string(),
        };
        
        let object_error = ObjectIOError::ObjectNotFound {
            key: "file.txt".to_string(),
            bucket: "test-bucket".to_string(),
        };

        let storage_error = ObjectIOError::StorageError {
            message: "Disk full".to_string(),
        };

        assert!(bucket_error.to_string().contains("test-bucket"));
        assert!(object_error.to_string().contains("file.txt"));
        assert!(storage_error.to_string().contains("Disk full"));
    }

    #[test]
    fn test_error_result_usage() {
        fn might_fail(should_fail: bool) -> Result<String> {
            if should_fail {
                Err(ObjectIOError::InternalError {
                    message: "Something went wrong".to_string(),
                })
            } else {
                Ok("Success".to_string())
            }
        }

        let success = might_fail(false);
        assert!(success.is_ok());
        assert_eq!(success.unwrap(), "Success");

        let failure = might_fail(true);
        assert!(failure.is_err());
        
        match failure.unwrap_err() {
            ObjectIOError::InternalError { message } => {
                assert_eq!(message, "Something went wrong");
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_io_error_conversion() {
        use std::io::{Error, ErrorKind};
        
        let io_error = Error::new(ErrorKind::NotFound, "File not found");
        let obj_error: ObjectIOError = io_error.into();
        
        match obj_error {
            ObjectIOError::IO(e) => {
                assert_eq!(e.kind(), ErrorKind::NotFound);
            }
            _ => panic!("Wrong error type"),
        }
    }
}

#[cfg(test)]
mod comprehensive_integration_tests {
    use super::*;

    #[test]
    fn test_full_bucket_workflow() {
        // Create a bucket with full metadata
        let mut tags = HashMap::new();
        tags.insert("project".to_string(), "objectio".to_string());
        tags.insert("env".to_string(), "test".to_string());

        let bucket = Bucket {
            name: "integration-test-bucket".to_string(),
            created_at: Utc::now(),
            owner: "integration-user".to_string(),
            region: "us-east-1".to_string(),
            versioning_enabled: true,
            public_read: false,
            tags,
        };

        // Validate the bucket name
        assert!(validate_bucket_name(&bucket.name).is_ok());

        // Serialize and deserialize
        let json = serde_json::to_string(&bucket).unwrap();
        let restored: Bucket = serde_json::from_str(&json).unwrap();

        assert_eq!(bucket.name, restored.name);
        assert_eq!(bucket.tags.len(), restored.tags.len());
    }

    #[test]
    fn test_full_object_workflow() {
        // Create object metadata
        let mut metadata = HashMap::new();
        metadata.insert("content-encoding".to_string(), "gzip".to_string());
        metadata.insert("cache-control".to_string(), "max-age=3600".to_string());

        let object = ObjectInfo {
            key: "api/v1/data.json.gz".to_string(),
            bucket: "api-data-bucket".to_string(),
            size: 524288, // 512KB
            etag: "e4d909c290d0fb1ca068ffaddf22cbd0".to_string(),
            content_type: "application/json".to_string(),
            created_at: Utc::now(),
            last_modified: Utc::now(),
            owner: "api-service".to_string(),
            metadata,
            storage_class: StorageClass::Standard,
        };

        // Validate bucket and object key
        assert!(validate_bucket_name(&object.bucket).is_ok());
        assert!(validate_object_key(&object.key).is_ok());

        // Test serialization round-trip
        let json = serde_json::to_string(&object).unwrap();
        let restored: ObjectInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(object.key, restored.key);
        assert_eq!(object.size, restored.size);
        assert_eq!(object.metadata.len(), restored.metadata.len());
    }

    #[test]
    fn test_error_propagation_chain() {
        // Test how errors propagate through Result chains
        fn validate_and_create_bucket(name: &str) -> Result<Bucket> {
            validate_bucket_name(name)?;
            
            Ok(Bucket {
                name: name.to_string(),
                created_at: Utc::now(),
                owner: "test".to_string(),
                region: "us-east-1".to_string(),
                versioning_enabled: false,
                public_read: false,
                tags: HashMap::new(),
            })
        }

        // Valid case
        let valid_result = validate_and_create_bucket("valid-bucket");
        assert!(valid_result.is_ok());

        // Invalid case
        let invalid_result = validate_and_create_bucket("Invalid_Bucket_Name");
        assert!(invalid_result.is_err());
        
        match invalid_result.unwrap_err() {
            ObjectIOError::InvalidBucketName { bucket } => {
                assert_eq!(bucket, "Invalid_Bucket_Name");
            }
            _ => panic!("Wrong error type"),
        }
    }
}
