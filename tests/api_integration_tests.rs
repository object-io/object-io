//! API Integration Tests
//! 
//! This module contains comprehensive tests for the S3-compatible REST API
//! endpoints, ensuring proper HTTP behavior and error handling.

use object_io_api::*;
use object_io_core::*;
use object_io_storage::FileSystemStorage;
use axum::{
    body::Body,
    http::{Request, StatusCode, Method, header},
    Router,
};
use tower::ServiceExt; // for `oneshot`
use tempfile::TempDir;
use std::collections::HashMap;

#[cfg(test)]
mod api_bucket_tests {
    use super::*;

    async fn setup_test_app() -> (Router, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage = FileSystemStorage::new(temp_dir.path().to_path_buf())
            .await
            .expect("Failed to create filesystem storage");
        
        let app = create_router(Box::new(storage));
        (app, temp_dir)
    }

    #[tokio::test]
    async fn test_create_bucket_success() {
        let (app, _temp_dir) = setup_test_app().await;

        let request = Request::builder()
            .method(Method::PUT)
            .uri("/test-bucket")
            .header(header::CONTENT_LENGTH, "0")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_create_bucket_invalid_name() {
        let (app, _temp_dir) = setup_test_app().await;

        let request = Request::builder()
            .method(Method::PUT)
            .uri("/Invalid_Bucket_Name")
            .header(header::CONTENT_LENGTH, "0")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_list_buckets() {
        let (app, _temp_dir) = setup_test_app().await;

        // First create a bucket
        let create_request = Request::builder()
            .method(Method::PUT)
            .uri("/test-bucket")
            .header(header::CONTENT_LENGTH, "0")
            .body(Body::empty())
            .unwrap();

        let create_response = app.oneshot(create_request).await.unwrap();
        assert_eq!(create_response.status(), StatusCode::OK);

        // Then list buckets
        let list_request = Request::builder()
            .method(Method::GET)
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let list_response = app.oneshot(list_request).await.unwrap();
        assert_eq!(list_response.status(), StatusCode::OK);

        // Check response body contains XML
        let body = hyper::body::to_bytes(list_response.into_body()).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("<ListAllMyBucketsResult"));
        assert!(body_str.contains("test-bucket"));
    }

    #[tokio::test]
    async fn test_delete_bucket() {
        let (app, _temp_dir) = setup_test_app().await;

        // Create bucket
        let create_request = Request::builder()
            .method(Method::PUT)
            .uri("/test-bucket")
            .header(header::CONTENT_LENGTH, "0")
            .body(Body::empty())
            .unwrap();

        let create_response = app.oneshot(create_request).await.unwrap();
        assert_eq!(create_response.status(), StatusCode::OK);

        // Delete bucket
        let delete_request = Request::builder()
            .method(Method::DELETE)
            .uri("/test-bucket")
            .body(Body::empty())
            .unwrap();

        let delete_response = app.oneshot(delete_request).await.unwrap();
        assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_delete_nonexistent_bucket() {
        let (app, _temp_dir) = setup_test_app().await;

        let request = Request::builder()
            .method(Method::DELETE)
            .uri("/nonexistent-bucket")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}

#[cfg(test)]
mod api_object_tests {
    use super::*;

    async fn setup_test_app_with_bucket() -> (Router, TempDir) {
        let (app, temp_dir) = setup_test_app().await;

        // Create a test bucket
        let request = Request::builder()
            .method(Method::PUT)
            .uri("/test-bucket")
            .header(header::CONTENT_LENGTH, "0")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        (app, temp_dir)
    }

    #[tokio::test]
    async fn test_put_object_success() {
        let (app, _temp_dir) = setup_test_app_with_bucket().await;

        let content = "Hello, ObjectIO API!";
        let request = Request::builder()
            .method(Method::PUT)
            .uri("/test-bucket/test-file.txt")
            .header(header::CONTENT_TYPE, "text/plain")
            .header(header::CONTENT_LENGTH, content.len().to_string())
            .body(Body::from(content))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Check ETag header is present
        let etag = response.headers().get("ETag");
        assert!(etag.is_some());
    }

    #[tokio::test]
    async fn test_get_object_success() {
        let (app, _temp_dir) = setup_test_app_with_bucket().await;

        let content = "Test content for retrieval";

        // First, put an object
        let put_request = Request::builder()
            .method(Method::PUT)
            .uri("/test-bucket/retrieve-test.txt")
            .header(header::CONTENT_TYPE, "text/plain")
            .header(header::CONTENT_LENGTH, content.len().to_string())
            .body(Body::from(content))
            .unwrap();

        let put_response = app.oneshot(put_request).await.unwrap();
        assert_eq!(put_response.status(), StatusCode::OK);

        // Then get it back
        let get_request = Request::builder()
            .method(Method::GET)
            .uri("/test-bucket/retrieve-test.txt")
            .body(Body::empty())
            .unwrap();

        let get_response = app.oneshot(get_request).await.unwrap();
        assert_eq!(get_response.status(), StatusCode::OK);

        // Check content type
        let content_type = get_response.headers().get(header::CONTENT_TYPE).unwrap();
        assert_eq!(content_type, "text/plain");

        // Check content
        let body = hyper::body::to_bytes(get_response.into_body()).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert_eq!(body_str, content);
    }

    #[tokio::test]
    async fn test_get_nonexistent_object() {
        let (app, _temp_dir) = setup_test_app_with_bucket().await;

        let request = Request::builder()
            .method(Method::GET)
            .uri("/test-bucket/nonexistent-file.txt")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_head_object() {
        let (app, _temp_dir) = setup_test_app_with_bucket().await;

        let content = "Content for HEAD test";

        // Put object
        let put_request = Request::builder()
            .method(Method::PUT)
            .uri("/test-bucket/head-test.txt")
            .header(header::CONTENT_TYPE, "text/plain")
            .header(header::CONTENT_LENGTH, content.len().to_string())
            .body(Body::from(content))
            .unwrap();

        let put_response = app.oneshot(put_request).await.unwrap();
        assert_eq!(put_response.status(), StatusCode::OK);

        // HEAD object
        let head_request = Request::builder()
            .method(Method::HEAD)
            .uri("/test-bucket/head-test.txt")
            .body(Body::empty())
            .unwrap();

        let head_response = app.oneshot(head_request).await.unwrap();
        assert_eq!(head_response.status(), StatusCode::OK);

        // Check headers
        let content_type = head_response.headers().get(header::CONTENT_TYPE).unwrap();
        assert_eq!(content_type, "text/plain");

        let content_length = head_response.headers().get(header::CONTENT_LENGTH).unwrap();
        assert_eq!(content_length, content.len().to_string().as_str());

        let etag = head_response.headers().get("ETag");
        assert!(etag.is_some());

        // HEAD should have no body
        let body = hyper::body::to_bytes(head_response.into_body()).await.unwrap();
        assert_eq!(body.len(), 0);
    }

    #[tokio::test]
    async fn test_delete_object() {
        let (app, _temp_dir) = setup_test_app_with_bucket().await;

        let content = "Content to be deleted";

        // Put object
        let put_request = Request::builder()
            .method(Method::PUT)
            .uri("/test-bucket/delete-me.txt")
            .header(header::CONTENT_TYPE, "text/plain")
            .header(header::CONTENT_LENGTH, content.len().to_string())
            .body(Body::from(content))
            .unwrap();

        let put_response = app.oneshot(put_request).await.unwrap();
        assert_eq!(put_response.status(), StatusCode::OK);

        // Delete object
        let delete_request = Request::builder()
            .method(Method::DELETE)
            .uri("/test-bucket/delete-me.txt")
            .body(Body::empty())
            .unwrap();

        let delete_response = app.oneshot(delete_request).await.unwrap();
        assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

        // Try to get deleted object
        let get_request = Request::builder()
            .method(Method::GET)
            .uri("/test-bucket/delete-me.txt")
            .body(Body::empty())
            .unwrap();

        let get_response = app.oneshot(get_request).await.unwrap();
        assert_eq!(get_response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_list_objects() {
        let (app, _temp_dir) = setup_test_app_with_bucket().await;

        // Put multiple objects
        let objects = vec![
            ("file1.txt", "content1"),
            ("path/file2.txt", "content2"),
            ("path/subpath/file3.txt", "content3"),
        ];

        for (key, content) in &objects {
            let put_request = Request::builder()
                .method(Method::PUT)
                .uri(&format!("/test-bucket/{}", key))
                .header(header::CONTENT_TYPE, "text/plain")
                .header(header::CONTENT_LENGTH, content.len().to_string())
                .body(Body::from(*content))
                .unwrap();

            let put_response = app.oneshot(put_request).await.unwrap();
            assert_eq!(put_response.status(), StatusCode::OK);
        }

        // List all objects
        let list_request = Request::builder()
            .method(Method::GET)
            .uri("/test-bucket")
            .body(Body::empty())
            .unwrap();

        let list_response = app.oneshot(list_request).await.unwrap();
        assert_eq!(list_response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(list_response.into_body()).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Check XML structure
        assert!(body_str.contains("<ListBucketResult"));
        assert!(body_str.contains("<Contents>"));
        assert!(body_str.contains("file1.txt"));
        assert!(body_str.contains("path/file2.txt"));
        assert!(body_str.contains("path/subpath/file3.txt"));
    }

    #[tokio::test]
    async fn test_list_objects_with_prefix() {
        let (app, _temp_dir) = setup_test_app_with_bucket().await;

        // Put objects with different prefixes
        let objects = vec![
            ("docs/readme.txt", "readme content"),
            ("docs/guide.txt", "guide content"),
            ("images/logo.png", "png data"),
            ("videos/intro.mp4", "video data"),
        ];

        for (key, content) in &objects {
            let put_request = Request::builder()
                .method(Method::PUT)
                .uri(&format!("/test-bucket/{}", key))
                .header(header::CONTENT_TYPE, "text/plain")
                .header(header::CONTENT_LENGTH, content.len().to_string())
                .body(Body::from(*content))
                .unwrap();

            let put_response = app.oneshot(put_request).await.unwrap();
            assert_eq!(put_response.status(), StatusCode::OK);
        }

        // List objects with prefix
        let list_request = Request::builder()
            .method(Method::GET)
            .uri("/test-bucket?prefix=docs/")
            .body(Body::empty())
            .unwrap();

        let list_response = app.oneshot(list_request).await.unwrap();
        assert_eq!(list_response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(list_response.into_body()).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Should contain docs objects but not others
        assert!(body_str.contains("docs/readme.txt"));
        assert!(body_str.contains("docs/guide.txt"));
        assert!(!body_str.contains("images/logo.png"));
        assert!(!body_str.contains("videos/intro.mp4"));
    }
}

#[cfg(test)]
mod api_error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_method_not_allowed() {
        let (app, _temp_dir) = setup_test_app().await;

        // PATCH is not supported
        let request = Request::builder()
            .method(Method::PATCH)
            .uri("/test-bucket")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn test_invalid_content_length() {
        let (app, _temp_dir) = setup_test_app().await;

        // Create bucket first
        let create_request = Request::builder()
            .method(Method::PUT)
            .uri("/test-bucket")
            .header(header::CONTENT_LENGTH, "0")
            .body(Body::empty())
            .unwrap();

        let create_response = app.oneshot(create_request).await.unwrap();
        assert_eq!(create_response.status(), StatusCode::OK);

        // Try to put object with mismatched content length
        let content = "Hello";
        let request = Request::builder()
            .method(Method::PUT)
            .uri("/test-bucket/test-file.txt")
            .header(header::CONTENT_TYPE, "text/plain")
            .header(header::CONTENT_LENGTH, "100") // Wrong length
            .body(Body::from(content))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_missing_content_type() {
        let (app, _temp_dir) = setup_test_app().await;

        // Create bucket first
        let create_request = Request::builder()
            .method(Method::PUT)
            .uri("/test-bucket")
            .header(header::CONTENT_LENGTH, "0")
            .body(Body::empty())
            .unwrap();

        let create_response = app.oneshot(create_request).await.unwrap();
        assert_eq!(create_response.status(), StatusCode::OK);

        // Put object without content type (should default to binary)
        let content = "Test content";
        let request = Request::builder()
            .method(Method::PUT)
            .uri("/test-bucket/test-file.txt")
            .header(header::CONTENT_LENGTH, content.len().to_string())
            .body(Body::from(content))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_object_in_nonexistent_bucket() {
        let (app, _temp_dir) = setup_test_app().await;

        let request = Request::builder()
            .method(Method::PUT)
            .uri("/nonexistent-bucket/test-file.txt")
            .header(header::CONTENT_TYPE, "text/plain")
            .header(header::CONTENT_LENGTH, "5")
            .body(Body::from("hello"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}

#[cfg(test)]
mod api_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_s3_workflow() {
        let (app, _temp_dir) = setup_test_app().await;

        // 1. Create bucket
        let create_bucket_request = Request::builder()
            .method(Method::PUT)
            .uri("/my-test-bucket")
            .header(header::CONTENT_LENGTH, "0")
            .body(Body::empty())
            .unwrap();

        let create_bucket_response = app.oneshot(create_bucket_request).await.unwrap();
        assert_eq!(create_bucket_response.status(), StatusCode::OK);

        // 2. Put multiple objects
        let files = vec![
            ("document.pdf", "PDF content here", "application/pdf"),
            ("image.jpg", "JPEG data", "image/jpeg"),
            ("data.json", r#"{"key": "value"}"#, "application/json"),
            ("readme.txt", "This is a readme file", "text/plain"),
        ];

        for (filename, content, content_type) in &files {
            let put_request = Request::builder()
                .method(Method::PUT)
                .uri(&format!("/my-test-bucket/{}", filename))
                .header(header::CONTENT_TYPE, *content_type)
                .header(header::CONTENT_LENGTH, content.len().to_string())
                .body(Body::from(*content))
                .unwrap();

            let put_response = app.oneshot(put_request).await.unwrap();
            assert_eq!(put_response.status(), StatusCode::OK);
        }

        // 3. List all objects
        let list_request = Request::builder()
            .method(Method::GET)
            .uri("/my-test-bucket")
            .body(Body::empty())
            .unwrap();

        let list_response = app.oneshot(list_request).await.unwrap();
        assert_eq!(list_response.status(), StatusCode::OK);

        let list_body = hyper::body::to_bytes(list_response.into_body()).await.unwrap();
        let list_body_str = String::from_utf8(list_body.to_vec()).unwrap();

        // Check all files are listed
        for (filename, _, _) in &files {
            assert!(list_body_str.contains(filename));
        }

        // 4. Get specific object
        let get_request = Request::builder()
            .method(Method::GET)
            .uri("/my-test-bucket/data.json")
            .body(Body::empty())
            .unwrap();

        let get_response = app.oneshot(get_request).await.unwrap();
        assert_eq!(get_response.status(), StatusCode::OK);

        let get_body = hyper::body::to_bytes(get_response.into_body()).await.unwrap();
        let get_body_str = String::from_utf8(get_body.to_vec()).unwrap();
        assert_eq!(get_body_str, r#"{"key": "value"}"#);

        // 5. HEAD object
        let head_request = Request::builder()
            .method(Method::HEAD)
            .uri("/my-test-bucket/image.jpg")
            .body(Body::empty())
            .unwrap();

        let head_response = app.oneshot(head_request).await.unwrap();
        assert_eq!(head_response.status(), StatusCode::OK);

        let content_type = head_response.headers().get(header::CONTENT_TYPE).unwrap();
        assert_eq!(content_type, "image/jpeg");

        // 6. Delete an object
        let delete_request = Request::builder()
            .method(Method::DELETE)
            .uri("/my-test-bucket/readme.txt")
            .body(Body::empty())
            .unwrap();

        let delete_response = app.oneshot(delete_request).await.unwrap();
        assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

        // 7. Verify object was deleted
        let get_deleted_request = Request::builder()
            .method(Method::GET)
            .uri("/my-test-bucket/readme.txt")
            .body(Body::empty())
            .unwrap();

        let get_deleted_response = app.oneshot(get_deleted_request).await.unwrap();
        assert_eq!(get_deleted_response.status(), StatusCode::NOT_FOUND);

        // 8. List buckets
        let list_buckets_request = Request::builder()
            .method(Method::GET)
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let list_buckets_response = app.oneshot(list_buckets_request).await.unwrap();
        assert_eq!(list_buckets_response.status(), StatusCode::OK);

        let buckets_body = hyper::body::to_bytes(list_buckets_response.into_body()).await.unwrap();
        let buckets_body_str = String::from_utf8(buckets_body.to_vec()).unwrap();
        assert!(buckets_body_str.contains("my-test-bucket"));
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let (app, _temp_dir) = setup_test_app().await;

        // Create bucket
        let create_request = Request::builder()
            .method(Method::PUT)
            .uri("/concurrent-test")
            .header(header::CONTENT_LENGTH, "0")
            .body(Body::empty())
            .unwrap();

        let create_response = app.oneshot(create_request).await.unwrap();
        assert_eq!(create_response.status(), StatusCode::OK);

        // Simulate concurrent uploads
        let mut tasks = Vec::new();
        
        for i in 0..10 {
            let filename = format!("file-{}.txt", i);
            let content = format!("Content for file {}", i);
            
            let put_request = Request::builder()
                .method(Method::PUT)
                .uri(&format!("/concurrent-test/{}", filename))
                .header(header::CONTENT_TYPE, "text/plain")
                .header(header::CONTENT_LENGTH, content.len().to_string())
                .body(Body::from(content))
                .unwrap();

            let task = tokio::spawn(async move {
                app.oneshot(put_request).await.unwrap()
            });
            
            tasks.push(task);
        }

        // Wait for all uploads to complete
        for task in tasks {
            let response = task.await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }

        // List objects to verify all were uploaded
        let list_request = Request::builder()
            .method(Method::GET)
            .uri("/concurrent-test")
            .body(Body::empty())
            .unwrap();

        let list_response = app.oneshot(list_request).await.unwrap();
        assert_eq!(list_response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(list_response.into_body()).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Check all files are present
        for i in 0..10 {
            assert!(body_str.contains(&format!("file-{}.txt", i)));
        }
    }
}
