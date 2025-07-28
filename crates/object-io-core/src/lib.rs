//! ObjectIO Core Types and Utilities
//!
//! This crate contains the core types, error definitions, and shared utilities
//! used across the ObjectIO S3-compatible storage system.

pub mod error;
pub mod types;
pub mod utils;

// Integration tests module
#[cfg(test)]
mod integration_tests;

// Re-export commonly used types
pub use error::{ObjectIOError, Result};
pub use types::*;
pub use utils::*;
