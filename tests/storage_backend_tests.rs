//! Tests for the storage backend implementations
//! 
//! This module tests the storage traits and their implementations
//! to ensure proper S3-compatible behavior.

use object_io_storage::*;
use object_io_core::*;
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::io::AsyncReadExt;
use chrono::Utc;

#[cfg(test)]
mod filesystem_storage_tests {
    use super::*;

    async fn setup_test_storage() -> (FileSystemStorage, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage = FileSystemStorage::new(temp_dir.path().to_path_buf())
            .await
            .expect("Failed to create filesystem storage");
        (storage, temp_dir)
    }

    #[tokio::test]
    async fn test_put_and_get_object() {
        let (storage, _temp_dir) = setup_test_storage().await;
        
        let bucket = "test-bucket";
        let key = "test-file.txt";
        let content = b"Hello, ObjectIO!";
        
        // Create bucket first
        storage.create_bucket(bucket, "test-user", "us-east-1")
            .await
            .expect("Failed to create bucket");

        // Put object
        let mut cursor = std::io::Cursor::new(content);
        let result = storage.put_object(bucket, key, &mut cursor, content.len() as u64, "text/plain")
            .await
            .expect("Failed to put object");

        assert_eq!(result.key, key);
        assert_eq!(result.bucket, bucket);
        assert_eq!(result.size, content.len() as u64);

        // Get object back
        let mut reader = storage.get_object(bucket, key)
            .await
            .expect("Failed to get object");

        let mut retrieved_content = Vec::new();
        reader.read_to_end(&mut retrieved_content)
            .await
            .expect("Failed to read object content");

        assert_eq!(retrieved_content, content);
    }

    #[tokio::test]
    async fn test_list_objects() {
        let (storage, _temp_dir) = setup_test_storage().await;
        
        let bucket = "list-test-bucket";
        storage.create_bucket(bucket, "test-user", "us-east-1")
            .await
            .expect("Failed to create bucket");

        // Put multiple objects
        let files = vec![
            ("file1.txt", b"content1"),
            ("path/file2.txt", b"content2"),
            ("path/subpath/file3.txt", b"content3"),
        ];

        for (key, content) in &files {
            let mut cursor = std::io::Cursor::new(*content);
            storage.put_object(bucket, key, &mut cursor, content.len() as u64, "text/plain")
                .await
                .expect("Failed to put object");
        }

        // List all objects
        let objects = storage.list_objects(bucket, None, None, None, 1000)
            .await
            .expect("Failed to list objects");

        assert_eq!(objects.len(), 3);
        
        // Check that all keys are present
        let keys: Vec<&str> = objects.iter().map(|obj| obj.key.as_str()).collect();
        assert!(keys.contains(&"file1.txt"));
        assert!(keys.contains(&"path/file2.txt"));
        assert!(keys.contains(&"path/subpath/file3.txt"));

        // List with prefix
        let prefixed_objects = storage.list_objects(bucket, Some("path/"), None, None, 1000)
            .await
            .expect("Failed to list objects with prefix");

        assert_eq!(prefixed_objects.len(), 2);
        for obj in &prefixed_objects {
            assert!(obj.key.starts_with("path/"));
        }
    }

    #[tokio::test]
    async fn test_delete_object() {
        let (storage, _temp_dir) = setup_test_storage().await;
        
        let bucket = "delete-test-bucket";
        let key = "delete-me.txt";
        let content = b"This will be deleted";

        storage.create_bucket(bucket, "test-user", "us-east-1")
            .await
            .expect("Failed to create bucket");

        // Put object
        let mut cursor = std::io::Cursor::new(content);
        storage.put_object(bucket, key, &mut cursor, content.len() as u64, "text/plain")
            .await
            .expect("Failed to put object");

        // Verify it exists
        let objects = storage.list_objects(bucket, None, None, None, 1000)
            .await
            .expect("Failed to list objects");
        assert_eq!(objects.len(), 1);

        // Delete it
        storage.delete_object(bucket, key)
            .await
            .expect("Failed to delete object");

        // Verify it's gone
        let objects_after = storage.list_objects(bucket, None, None, None, 1000)
            .await
            .expect("Failed to list objects after deletion");
        assert_eq!(objects_after.len(), 0);

        // Try to get deleted object (should fail)
        let get_result = storage.get_object(bucket, key).await;
        assert!(get_result.is_err());
    }

    #[tokio::test]
    async fn test_bucket_operations() {
        let (storage, _temp_dir) = setup_test_storage().await;
        
        let bucket_name = "bucket-ops-test";

        // Create bucket
        let bucket = storage.create_bucket(bucket_name, "test-user", "us-east-1")
            .await
            .expect("Failed to create bucket");

        assert_eq!(bucket.name, bucket_name);
        assert_eq!(bucket.owner, "test-user");
        assert_eq!(bucket.region, "us-east-1");

        // List buckets
        let buckets = storage.list_buckets()
            .await
            .expect("Failed to list buckets");

        assert_eq!(buckets.len(), 1);
        assert_eq!(buckets[0].name, bucket_name);

        // Get bucket info
        let bucket_info = storage.get_bucket(bucket_name)
            .await
            .expect("Failed to get bucket info");

        assert_eq!(bucket_info.name, bucket_name);

        // Delete bucket
        storage.delete_bucket(bucket_name)
            .await
            .expect("Failed to delete bucket");

        // Verify bucket is gone
        let buckets_after = storage.list_buckets()
            .await
            .expect("Failed to list buckets after deletion");
        assert_eq!(buckets_after.len(), 0);
    }

    #[tokio::test]
    async fn test_head_object() {
        let (storage, _temp_dir) = setup_test_storage().await;
        
        let bucket = "head-test-bucket";
        let key = "test-file.pdf";
        let content = b"PDF content here";

        storage.create_bucket(bucket, "test-user", "us-east-1")
            .await
            .expect("Failed to create bucket");

        // Put object with specific content type
        let mut cursor = std::io::Cursor::new(content);
        storage.put_object(bucket, key, &mut cursor, content.len() as u64, "application/pdf")
            .await
            .expect("Failed to put object");

        // Head object
        let object_info = storage.head_object(bucket, key)
            .await
            .expect("Failed to head object");

        assert_eq!(object_info.key, key);
        assert_eq!(object_info.bucket, bucket);
        assert_eq!(object_info.size, content.len() as u64);
        assert_eq!(object_info.content_type, "application/pdf");
        assert!(!object_info.etag.is_empty());
    }

    #[tokio::test]
    async fn test_error_conditions() {
        let (storage, _temp_dir) = setup_test_storage().await;

        // Try to get object from non-existent bucket
        let result = storage.get_object("non-existent-bucket", "some-key").await;
        assert!(result.is_err());

        // Try to get non-existent object from existent bucket
        storage.create_bucket("test-bucket", "user", "region")
            .await
            .expect("Failed to create bucket");

        let result = storage.get_object("test-bucket", "non-existent-key").await;
        assert!(result.is_err());

        // Try to delete non-existent object
        let result = storage.delete_object("test-bucket", "non-existent-key").await;
        assert!(result.is_err());

        // Try to create bucket with invalid name
        let result = storage.create_bucket("Invalid_Bucket_Name", "user", "region").await;
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod storage_trait_tests {
    use super::*;
    use async_trait::async_trait;
    use std::io::Cursor;

    // Mock storage implementation for testing the trait
    struct MockStorage {
        buckets: std::sync::Mutex<HashMap<String, Bucket>>,
        objects: std::sync::Mutex<HashMap<(String, String), (ObjectInfo, Vec<u8>)>>,
    }

    impl MockStorage {
        fn new() -> Self {
            Self {
                buckets: std::sync::Mutex::new(HashMap::new()),
                objects: std::sync::Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl StorageBackend for MockStorage {
        async fn put_object(
            &self,
            bucket: &str,
            key: &str,
            reader: &mut dyn AsyncRead,
            size: u64,
            content_type: &str,
        ) -> Result<ObjectInfo> {
            // Read all data
            let mut buffer = Vec::new();
            let mut temp_reader = reader;
            temp_reader.read_to_end(&mut buffer).await.map_err(ObjectIOError::from)?;

            if buffer.len() != size as usize {
                return Err(ObjectIOError::InvalidRequest {
                    message: "Size mismatch".to_string(),
                });
            }

            let object_info = ObjectInfo {
                key: key.to_string(),
                bucket: bucket.to_string(),
                size,
                etag: generate_etag(&buffer),
                content_type: content_type.to_string(),
                created_at: Utc::now(),
                last_modified: Utc::now(),
                owner: "mock-user".to_string(),
                metadata: HashMap::new(),
                storage_class: StorageClass::Standard,
            };

            self.objects.lock().unwrap().insert(
                (bucket.to_string(), key.to_string()),
                (object_info.clone(), buffer),
            );

            Ok(object_info)
        }

        async fn get_object(&self, bucket: &str, key: &str) -> Result<Box<dyn AsyncRead + Send + Unpin>> {
            let objects = self.objects.lock().unwrap();
            let data = objects.get(&(bucket.to_string(), key.to_string()))
                .ok_or_else(|| ObjectIOError::ObjectNotFound {
                    key: key.to_string(),
                    bucket: bucket.to_string(),
                })?;

            let cursor = Cursor::new(data.1.clone());
            Ok(Box::new(cursor))
        }

        async fn delete_object(&self, bucket: &str, key: &str) -> Result<()> {
            let mut objects = self.objects.lock().unwrap();
            objects.remove(&(bucket.to_string(), key.to_string()))
                .ok_or_else(|| ObjectIOError::ObjectNotFound {
                    key: key.to_string(),
                    bucket: bucket.to_string(),
                })?;
            Ok(())
        }

        async fn head_object(&self, bucket: &str, key: &str) -> Result<ObjectInfo> {
            let objects = self.objects.lock().unwrap();
            let object_info = objects.get(&(bucket.to_string(), key.to_string()))
                .ok_or_else(|| ObjectIOError::ObjectNotFound {
                    key: key.to_string(),
                    bucket: bucket.to_string(),
                })?;
            Ok(object_info.0.clone())
        }

        async fn list_objects(
            &self,
            bucket: &str,
            prefix: Option<&str>,
            delimiter: Option<&str>,
            marker: Option<&str>,
            max_keys: i32,
        ) -> Result<Vec<ObjectInfo>> {
            let objects = self.objects.lock().unwrap();
            let mut result: Vec<ObjectInfo> = objects
                .iter()
                .filter(|((b, k), _)| {
                    b == bucket &&
                    prefix.map_or(true, |p| k.starts_with(p)) &&
                    marker.map_or(true, |m| k > m)
                })
                .map(|(_, (info, _))| info.clone())
                .collect();

            result.sort_by(|a, b| a.key.cmp(&b.key));
            result.truncate(max_keys as usize);
            Ok(result)
        }

        async fn create_bucket(&self, name: &str, owner: &str, region: &str) -> Result<Bucket> {
            validate_bucket_name(name)?;

            let bucket = Bucket {
                name: name.to_string(),
                created_at: Utc::now(),
                owner: owner.to_string(),
                region: region.to_string(),
                versioning_enabled: false,
                public_read: false,
                tags: HashMap::new(),
            };

            self.buckets.lock().unwrap().insert(name.to_string(), bucket.clone());
            Ok(bucket)
        }

        async fn delete_bucket(&self, name: &str) -> Result<()> {
            let mut buckets = self.buckets.lock().unwrap();
            buckets.remove(name)
                .ok_or_else(|| ObjectIOError::BucketNotFound {
                    bucket: name.to_string(),
                })?;
            Ok(())
        }

        async fn list_buckets(&self) -> Result<Vec<Bucket>> {
            let buckets = self.buckets.lock().unwrap();
            Ok(buckets.values().cloned().collect())
        }

        async fn get_bucket(&self, name: &str) -> Result<Bucket> {
            let buckets = self.buckets.lock().unwrap();
            buckets.get(name)
                .cloned()
                .ok_or_else(|| ObjectIOError::BucketNotFound {
                    bucket: name.to_string(),
                })
        }
    }

    #[tokio::test]
    async fn test_mock_storage_full_workflow() {
        let storage = MockStorage::new();

        // Create bucket
        let bucket = storage.create_bucket("test-bucket", "test-user", "us-east-1")
            .await
            .expect("Failed to create bucket");

        assert_eq!(bucket.name, "test-bucket");

        // Put object
        let content = b"Mock storage test content";
        let mut cursor = Cursor::new(content);
        let object_info = storage.put_object("test-bucket", "test.txt", &mut cursor, content.len() as u64, "text/plain")
            .await
            .expect("Failed to put object");

        assert_eq!(object_info.size, content.len() as u64);
        assert_eq!(object_info.content_type, "text/plain");

        // Head object
        let head_info = storage.head_object("test-bucket", "test.txt")
            .await
            .expect("Failed to head object");

        assert_eq!(head_info.key, object_info.key);
        assert_eq!(head_info.etag, object_info.etag);

        // Get object
        let mut reader = storage.get_object("test-bucket", "test.txt")
            .await
            .expect("Failed to get object");

        let mut retrieved = Vec::new();
        reader.read_to_end(&mut retrieved).await.expect("Failed to read");
        assert_eq!(retrieved, content);

        // List objects
        let objects = storage.list_objects("test-bucket", None, None, None, 1000)
            .await
            .expect("Failed to list objects");

        assert_eq!(objects.len(), 1);
        assert_eq!(objects[0].key, "test.txt");

        // Delete object
        storage.delete_object("test-bucket", "test.txt")
            .await
            .expect("Failed to delete object");

        let objects_after = storage.list_objects("test-bucket", None, None, None, 1000)
            .await
            .expect("Failed to list objects after deletion");
        assert_eq!(objects_after.len(), 0);
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[tokio::test]
    async fn test_large_object_handling() {
        let (storage, _temp_dir) = setup_test_storage().await;
        
        let bucket = "perf-test-bucket";
        storage.create_bucket(bucket, "test-user", "us-east-1")
            .await
            .expect("Failed to create bucket");

        // Create a 1MB object
        let size = 1024 * 1024; // 1MB
        let content = vec![0u8; size];
        let mut cursor = std::io::Cursor::new(&content);

        let start = std::time::Instant::now();
        
        let object_info = storage.put_object(bucket, "large-file.bin", &mut cursor, size as u64, "application/octet-stream")
            .await
            .expect("Failed to put large object");

        let put_duration = start.elapsed();
        println!("Put 1MB object in: {:?}", put_duration);

        assert_eq!(object_info.size, size as u64);

        // Time the retrieval
        let start = std::time::Instant::now();
        
        let mut reader = storage.get_object(bucket, "large-file.bin")
            .await
            .expect("Failed to get large object");

        let mut retrieved = Vec::new();
        reader.read_to_end(&mut retrieved)
            .await
            .expect("Failed to read large object");

        let get_duration = start.elapsed();
        println!("Get 1MB object in: {:?}", get_duration);

        assert_eq!(retrieved.len(), size);
        assert_eq!(retrieved, content);
    }

    #[tokio::test]
    async fn test_many_small_objects() {
        let (storage, _temp_dir) = setup_test_storage().await;
        
        let bucket = "many-objects-bucket";
        storage.create_bucket(bucket, "test-user", "us-east-1")
            .await
            .expect("Failed to create bucket");

        let object_count = 100;
        let start = std::time::Instant::now();

        // Create many small objects
        for i in 0..object_count {
            let key = format!("file-{:03}.txt", i);
            let content = format!("Content for file {}", i).into_bytes();
            let mut cursor = std::io::Cursor::new(&content);

            storage.put_object(bucket, &key, &mut cursor, content.len() as u64, "text/plain")
                .await
                .expect("Failed to put object");
        }

        let put_duration = start.elapsed();
        println!("Put {} objects in: {:?}", object_count, put_duration);

        // List all objects
        let start = std::time::Instant::now();
        
        let objects = storage.list_objects(bucket, None, None, None, 1000)
            .await
            .expect("Failed to list objects");

        let list_duration = start.elapsed();
        println!("Listed {} objects in: {:?}", objects.len(), list_duration);

        assert_eq!(objects.len(), object_count);
    }
}
