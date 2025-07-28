//! ObjectIO Storage Abstraction
//!
//! This crate provides a pluggable storage backend abstraction for ObjectIO.

pub mod backend;
pub mod filesystem;
pub mod traits;

pub use backend::StorageBackend;
pub use traits::Storage;
