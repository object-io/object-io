//! Additional integration tests for ObjectIO core functionality
//! 
//! These tests validate the complete workflow of bucket and object operations
//! using the core library components.

#[cfg(test)]
mod integration_tests {
    use crate::*;
    use std::collections::HashMap;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_bucket_creation_and_validation() {
        let user = User {
            id: Uuid::new_v4(),
            name: "test-user".to_string(),
            email: "test@example.com".to_string(),
            access_keys: vec![],
            created_at: Utc::now(),
        };

        let access_control = AccessControl {
            owner: user.clone(),
            acl: vec![],
            policy: None,
        };

        let bucket = Bucket {
            name: "integration-test-bucket".to_string(),
            created_at: Utc::now(),
            region: "us-east-1".to_string(),
            versioning: VersioningStatus::Enabled,
            access_control,
        };

        // Validate the bucket name
        assert!(validate_bucket_name(&bucket.name).is_ok());
        assert_eq!(bucket.name, "integration-test-bucket");
        assert_eq!(bucket.access_control.owner.name, "test-user");
        assert_eq!(bucket.region, "us-east-1");
        assert!(matches!(bucket.versioning, VersioningStatus::Enabled));
    }

    #[test]
    fn test_object_info_creation_and_validation() {
        let object_info = ObjectInfo {
            key: "api/v1/data.json".to_string(),
            size: 524288, // 512KB
            etag: "e4d909c290d0fb1ca068ffaddf22cbd0".to_string(),
            last_modified: Utc::now(),
            storage_class: "STANDARD".to_string(),
        };

        // Validate object key
        assert!(validate_object_key(&object_info.key).is_ok());

        // Verify object properties
        assert_eq!(object_info.key, "api/v1/data.json");
        assert_eq!(object_info.size, 524288);
        assert_eq!(object_info.storage_class, "STANDARD");
    }

    #[test]
    fn test_full_object_creation() {
        let mut metadata = HashMap::new();
        metadata.insert("content-encoding".to_string(), "gzip".to_string());
        metadata.insert("cache-control".to_string(), "max-age=3600".to_string());

        let object = Object {
            key: "documents/report.pdf".to_string(),
            bucket: "api-data-bucket".to_string(),
            size: 1048576,
            etag: "abcdef123456".to_string(),
            last_modified: Utc::now(),
            content_type: "application/pdf".to_string(),
            content_encoding: Some("gzip".to_string()),
            metadata,
            storage_class: StorageClass::Standard,
        };

        // Validate bucket and object key
        assert!(validate_bucket_name(&object.bucket).is_ok());
        assert!(validate_object_key(&object.key).is_ok());

        // Verify object properties
        assert_eq!(object.key, "documents/report.pdf");
        assert_eq!(object.bucket, "api-data-bucket");
        assert_eq!(object.size, 1048576);
        assert_eq!(object.content_type, "application/pdf");
        assert_eq!(object.metadata.len(), 2);
    }

    #[test]
    fn test_comprehensive_validation_rules() {
        // Test valid bucket names
        let valid_bucket_names = vec![
            "my-bucket",
            "test123",
            "a.b.c",
            "abc",
            "bucket-with-numbers-123",
        ];

        for name in valid_bucket_names {
            assert!(validate_bucket_name(name).is_ok(), "Failed for valid name: {}", name);
        }

        // Test invalid bucket names
        let invalid_bucket_names = vec![
            "", // empty
            "Invalid_Bucket", // uppercase and underscore
            "-bucket", // starts with hyphen
            "bucket.", // ends with period
        ];

        for name in invalid_bucket_names {
            assert!(validate_bucket_name(name).is_err(), "Should fail for invalid name: {}", name);
        }

        // Test valid object keys
        let valid_object_keys = vec![
            "file.txt",
            "path/to/file.pdf",
            "documents/reports/2024/annual.docx",
            "unicode-文件.txt",
        ];

        for key in valid_object_keys {
            assert!(validate_object_key(key).is_ok(), "Failed for valid key: {}", key);
        }

        // Test invalid object keys
        let invalid_object_keys = vec![
            "", // empty
        ];

        for key in invalid_object_keys {
            assert!(validate_object_key(key).is_err(), "Should fail for invalid key: {}", key);
        }
    }

    #[test]
    fn test_error_handling_workflow() {
        // Test error creation and matching
        let bucket_error = ObjectIOError::BucketNotFound {
            bucket: "missing-bucket".to_string(),
        };

        let object_error = ObjectIOError::ObjectNotFound {
            key: "missing-file.txt".to_string(),
            bucket: "existing-bucket".to_string(),
        };

        // Test error display
        let bucket_error_msg = bucket_error.to_string();
        assert!(bucket_error_msg.contains("missing-bucket"));

        let object_error_msg = object_error.to_string();
        assert!(object_error_msg.contains("missing-file.txt"));
        assert!(object_error_msg.contains("existing-bucket"));

        // Test Result type usage
        fn might_fail(should_fail: bool) -> crate::Result<String> {
            if should_fail {
                Err(ObjectIOError::InvalidRequest {
                    message: "Test error".to_string(),
                })
            } else {
                Ok("Success".to_string())
            }
        }

        // Test success case
        let success = might_fail(false);
        assert!(success.is_ok());
        assert_eq!(success.unwrap(), "Success");

        // Test error case
        let failure = might_fail(true);
        assert!(failure.is_err());
        match failure.unwrap_err() {
            ObjectIOError::InvalidRequest { message } => {
                assert_eq!(message, "Test error");
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_versioning_status() {
        // Test versioning status variants
        let enabled = VersioningStatus::Enabled;

        // Test default
        assert!(matches!(VersioningStatus::default(), VersioningStatus::Unversioned));

        // Test serialization/deserialization
        let json_enabled = serde_json::to_string(&enabled).unwrap();
        let deserialized_enabled: VersioningStatus = serde_json::from_str(&json_enabled).unwrap();
        assert!(matches!(deserialized_enabled, VersioningStatus::Enabled));
    }

    #[test]
    fn test_etag_generation() {
        // Test ETag generation consistency
        let content1 = b"Hello, ObjectIO!";
        let content2 = b"Different content";
        let content1_again = b"Hello, ObjectIO!";

        let etag1 = generate_etag(content1);
        let etag2 = generate_etag(content2);
        let etag1_again = generate_etag(content1_again);

        // Same content should produce same ETag
        assert_eq!(etag1, etag1_again);

        // Different content should produce different ETags
        assert_ne!(etag1, etag2);

        // ETags should be 64 characters (SHA256 hex)
        assert_eq!(etag1.len(), 64);
        assert_eq!(etag2.len(), 64);

        // ETags should be valid hex
        assert!(etag1.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(etag2.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_content_range_parsing() {
        // Test various content range formats
        assert_eq!(parse_content_range("bytes=0-499"), Some((0, Some(499))));
        assert_eq!(parse_content_range("bytes=500-999"), Some((500, Some(999))));
        assert_eq!(parse_content_range("bytes=500-"), Some((500, None)));
        assert_eq!(parse_content_range("bytes=0-0"), Some((0, Some(0))));

        // Test invalid formats
        assert_eq!(parse_content_range("invalid"), None);
        assert_eq!(parse_content_range("bytes="), None);
        assert_eq!(parse_content_range("bytes=abc-def"), None);
        assert_eq!(parse_content_range(""), None);

        // Test edge cases
        assert_eq!(parse_content_range("bytes=1000-500"), Some((1000, Some(500)))); // Invalid range but parsed
    }

    #[test]
    fn test_serialization_roundtrip() {
        // Test User serialization
        let user = User {
            id: Uuid::new_v4(),
            name: "test-user".to_string(),
            email: "test@example.com".to_string(),
            access_keys: vec![],
            created_at: Utc::now(),
        };

        let user_json = serde_json::to_string(&user).unwrap();
        let restored_user: User = serde_json::from_str(&user_json).unwrap();

        assert_eq!(user.name, restored_user.name);
        assert_eq!(user.email, restored_user.email);
        assert_eq!(user.id, restored_user.id);

        // Test Bucket serialization
        let access_control = AccessControl {
            owner: user,
            acl: vec![],
            policy: None,
        };

        let original_bucket = Bucket {
            name: "serialization-test".to_string(),
            created_at: Utc::now(),
            region: "us-west-2".to_string(),
            versioning: VersioningStatus::Enabled,
            access_control,
        };

        let bucket_json = serde_json::to_string(&original_bucket).unwrap();
        let restored_bucket: Bucket = serde_json::from_str(&bucket_json).unwrap();

        assert_eq!(original_bucket.name, restored_bucket.name);
        assert_eq!(original_bucket.region, restored_bucket.region);
        assert!(matches!(restored_bucket.versioning, VersioningStatus::Enabled));

        // Test ObjectInfo serialization
        let original_object_info = ObjectInfo {
            key: "test/object.txt".to_string(),
            size: 1024,
            etag: "abc123".to_string(),
            last_modified: Utc::now(),
            storage_class: "GLACIER".to_string(),
        };

        let object_json = serde_json::to_string(&original_object_info).unwrap();
        let restored_object_info: ObjectInfo = serde_json::from_str(&object_json).unwrap();

        assert_eq!(original_object_info.key, restored_object_info.key);
        assert_eq!(original_object_info.size, restored_object_info.size);
        assert_eq!(original_object_info.etag, restored_object_info.etag);
        assert_eq!(original_object_info.storage_class, restored_object_info.storage_class);
    }

    #[test]
    fn test_object_summary_creation() {
        let summary = ObjectSummary {
            key: "summary-test.txt".to_string(),
            size: 2048,
            etag: "summary-etag".to_string(),
            last_modified: Utc::now(),
            storage_class: StorageClass::Standard,
        };

        assert_eq!(summary.key, "summary-test.txt");
        assert_eq!(summary.size, 2048);
        assert_eq!(summary.etag, "summary-etag");
    }
}
