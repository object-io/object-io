//! Test Configuration and Utilities
//! 
//! This module provides common test utilities, configuration,
//! and helper functions for ObjectIO testing.

use std::env;
use std::sync::Once;
use tempfile::TempDir;
use tracing::{info, Level};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

static INIT: Once = Once::new();

/// Initialize test logging (call once per test binary)
pub fn init_test_logging() {
    INIT.call_once(|| {
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("object_io=debug,warn"));

        tracing_subscriber::registry()
            .with(fmt::layer().with_test_writer())
            .with(filter)
            .init();

        info!("Test logging initialized");
    });
}

/// Test configuration for ObjectIO tests
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub storage_temp_dir: String,
    pub metadata_temp_dir: String,
    pub test_bucket_prefix: String,
    pub test_object_prefix: String,
    pub concurrent_operations: usize,
    pub large_file_size: usize,
    pub performance_file_count: usize,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            storage_temp_dir: env::var("OBJECTIO_TEST_STORAGE_DIR")
                .unwrap_or_else(|_| "/tmp/objectio_test_storage".to_string()),
            metadata_temp_dir: env::var("OBJECTIO_TEST_METADATA_DIR")
                .unwrap_or_else(|_| "/tmp/objectio_test_metadata".to_string()),
            test_bucket_prefix: "test-bucket".to_string(),
            test_object_prefix: "test-object".to_string(),
            concurrent_operations: env::var("OBJECTIO_TEST_CONCURRENT_OPS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            large_file_size: env::var("OBJECTIO_TEST_LARGE_FILE_SIZE")
                .unwrap_or_else(|_| "1048576".to_string()) // 1MB
                .parse()
                .unwrap_or(1048576),
            performance_file_count: env::var("OBJECTIO_TEST_PERF_FILE_COUNT")
                .unwrap_or_else(|_| "50".to_string())
                .parse()
                .unwrap_or(50),
        }
    }
}

/// Test data generator for consistent test content
pub struct TestDataGenerator;

impl TestDataGenerator {
    /// Generate test content of specified size
    pub fn generate_content(size: usize) -> Vec<u8> {
        let pattern = b"ObjectIO Test Data ";
        let mut content = Vec::with_capacity(size);
        
        while content.len() < size {
            let remaining = size - content.len();
            let chunk_size = std::cmp::min(pattern.len(), remaining);
            content.extend_from_slice(&pattern[..chunk_size]);
        }
        
        content
    }

    /// Generate random-ish content with a seed for reproducibility
    pub fn generate_seeded_content(size: usize, seed: u64) -> Vec<u8> {
        let mut content = Vec::with_capacity(size);
        let mut state = seed;
        
        for _ in 0..size {
            // Simple LCG for reproducible "random" data
            state = state.wrapping_mul(1103515245).wrapping_add(12345);
            content.push((state >> 16) as u8);
        }
        
        content
    }

    /// Generate structured JSON test data
    pub fn generate_json_data(object_count: usize) -> String {
        let mut objects = Vec::new();
        
        for i in 0..object_count {
            objects.push(format!(
                r#"{{"id": {}, "name": "object-{}", "description": "Test object number {}", "timestamp": "2024-01-{:02}T10:00:00Z"}}"#,
                i, i, i, (i % 28) + 1
            ));
        }
        
        format!("[{}]", objects.join(","))
    }

    /// Generate CSV test data
    pub fn generate_csv_data(row_count: usize) -> String {
        let mut csv = String::from("id,name,value,category\n");
        
        for i in 0..row_count {
            csv.push_str(&format!(
                "{},object-{},{:.2},category-{}\n",
                i, i, (i as f64) * 3.14159, i % 5
            ));
        }
        
        csv
    }

    /// Generate binary test data with patterns
    pub fn generate_binary_pattern(size: usize) -> Vec<u8> {
        let mut data = Vec::with_capacity(size);
        
        for i in 0..size {
            data.push(match i % 4 {
                0 => 0xDE,
                1 => 0xAD,
                2 => 0xBE,
                3 => 0xEF,
                _ => unreachable!(),
            });
        }
        
        data
    }
}

/// Performance measurement utilities
pub struct PerformanceTracker {
    operation_name: String,
    start_time: std::time::Instant,
}

impl PerformanceTracker {
    pub fn new(operation_name: &str) -> Self {
        info!("Starting performance tracking for: {}", operation_name);
        Self {
            operation_name: operation_name.to_string(),
            start_time: std::time::Instant::now(),
        }
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    pub fn finish(self) -> std::time::Duration {
        let duration = self.elapsed();
        info!("Completed {} in {:?}", self.operation_name, duration);
        duration
    }

    pub fn checkpoint(&self, checkpoint_name: &str) {
        let elapsed = self.elapsed();
        info!("{} - {}: {:?}", self.operation_name, checkpoint_name, elapsed);
    }
}

/// Assert performance meets expectations
pub fn assert_performance(duration: std::time::Duration, max_duration: std::time::Duration, operation: &str) {
    if duration > max_duration {
        panic!(
            "Performance assertion failed for {}: took {:?}, expected max {:?}",
            operation, duration, max_duration
        );
    }
    info!("Performance assertion passed for {}: {:?} <= {:?}", operation, duration, max_duration);
}

/// Test file cleanup utilities
pub struct TestCleanup {
    temp_dirs: Vec<TempDir>,
}

impl TestCleanup {
    pub fn new() -> Self {
        Self {
            temp_dirs: Vec::new(),
        }
    }

    pub fn add_temp_dir(&mut self, temp_dir: TempDir) {
        self.temp_dirs.push(temp_dir);
    }

    pub fn cleanup(self) {
        info!("Cleaning up {} temporary directories", self.temp_dirs.len());
        // TempDir Drop implementation handles cleanup
    }
}

impl Default for TestCleanup {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive test result reporting
#[derive(Debug, Default)]
pub struct TestResults {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub performance_results: Vec<(String, std::time::Duration)>,
}

impl TestResults {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_test_result(&mut self, test_name: &str, passed: bool) {
        self.total_tests += 1;
        if passed {
            self.passed_tests += 1;
            info!("✅ Test passed: {}", test_name);
        } else {
            self.failed_tests += 1;
            info!("❌ Test failed: {}", test_name);
        }
    }

    pub fn add_performance_result(&mut self, operation: String, duration: std::time::Duration) {
        self.performance_results.push((operation, duration));
    }

    pub fn print_summary(&self) {
        println!("\n=== ObjectIO Test Results Summary ===");
        println!("Total Tests: {}", self.total_tests);
        println!("Passed: {}", self.passed_tests);
        println!("Failed: {}", self.failed_tests);
        println!("Success Rate: {:.2}%", 
                (self.passed_tests as f64 / self.total_tests as f64) * 100.0);

        if !self.performance_results.is_empty() {
            println!("\n=== Performance Results ===");
            for (operation, duration) in &self.performance_results {
                println!("{}: {:?}", operation, duration);
            }
        }

        println!("=====================================\n");
    }
}

/// Macro for easier test timing
#[macro_export]
macro_rules! timed_test {
    ($test_name:expr, $test_body:block) => {{
        let tracker = $crate::test_utils::PerformanceTracker::new($test_name);
        let result = $test_body;
        let duration = tracker.finish();
        (result, duration)
    }};
}

/// Macro for performance assertions
#[macro_export]
macro_rules! assert_perf {
    ($duration:expr, $max_duration:expr, $operation:expr) => {
        $crate::test_utils::assert_performance($duration, $max_duration, $operation);
    };
}

#[cfg(test)]
mod test_utils_tests {
    use super::*;

    #[test]
    fn test_data_generator() {
        // Test content generation
        let content = TestDataGenerator::generate_content(100);
        assert_eq!(content.len(), 100);

        // Test seeded generation is reproducible
        let content1 = TestDataGenerator::generate_seeded_content(50, 12345);
        let content2 = TestDataGenerator::generate_seeded_content(50, 12345);
        assert_eq!(content1, content2);

        // Test different seeds produce different content
        let content3 = TestDataGenerator::generate_seeded_content(50, 54321);
        assert_ne!(content1, content3);

        // Test JSON generation
        let json = TestDataGenerator::generate_json_data(3);
        assert!(json.contains("object-0"));
        assert!(json.contains("object-2"));

        // Test CSV generation
        let csv = TestDataGenerator::generate_csv_data(2);
        assert!(csv.contains("id,name,value,category"));
        assert!(csv.contains("object-0"));

        // Test binary pattern
        let binary = TestDataGenerator::generate_binary_pattern(8);
        assert_eq!(binary, vec![0xDE, 0xAD, 0xBE, 0xEF, 0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn test_performance_tracker() {
        let tracker = PerformanceTracker::new("test_operation");
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let elapsed = tracker.elapsed();
        assert!(elapsed >= std::time::Duration::from_millis(10));
        
        let final_duration = tracker.finish();
        assert!(final_duration >= elapsed);
    }

    #[test]
    fn test_config() {
        let config = TestConfig::default();
        assert!(config.concurrent_operations > 0);
        assert!(config.large_file_size > 0);
        assert!(config.performance_file_count > 0);
    }

    #[test]
    fn test_results_tracking() {
        let mut results = TestResults::new();
        
        results.add_test_result("test1", true);
        results.add_test_result("test2", false);
        results.add_test_result("test3", true);
        
        assert_eq!(results.total_tests, 3);
        assert_eq!(results.passed_tests, 2);
        assert_eq!(results.failed_tests, 1);

        results.add_performance_result("upload".to_string(), std::time::Duration::from_millis(100));
        assert_eq!(results.performance_results.len(), 1);
    }
}
