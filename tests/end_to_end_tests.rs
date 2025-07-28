//! ObjectIO Comprehensive Test Suite
//! 
//! This module provides end-to-end integration tests that validate
//! the entire S3-compatible storage system from API to storage backend.

use object_io_core::*;
use object_io_storage::FileSystemStorage;
use object_io_metadata::Database;
use object_io_api::create_router;
use object_io_server::ObjectIOServer;

use axum::{
    body::Body,
    http::{Request, StatusCode, Method, header},
};
use tower::ServiceExt;
use tempfile::TempDir;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[cfg(test)]
mod end_to_end_tests {
    use super::*;

    async fn setup_full_stack() -> (ObjectIOServer, TempDir, TempDir) {
        let storage_temp = TempDir::new().expect("Failed to create storage temp dir");
        let metadata_temp = TempDir::new().expect("Failed to create metadata temp dir");
        
        let storage = FileSystemStorage::new(storage_temp.path().to_path_buf())
            .await
            .expect("Failed to create filesystem storage");
            
        let metadata_path = metadata_temp.path().join("test_metadata.db");
        let database = Database::new(&format!("rocksdb://{}", metadata_path.display()))
            .await
            .expect("Failed to create metadata database");

        let server = ObjectIOServer::new(Box::new(storage), Arc::new(database))
            .await
            .expect("Failed to create server");

        (server, storage_temp, metadata_temp)
    }

    #[tokio::test]
    async fn test_complete_s3_compatibility() {
        let (server, _storage_temp, _metadata_temp) = setup_full_stack().await;
        let app = server.router();

        // Test S3 API compliance through a complete workflow

        // 1. List buckets (should be empty initially)
        let list_buckets_request = Request::builder()
            .method(Method::GET)
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let list_response = app.oneshot(list_buckets_request).await.unwrap();
        assert_eq!(list_response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(list_response.into_body()).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("<ListAllMyBucketsResult"));

        // 2. Create a bucket
        let create_bucket_request = Request::builder()
            .method(Method::PUT)
            .uri("/s3-compat-test")
            .header(header::CONTENT_LENGTH, "0")
            .body(Body::empty())
            .unwrap();

        let create_response = app.oneshot(create_bucket_request).await.unwrap();
        assert_eq!(create_response.status(), StatusCode::OK);

        // 3. Verify bucket appears in list
        let list_buckets_request2 = Request::builder()
            .method(Method::GET)
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let list_response2 = app.oneshot(list_buckets_request2).await.unwrap();
        assert_eq!(list_response2.status(), StatusCode::OK);

        let body2 = hyper::body::to_bytes(list_response2.into_body()).await.unwrap();
        let body_str2 = String::from_utf8(body2.to_vec()).unwrap();
        assert!(body_str2.contains("s3-compat-test"));

        // 4. PUT objects with various content types
        let test_files = vec![
            ("text/document.txt", "This is a text document", "text/plain"),
            ("images/photo.jpg", "JPEG_IMAGE_DATA_HERE", "image/jpeg"),
            ("data/config.json", r#"{"setting": "value"}"#, "application/json"),
            ("archives/data.zip", "BINARY_ZIP_DATA", "application/zip"),
            ("videos/clip.mp4", "MP4_VIDEO_DATA", "video/mp4"),
        ];

        for (key, content, content_type) in &test_files {
            let put_request = Request::builder()
                .method(Method::PUT)
                .uri(&format!("/s3-compat-test/{}", key))
                .header(header::CONTENT_TYPE, *content_type)
                .header(header::CONTENT_LENGTH, content.len().to_string())
                .body(Body::from(*content))
                .unwrap();

            let put_response = app.oneshot(put_request).await.unwrap();
            assert_eq!(put_response.status(), StatusCode::OK);

            // Verify ETag header
            let etag = put_response.headers().get("ETag");
            assert!(etag.is_some());
        }

        // 5. List objects in bucket
        let list_objects_request = Request::builder()
            .method(Method::GET)
            .uri("/s3-compat-test")
            .body(Body::empty())
            .unwrap();

        let list_objects_response = app.oneshot(list_objects_request).await.unwrap();
        assert_eq!(list_objects_response.status(), StatusCode::OK);

        let objects_body = hyper::body::to_bytes(list_objects_response.into_body()).await.unwrap();
        let objects_body_str = String::from_utf8(objects_body.to_vec()).unwrap();

        // Verify XML structure and all objects are listed
        assert!(objects_body_str.contains("<ListBucketResult"));
        for (key, _, _) in &test_files {
            assert!(objects_body_str.contains(key));
        }

        // 6. GET specific objects and verify content
        for (key, expected_content, expected_content_type) in &test_files {
            let get_request = Request::builder()
                .method(Method::GET)
                .uri(&format!("/s3-compat-test/{}", key))
                .body(Body::empty())
                .unwrap();

            let get_response = app.oneshot(get_request).await.unwrap();
            assert_eq!(get_response.status(), StatusCode::OK);

            // Check content type
            let content_type = get_response.headers().get(header::CONTENT_TYPE).unwrap();
            assert_eq!(content_type, *expected_content_type);

            // Check content
            let content_body = hyper::body::to_bytes(get_response.into_body()).await.unwrap();
            let content_str = String::from_utf8(content_body.to_vec()).unwrap();
            assert_eq!(content_str, *expected_content);
        }

        // 7. HEAD operations
        for (key, expected_content, expected_content_type) in &test_files {
            let head_request = Request::builder()
                .method(Method::HEAD)
                .uri(&format!("/s3-compat-test/{}", key))
                .body(Body::empty())
                .unwrap();

            let head_response = app.oneshot(head_request).await.unwrap();
            assert_eq!(head_response.status(), StatusCode::OK);

            // Verify headers
            let content_type = head_response.headers().get(header::CONTENT_TYPE).unwrap();
            assert_eq!(content_type, *expected_content_type);

            let content_length = head_response.headers().get(header::CONTENT_LENGTH).unwrap();
            assert_eq!(content_length, expected_content.len().to_string().as_str());

            // HEAD should have no body
            let body = hyper::body::to_bytes(head_response.into_body()).await.unwrap();
            assert_eq!(body.len(), 0);
        }

        // 8. List with prefix filtering
        let prefix_request = Request::builder()
            .method(Method::GET)
            .uri("/s3-compat-test?prefix=images/")
            .body(Body::empty())
            .unwrap();

        let prefix_response = app.oneshot(prefix_request).await.unwrap();
        assert_eq!(prefix_response.status(), StatusCode::OK);

        let prefix_body = hyper::body::to_bytes(prefix_response.into_body()).await.unwrap();
        let prefix_body_str = String::from_utf8(prefix_body.to_vec()).unwrap();

        assert!(prefix_body_str.contains("images/photo.jpg"));
        assert!(!prefix_body_str.contains("text/document.txt"));

        // 9. Delete objects
        for (key, _, _) in &test_files[0..2] { // Delete first two objects
            let delete_request = Request::builder()
                .method(Method::DELETE)
                .uri(&format!("/s3-compat-test/{}", key))
                .body(Body::empty())
                .unwrap();

            let delete_response = app.oneshot(delete_request).await.unwrap();
            assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);
        }

        // 10. Verify objects were deleted
        let final_list_request = Request::builder()
            .method(Method::GET)
            .uri("/s3-compat-test")
            .body(Body::empty())
            .unwrap();

        let final_list_response = app.oneshot(final_list_request).await.unwrap();
        assert_eq!(final_list_response.status(), StatusCode::OK);

        let final_body = hyper::body::to_bytes(final_list_response.into_body()).await.unwrap();
        let final_body_str = String::from_utf8(final_body.to_vec()).unwrap();

        // Should not contain deleted objects
        assert!(!final_body_str.contains("text/document.txt"));
        assert!(!final_body_str.contains("images/photo.jpg"));

        // Should still contain remaining objects
        assert!(final_body_str.contains("data/config.json"));
        assert!(final_body_str.contains("archives/data.zip"));
        assert!(final_body_str.contains("videos/clip.mp4"));
    }

    #[tokio::test]
    async fn test_error_handling_consistency() {
        let (server, _storage_temp, _metadata_temp) = setup_full_stack().await;
        let app = server.router();

        // Test various error conditions for S3 compliance

        // 1. GET non-existent bucket
        let request = Request::builder()
            .method(Method::GET)
            .uri("/non-existent-bucket")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // 2. PUT object in non-existent bucket
        let request = Request::builder()
            .method(Method::PUT)
            .uri("/non-existent-bucket/file.txt")
            .header(header::CONTENT_TYPE, "text/plain")
            .header(header::CONTENT_LENGTH, "5")
            .body(Body::from("hello"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // 3. Invalid bucket name
        let request = Request::builder()
            .method(Method::PUT)
            .uri("/Invalid_Bucket_Name")
            .header(header::CONTENT_LENGTH, "0")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // 4. Method not allowed
        let request = Request::builder()
            .method(Method::PATCH)
            .uri("/test-bucket")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);

        // 5. Invalid content length
        let request = Request::builder()
            .method(Method::PUT)
            .uri("/test-bucket")
            .header(header::CONTENT_LENGTH, "0")
            .body(Body::empty())
            .unwrap();

        // Create bucket first
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Try with wrong content length
        let request = Request::builder()
            .method(Method::PUT)
            .uri("/test-bucket/file.txt")
            .header(header::CONTENT_TYPE, "text/plain")
            .header(header::CONTENT_LENGTH, "100") // Wrong length
            .body(Body::from("short"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_performance_under_load() {
        let (server, _storage_temp, _metadata_temp) = setup_full_stack().await;
        let app = server.router();

        // Create test bucket
        let create_request = Request::builder()
            .method(Method::PUT)
            .uri("/perf-test-bucket")
            .header(header::CONTENT_LENGTH, "0")
            .body(Body::empty())
            .unwrap();

        let create_response = app.oneshot(create_request).await.unwrap();
        assert_eq!(create_response.status(), StatusCode::OK);

        // Performance test: Upload many files concurrently
        let file_count = 50;
        let start_time = std::time::Instant::now();

        let mut tasks = Vec::new();
        for i in 0..file_count {
            let content = format!("Performance test content for file {}", i);
            let filename = format!("perf-file-{:03}.txt", i);

            let put_request = Request::builder()
                .method(Method::PUT)
                .uri(&format!("/perf-test-bucket/{}", filename))
                .header(header::CONTENT_TYPE, "text/plain")
                .header(header::CONTENT_LENGTH, content.len().to_string())
                .body(Body::from(content))
                .unwrap();

            let task = tokio::spawn(async move {
                app.oneshot(put_request).await.unwrap()
            });

            tasks.push(task);
        }

        // Wait for all uploads
        for task in tasks {
            let response = task.await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }

        let upload_duration = start_time.elapsed();
        println!("Uploaded {} files in {:?}", file_count, upload_duration);

        // Performance test: List all objects
        let list_start = std::time::Instant::now();

        let list_request = Request::builder()
            .method(Method::GET)
            .uri("/perf-test-bucket")
            .body(Body::empty())
            .unwrap();

        let list_response = app.oneshot(list_request).await.unwrap();
        assert_eq!(list_response.status(), StatusCode::OK);

        let list_duration = list_start.elapsed();
        println!("Listed {} files in {:?}", file_count, list_duration);

        // Verify all files are listed
        let body = hyper::body::to_bytes(list_response.into_body()).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        for i in 0..file_count {
            let filename = format!("perf-file-{:03}.txt", i);
            assert!(body_str.contains(&filename));
        }

        // Performance test: Download files concurrently
        let download_start = std::time::Instant::now();

        let mut download_tasks = Vec::new();
        for i in 0..file_count {
            let filename = format!("perf-file-{:03}.txt", i);

            let get_request = Request::builder()
                .method(Method::GET)
                .uri(&format!("/perf-test-bucket/{}", filename))
                .body(Body::empty())
                .unwrap();

            let task = tokio::spawn(async move {
                app.oneshot(get_request).await.unwrap()
            });

            download_tasks.push(task);
        }

        for task in download_tasks {
            let response = task.await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }

        let download_duration = download_start.elapsed();
        println!("Downloaded {} files in {:?}", file_count, download_duration);

        // Basic performance assertions (adjust based on expected performance)
        assert!(upload_duration.as_secs() < 30, "Upload took too long: {:?}", upload_duration);
        assert!(list_duration.as_millis() < 1000, "List took too long: {:?}", list_duration);
        assert!(download_duration.as_secs() < 30, "Download took too long: {:?}", download_duration);
    }

    #[tokio::test]
    async fn test_data_consistency() {
        let (server, _storage_temp, _metadata_temp) = setup_full_stack().await;
        let app = server.router();

        // Create bucket
        let create_request = Request::builder()
            .method(Method::PUT)
            .uri("/consistency-test")
            .header(header::CONTENT_LENGTH, "0")
            .body(Body::empty())
            .unwrap();

        let create_response = app.oneshot(create_request).await.unwrap();
        assert_eq!(create_response.status(), StatusCode::OK);

        // Upload file with specific content
        let original_content = "Original content that must be preserved exactly!";
        let put_request = Request::builder()
            .method(Method::PUT)
            .uri("/consistency-test/data-integrity-test.txt")
            .header(header::CONTENT_TYPE, "text/plain")
            .header(header::CONTENT_LENGTH, original_content.len().to_string())
            .body(Body::from(original_content))
            .unwrap();

        let put_response = app.oneshot(put_request).await.unwrap();
        assert_eq!(put_response.status(), StatusCode::OK);

        let original_etag = put_response.headers().get("ETag").unwrap().clone();

        // Wait a bit to ensure any async operations complete
        sleep(Duration::from_millis(100)).await;

        // Retrieve file multiple times and verify consistency
        for _ in 0..5 {
            let get_request = Request::builder()
                .method(Method::GET)
                .uri("/consistency-test/data-integrity-test.txt")
                .body(Body::empty())
                .unwrap();

            let get_response = app.oneshot(get_request).await.unwrap();
            assert_eq!(get_response.status(), StatusCode::OK);

            // Verify content is exactly the same
            let body = hyper::body::to_bytes(get_response.into_body()).await.unwrap();
            let content = String::from_utf8(body.to_vec()).unwrap();
            assert_eq!(content, original_content);

            // Verify ETag is consistent
            let etag = get_response.headers().get("ETag").unwrap();
            assert_eq!(etag, &original_etag);
        }

        // Test HEAD consistency
        let head_request = Request::builder()
            .method(Method::HEAD)
            .uri("/consistency-test/data-integrity-test.txt")
            .body(Body::empty())
            .unwrap();

        let head_response = app.oneshot(head_request).await.unwrap();
        assert_eq!(head_response.status(), StatusCode::OK);

        let head_etag = head_response.headers().get("ETag").unwrap();
        assert_eq!(head_etag, &original_etag);

        let content_length = head_response.headers().get(header::CONTENT_LENGTH).unwrap();
        assert_eq!(content_length, original_content.len().to_string().as_str());
    }

    #[tokio::test]
    async fn test_large_file_handling() {
        let (server, _storage_temp, _metadata_temp) = setup_full_stack().await;
        let app = server.router();

        // Create bucket
        let create_request = Request::builder()
            .method(Method::PUT)
            .uri("/large-file-test")
            .header(header::CONTENT_LENGTH, "0")
            .body(Body::empty())
            .unwrap();

        let create_response = app.oneshot(create_request).await.unwrap();
        assert_eq!(create_response.status(), StatusCode::OK);

        // Create a 1MB file
        let file_size = 1024 * 1024; // 1MB
        let large_content = "A".repeat(file_size);

        let put_request = Request::builder()
            .method(Method::PUT)
            .uri("/large-file-test/large-file.txt")
            .header(header::CONTENT_TYPE, "text/plain")
            .header(header::CONTENT_LENGTH, large_content.len().to_string())
            .body(Body::from(large_content.clone()))
            .unwrap();

        let start_time = std::time::Instant::now();
        let put_response = app.oneshot(put_request).await.unwrap();
        let upload_time = start_time.elapsed();

        assert_eq!(put_response.status(), StatusCode::OK);
        println!("Uploaded 1MB file in {:?}", upload_time);

        // Download and verify
        let get_request = Request::builder()
            .method(Method::GET)
            .uri("/large-file-test/large-file.txt")
            .body(Body::empty())
            .unwrap();

        let start_time = std::time::Instant::now();
        let get_response = app.oneshot(get_request).await.unwrap();
        assert_eq!(get_response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(get_response.into_body()).await.unwrap();
        let download_time = start_time.elapsed();

        assert_eq!(body.len(), file_size);
        assert_eq!(String::from_utf8(body.to_vec()).unwrap(), large_content);
        println!("Downloaded 1MB file in {:?}", download_time);

        // Performance assertions
        assert!(upload_time.as_secs() < 10, "Large file upload took too long: {:?}", upload_time);
        assert!(download_time.as_secs() < 10, "Large file download took too long: {:?}", download_time);
    }
}

#[cfg(test)]
mod benchmark_tests {
    use super::*;

    #[tokio::test]
    async fn benchmark_api_operations() {
        let (server, _storage_temp, _metadata_temp) = setup_full_stack().await;
        let app = server.router();

        // Benchmark bucket creation
        let start = std::time::Instant::now();
        for i in 0..10 {
            let bucket_name = format!("bench-bucket-{}", i);
            let request = Request::builder()
                .method(Method::PUT)
                .uri(&format!("/{}", bucket_name))
                .header(header::CONTENT_LENGTH, "0")
                .body(Body::empty())
                .unwrap();

            let response = app.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }
        let bucket_creation_time = start.elapsed();
        println!("Created 10 buckets in {:?} ({:?} per bucket)", 
                bucket_creation_time, bucket_creation_time / 10);

        // Benchmark object creation
        let start = std::time::Instant::now();
        let content = "Benchmark test content";
        
        for i in 0..100 {
            let object_key = format!("bench-object-{}.txt", i);
            let request = Request::builder()
                .method(Method::PUT)
                .uri(&format!("/bench-bucket-0/{}", object_key))
                .header(header::CONTENT_TYPE, "text/plain")
                .header(header::CONTENT_LENGTH, content.len().to_string())
                .body(Body::from(content))
                .unwrap();

            let response = app.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }
        let object_creation_time = start.elapsed();
        println!("Created 100 objects in {:?} ({:?} per object)", 
                object_creation_time, object_creation_time / 100);

        // Benchmark listing
        let start = std::time::Instant::now();
        let request = Request::builder()
            .method(Method::GET)
            .uri("/bench-bucket-0")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let listing_time = start.elapsed();
        println!("Listed 100 objects in {:?}", listing_time);

        // Performance targets (adjust based on requirements)
        assert!(bucket_creation_time.as_millis() < 1000, "Bucket creation too slow");
        assert!(object_creation_time.as_millis() < 5000, "Object creation too slow");
        assert!(listing_time.as_millis() < 500, "Object listing too slow");
    }
}
