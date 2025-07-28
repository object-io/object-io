//! Metadata Store Integration Tests
//! 
//! This module tests the SurrealDB-based metadata storage layer
//! for buckets and objects information.

use object_io_metadata::*;
use object_io_core::*;
use std::collections::HashMap;
use tempfile::TempDir;
use chrono::Utc;

#[cfg(test)]
mod database_tests {
    use super::*;

    async fn setup_test_database() -> (Database, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let db_path = temp_dir.path().join("test_metadata.db");
        
        let database = Database::new(&format!("rocksdb://{}", db_path.display()))
            .await
            .expect("Failed to create test database");
            
        (database, temp_dir)
    }

    #[tokio::test]
    async fn test_database_connection() {
        let (_database, _temp_dir) = setup_test_database().await;
        // If we get here without panicking, connection works
    }

    #[tokio::test]
    async fn test_bucket_operations() {
        let (database, _temp_dir) = setup_test_database().await;

        let mut tags = HashMap::new();
        tags.insert("environment".to_string(), "test".to_string());
        tags.insert("project".to_string(), "objectio".to_string());

        let bucket = Bucket {
            name: "test-bucket".to_string(),
            created_at: Utc::now(),
            owner: "test-user".to_string(),
            region: "us-east-1".to_string(),
            versioning_enabled: true,
            public_read: false,
            tags,
        };

        // Create bucket
        let created_bucket = database.create_bucket(&bucket)
            .await
            .expect("Failed to create bucket");

        assert_eq!(created_bucket.name, bucket.name);
        assert_eq!(created_bucket.owner, bucket.owner);
        assert_eq!(created_bucket.tags.len(), 2);

        // Get bucket
        let retrieved_bucket = database.get_bucket(&bucket.name)
            .await
            .expect("Failed to get bucket");

        assert_eq!(retrieved_bucket.name, bucket.name);
        assert_eq!(retrieved_bucket.region, bucket.region);
        assert_eq!(retrieved_bucket.versioning_enabled, bucket.versioning_enabled);

        // List buckets
        let buckets = database.list_buckets()
            .await
            .expect("Failed to list buckets");

        assert_eq!(buckets.len(), 1);
        assert_eq!(buckets[0].name, bucket.name);

        // Update bucket
        let mut updated_bucket = bucket.clone();
        updated_bucket.versioning_enabled = false;
        updated_bucket.public_read = true;

        let result = database.update_bucket(&updated_bucket)
            .await
            .expect("Failed to update bucket");

        assert!(!result.versioning_enabled);
        assert!(result.public_read);

        // Delete bucket
        database.delete_bucket(&bucket.name)
            .await
            .expect("Failed to delete bucket");

        // Verify deletion
        let get_result = database.get_bucket(&bucket.name).await;
        assert!(get_result.is_err());

        let buckets_after = database.list_buckets()
            .await
            .expect("Failed to list buckets after deletion");
        assert_eq!(buckets_after.len(), 0);
    }

    #[tokio::test]
    async fn test_object_operations() {
        let (database, _temp_dir) = setup_test_database().await;

        // First create a bucket
        let bucket = Bucket {
            name: "object-test-bucket".to_string(),
            created_at: Utc::now(),
            owner: "test-user".to_string(),
            region: "us-east-1".to_string(),
            versioning_enabled: false,
            public_read: false,
            tags: HashMap::new(),
        };

        database.create_bucket(&bucket)
            .await
            .expect("Failed to create bucket for object tests");

        // Create object metadata
        let mut metadata = HashMap::new();
        metadata.insert("uploaded-by".to_string(), "user123".to_string());
        metadata.insert("department".to_string(), "engineering".to_string());

        let object_info = ObjectInfo {
            key: "documents/report.pdf".to_string(),
            bucket: bucket.name.clone(),
            size: 1048576,
            etag: "abcdef123456".to_string(),
            content_type: "application/pdf".to_string(),
            created_at: Utc::now(),
            last_modified: Utc::now(),
            owner: "test-user".to_string(),
            metadata,
            storage_class: StorageClass::Standard,
        };

        // Create object
        let created_object = database.create_object(&object_info)
            .await
            .expect("Failed to create object");

        assert_eq!(created_object.key, object_info.key);
        assert_eq!(created_object.size, object_info.size);
        assert_eq!(created_object.metadata.len(), 2);

        // Get object
        let retrieved_object = database.get_object(&bucket.name, &object_info.key)
            .await
            .expect("Failed to get object");

        assert_eq!(retrieved_object.key, object_info.key);
        assert_eq!(retrieved_object.etag, object_info.etag);
        assert_eq!(retrieved_object.content_type, object_info.content_type);

        // List objects
        let objects = database.list_objects(&bucket.name, None, None, 1000)
            .await
            .expect("Failed to list objects");

        assert_eq!(objects.len(), 1);
        assert_eq!(objects[0].key, object_info.key);

        // Update object
        let mut updated_object = object_info.clone();
        updated_object.storage_class = StorageClass::Glacier;
        updated_object.metadata.insert("archive-date".to_string(), "2024-01-01".to_string());

        let result = database.update_object(&updated_object)
            .await
            .expect("Failed to update object");

        assert_eq!(result.storage_class, StorageClass::Glacier);
        assert_eq!(result.metadata.len(), 3);

        // Delete object
        database.delete_object(&bucket.name, &object_info.key)
            .await
            .expect("Failed to delete object");

        // Verify deletion
        let get_result = database.get_object(&bucket.name, &object_info.key).await;
        assert!(get_result.is_err());

        let objects_after = database.list_objects(&bucket.name, None, None, 1000)
            .await
            .expect("Failed to list objects after deletion");
        assert_eq!(objects_after.len(), 0);
    }

    #[tokio::test]
    async fn test_object_listing_with_filters() {
        let (database, _temp_dir) = setup_test_database().await;

        // Create bucket
        let bucket = Bucket {
            name: "filter-test-bucket".to_string(),
            created_at: Utc::now(),
            owner: "test-user".to_string(),
            region: "us-east-1".to_string(),
            versioning_enabled: false,
            public_read: false,
            tags: HashMap::new(),
        };

        database.create_bucket(&bucket)
            .await
            .expect("Failed to create bucket");

        // Create multiple objects with different prefixes
        let objects = vec![
            ("docs/readme.txt", "text/plain"),
            ("docs/guide.md", "text/markdown"),
            ("images/logo.png", "image/png"),
            ("images/banner.jpg", "image/jpeg"),
            ("videos/intro.mp4", "video/mp4"),
            ("archive/old-file.txt", "text/plain"),
        ];

        for (key, content_type) in &objects {
            let object_info = ObjectInfo {
                key: key.to_string(),
                bucket: bucket.name.clone(),
                size: 1024,
                etag: format!("etag-{}", key.replace('/', "-")),
                content_type: content_type.to_string(),
                created_at: Utc::now(),
                last_modified: Utc::now(),
                owner: "test-user".to_string(),
                metadata: HashMap::new(),
                storage_class: StorageClass::Standard,
            };

            database.create_object(&object_info)
                .await
                .expect("Failed to create object");
        }

        // List all objects
        let all_objects = database.list_objects(&bucket.name, None, None, 1000)
            .await
            .expect("Failed to list all objects");
        assert_eq!(all_objects.len(), 6);

        // List with prefix filter
        let docs_objects = database.list_objects(&bucket.name, Some("docs/"), None, 1000)
            .await
            .expect("Failed to list docs objects");
        assert_eq!(docs_objects.len(), 2);
        for obj in &docs_objects {
            assert!(obj.key.starts_with("docs/"));
        }

        // List with prefix and limit
        let limited_objects = database.list_objects(&bucket.name, Some("images/"), None, 1)
            .await
            .expect("Failed to list limited objects");
        assert_eq!(limited_objects.len(), 1);
        assert!(limited_objects[0].key.starts_with("images/"));

        // List with marker (pagination)
        let first_page = database.list_objects(&bucket.name, None, None, 3)
            .await
            .expect("Failed to list first page");
        assert_eq!(first_page.len(), 3);

        // Get the last key from first page as marker
        let marker = &first_page[2].key;
        let second_page = database.list_objects(&bucket.name, None, Some(marker), 3)
            .await
            .expect("Failed to list second page");
        
        // Should get remaining objects (excluding the marker)
        assert!(!second_page.is_empty());
        for obj in &second_page {
            assert!(obj.key > *marker);
        }
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let (database, _temp_dir) = setup_test_database().await;

        // Create a bucket for concurrent tests
        let bucket = Bucket {
            name: "concurrent-test-bucket".to_string(),
            created_at: Utc::now(),
            owner: "test-user".to_string(),
            region: "us-east-1".to_string(),
            versioning_enabled: false,
            public_read: false,
            tags: HashMap::new(),
        };

        database.create_bucket(&bucket)
            .await
            .expect("Failed to create bucket");

        // Create multiple objects concurrently
        let mut tasks = Vec::new();

        for i in 0..10 {
            let db_clone = database.clone();
            let bucket_name = bucket.name.clone();
            
            let task = tokio::spawn(async move {
                let object_info = ObjectInfo {
                    key: format!("concurrent-file-{}.txt", i),
                    bucket: bucket_name,
                    size: 1024 * i,
                    etag: format!("etag-{}", i),
                    content_type: "text/plain".to_string(),
                    created_at: Utc::now(),
                    last_modified: Utc::now(),
                    owner: "concurrent-user".to_string(),
                    metadata: HashMap::new(),
                    storage_class: StorageClass::Standard,
                };

                db_clone.create_object(&object_info).await
            });

            tasks.push(task);
        }

        // Wait for all tasks to complete
        for task in tasks {
            let result = task.await.expect("Task panicked");
            result.expect("Failed to create object concurrently");
        }

        // Verify all objects were created
        let objects = database.list_objects(&bucket.name, None, None, 1000)
            .await
            .expect("Failed to list objects after concurrent creation");

        assert_eq!(objects.len(), 10);

        // Verify each object exists
        for i in 0..10 {
            let key = format!("concurrent-file-{}.txt", i);
            let obj = database.get_object(&bucket.name, &key)
                .await
                .expect("Failed to get concurrent object");
            assert_eq!(obj.key, key);
            assert_eq!(obj.size, 1024 * i);
        }
    }

    #[tokio::test]
    async fn test_error_conditions() {
        let (database, _temp_dir) = setup_test_database().await;

        // Try to get non-existent bucket
        let result = database.get_bucket("non-existent-bucket").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ObjectIOError::BucketNotFound { bucket } => {
                assert_eq!(bucket, "non-existent-bucket");
            }
            _ => panic!("Wrong error type"),
        }

        // Try to delete non-existent bucket
        let result = database.delete_bucket("non-existent-bucket").await;
        assert!(result.is_err());

        // Create a bucket first
        let bucket = Bucket {
            name: "error-test-bucket".to_string(),
            created_at: Utc::now(),
            owner: "test-user".to_string(),
            region: "us-east-1".to_string(),
            versioning_enabled: false,
            public_read: false,
            tags: HashMap::new(),
        };

        database.create_bucket(&bucket)
            .await
            .expect("Failed to create bucket");

        // Try to create duplicate bucket
        let result = database.create_bucket(&bucket).await;
        assert!(result.is_err());

        // Try to get non-existent object
        let result = database.get_object(&bucket.name, "non-existent-object").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ObjectIOError::ObjectNotFound { key, bucket: bucket_name } => {
                assert_eq!(key, "non-existent-object");
                assert_eq!(bucket_name, bucket.name);
            }
            _ => panic!("Wrong error type"),
        }

        // Try to delete non-existent object
        let result = database.delete_object(&bucket.name, "non-existent-object").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_database_persistence() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let db_path = temp_dir.path().join("persistence_test.db");
        let db_url = format!("rocksdb://{}", db_path.display());

        // Create bucket in first database instance
        {
            let database = Database::new(&db_url)
                .await
                .expect("Failed to create database");

            let bucket = Bucket {
                name: "persistent-bucket".to_string(),
                created_at: Utc::now(),
                owner: "test-user".to_string(),
                region: "us-east-1".to_string(),
                versioning_enabled: true,
                public_read: false,
                tags: HashMap::new(),
            };

            database.create_bucket(&bucket)
                .await
                .expect("Failed to create bucket");

            let object_info = ObjectInfo {
                key: "persistent-file.txt".to_string(),
                bucket: bucket.name.clone(),
                size: 2048,
                etag: "persistent-etag".to_string(),
                content_type: "text/plain".to_string(),
                created_at: Utc::now(),
                last_modified: Utc::now(),
                owner: "test-user".to_string(),
                metadata: HashMap::new(),
                storage_class: StorageClass::Standard,
            };

            database.create_object(&object_info)
                .await
                .expect("Failed to create object");
        } // Database instance goes out of scope

        // Create new database instance and verify data persisted
        {
            let database = Database::new(&db_url)
                .await
                .expect("Failed to create second database instance");

            let buckets = database.list_buckets()
                .await
                .expect("Failed to list buckets in second instance");
            assert_eq!(buckets.len(), 1);
            assert_eq!(buckets[0].name, "persistent-bucket");

            let objects = database.list_objects("persistent-bucket", None, None, 1000)
                .await
                .expect("Failed to list objects in second instance");
            assert_eq!(objects.len(), 1);
            assert_eq!(objects[0].key, "persistent-file.txt");
            assert_eq!(objects[0].size, 2048);
        }
    }
}
