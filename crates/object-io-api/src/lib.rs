//! ObjectIO API Layer
//!
//! This crate implements the S3-compatible REST API endpoints for ObjectIO.

pub mod auth;
pub mod handlers;
pub mod middleware;
pub mod responses;
pub mod routes;

pub use routes::create_app;
