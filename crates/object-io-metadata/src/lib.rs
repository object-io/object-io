//! ObjectIO Metadata Management
//!
//! This crate handles metadata storage and retrieval using SurrealDB.

pub mod database;
pub mod models;
pub mod operations;

pub use database::Database;
pub use operations::MetadataOperations;
