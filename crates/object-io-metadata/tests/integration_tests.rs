//! Integration tests for metadata operations

use chrono::Utc;
use object_io_metadata::{Database, MetadataOperations};
use std::collections::HashMap;
use tempfile::TempDir;
use tokio;

#[tokio::test]
async fn test_database_connection_and_schema() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_db");
    
    // Test database creation and schema initialization
    let database = Database::new(db_path.to_str().unwrap()).await.unwrap();
    database.init_schema().await.unwrap();
    
    println!("✅ Database connection and schema initialization successful");
}

#[tokio::test]
async fn test_bucket_operations() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_db");
    
    let database = Database::new(db_path.to_str().unwrap()).await.unwrap();
    database.init_schema().await.unwrap();
    let ops = MetadataOperations::new(database);
    
    // Test bucket creation
    let bucket = ops.create_bucket("test-bucket", "testuser").await.unwrap();
    assert_eq!(bucket.name, "test-bucket");
    assert_eq!(bucket.access_control.owner.name, "testuser");
    println!("✅ Bucket creation successful");
    
    // Test bucket retrieval
    let retrieved = ops.get_bucket("test-bucket").await.unwrap();
    assert!(retrieved.is_some());
    let retrieved_bucket = retrieved.unwrap();
    assert_eq!(retrieved_bucket.name, "test-bucket");
    println!("✅ Bucket retrieval successful");
    
    // Test bucket listing
    let buckets = ops.list_buckets("testuser").await.unwrap();
    assert_eq!(buckets.len(), 1);
    assert_eq!(buckets[0].name, "test-bucket");
    println!("✅ Bucket listing successful");
    
    // Test bucket deletion
    ops.delete_bucket("test-bucket").await.unwrap();
    let deleted = ops.get_bucket("test-bucket").await.unwrap();
    assert!(deleted.is_none());
    println!("✅ Bucket deletion successful");
}

#[tokio::test]
async fn test_object_operations() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_db");
    
    let database = Database::new(db_path.to_str().unwrap()).await.unwrap();
    database.init_schema().await.unwrap();
    let ops = MetadataOperations::new(database);
    
    // Create bucket first
    ops.create_bucket("test-bucket", "testuser").await.unwrap();
    
    // Test object metadata storage
    let mut metadata = HashMap::new();
    metadata.insert("custom-key".to_string(), "custom-value".to_string());
    
    let object_info = ops.put_object_metadata(
        "test-bucket",
        "test/object.txt",
        1024,
        "text/plain",
        "abcdef1234567890",
        "/path/to/storage/object.txt",
        metadata.clone(),
    ).await.unwrap();
    
    assert_eq!(object_info.key, "test/object.txt");
    assert_eq!(object_info.size, 1024);
    assert_eq!(object_info.etag, "abcdef1234567890");
    println!("✅ Object metadata storage successful");
    
    // Test object metadata retrieval
    let retrieved = ops.get_object_metadata("test-bucket", "test/object.txt").await.unwrap();
    assert!(retrieved.is_some());
    let retrieved_object = retrieved.unwrap();
    assert_eq!(retrieved_object.key, "test/object.txt");
    assert_eq!(retrieved_object.size, 1024);
    println!("✅ Object metadata retrieval successful");
    
    // Test object listing
    let objects = ops.list_objects("test-bucket", None, None).await.unwrap();
    assert_eq!(objects.len(), 1);
    assert_eq!(objects[0].key, "test/object.txt");
    assert_eq!(objects[0].bucket, "test-bucket");
    println!("✅ Object listing successful");
    
    // Test object listing with prefix
    let objects_with_prefix = ops.list_objects("test-bucket", Some("test/"), None).await.unwrap();
    assert_eq!(objects_with_prefix.len(), 1);
    println!("✅ Object listing with prefix successful");
    
    // Test object deletion
    ops.delete_object("test-bucket", "test/object.txt").await.unwrap();
    let deleted = ops.get_object_metadata("test-bucket", "test/object.txt").await.unwrap();
    assert!(deleted.is_none());
    println!("✅ Object deletion successful");
}

#[tokio::test]
async fn test_multiple_buckets_and_objects() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_db");
    
    let database = Database::new(db_path.to_str().unwrap()).await.unwrap();
    database.init_schema().await.unwrap();
    let ops = MetadataOperations::new(database);
    
    // Create multiple buckets
    ops.create_bucket("bucket1", "user1").await.unwrap();
    ops.create_bucket("bucket2", "user1").await.unwrap();
    ops.create_bucket("bucket3", "user2").await.unwrap();
    
    // Add objects to different buckets
    ops.put_object_metadata(
        "bucket1", "file1.txt", 100, "text/plain", "etag1", "/path1", HashMap::new()
    ).await.unwrap();
    
    ops.put_object_metadata(
        "bucket1", "file2.txt", 200, "text/plain", "etag2", "/path2", HashMap::new()
    ).await.unwrap();
    
    ops.put_object_metadata(
        "bucket2", "file3.txt", 300, "text/plain", "etag3", "/path3", HashMap::new()
    ).await.unwrap();
    
    // Test user-specific bucket listing
    let user1_buckets = ops.list_buckets("user1").await.unwrap();
    assert_eq!(user1_buckets.len(), 2);
    
    let user2_buckets = ops.list_buckets("user2").await.unwrap();
    assert_eq!(user2_buckets.len(), 1);
    
    // Test bucket-specific object listing
    let bucket1_objects = ops.list_objects("bucket1", None, None).await.unwrap();
    assert_eq!(bucket1_objects.len(), 2);
    
    let bucket2_objects = ops.list_objects("bucket2", None, None).await.unwrap();
    assert_eq!(bucket2_objects.len(), 1);
    
    println!("✅ Multiple buckets and objects test successful");
}

#[tokio::test]
async fn test_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_db");
    
    let database = Database::new(db_path.to_str().unwrap()).await.unwrap();
    database.init_schema().await.unwrap();
    let ops = MetadataOperations::new(database);
    
    // Test retrieving non-existent bucket
    let non_existent = ops.get_bucket("non-existent-bucket").await.unwrap();
    assert!(non_existent.is_none());
    
    // Test retrieving non-existent object
    let non_existent_object = ops.get_object_metadata("non-existent-bucket", "non-existent-key").await.unwrap();
    assert!(non_existent_object.is_none());
    
    // Test listing objects in non-existent bucket (should return empty list)
    let objects = ops.list_objects("non-existent-bucket", None, None).await.unwrap();
    assert_eq!(objects.len(), 0);
    
    println!("✅ Error handling tests successful");
}

#[tokio::test]
async fn test_database_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("persistence_test_db");
    
    // First connection - create data
    {
        let database = Database::new(db_path.to_str().unwrap()).await.unwrap();
        database.init_schema().await.unwrap();
        let ops = MetadataOperations::new(database);
        
        ops.create_bucket("persistent-bucket", "testuser").await.unwrap();
        ops.put_object_metadata(
            "persistent-bucket", "persistent-object", 512, "application/octet-stream",
            "persistent-etag", "/persistent/path", HashMap::new()
        ).await.unwrap();
    }
    
    // Small delay to ensure first connection is fully closed
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Second connection - verify persistence
    {
        let database = Database::new(db_path.to_str().unwrap()).await.unwrap();
        let ops = MetadataOperations::new(database);
        
        let bucket = ops.get_bucket("persistent-bucket").await.unwrap();
        assert!(bucket.is_some());
        
        let object = ops.get_object_metadata("persistent-bucket", "persistent-object").await.unwrap();
        assert!(object.is_some());
        let persistent_object = object.unwrap();
        assert_eq!(persistent_object.size, 512);
        assert_eq!(persistent_object.etag, "persistent-etag");
    }
    
    println!("✅ Database persistence test successful");
}

#[tokio::test]
async fn test_schema_enforcement() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_db");
    
    let database = Database::new(db_path.to_str().unwrap()).await.unwrap();
    database.init_schema().await.unwrap();
    
    // Test that schema is properly enforced by creating records
    // that conform to the expected structure
    let connection = database.connection();
    
    // This should work - valid bucket record
    let valid_bucket = serde_json::json!({
        "name": "valid-bucket",
        "created_at": Utc::now().to_rfc3339(),
        "updated_at": Utc::now().to_rfc3339(),
        "owner": "testuser",
        "acl": {}
    });
    
    let result: Vec<serde_json::Value> = connection
        .create("bucket")
        .content(valid_bucket)
        .await
        .unwrap();
    
    assert!(!result.is_empty());
    println!("✅ Schema enforcement test successful");
}
