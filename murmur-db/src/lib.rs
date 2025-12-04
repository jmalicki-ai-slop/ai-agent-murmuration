//! Database persistence layer for Murmuration
//!
//! Provides SQLite-backed storage for:
//! - GitHub issue state tracking
//! - Agent run history
//! - Conversation logs

pub mod db;
pub mod error;
pub mod models;
pub mod repos;

pub use db::{Database, DatabaseConfig};
pub use error::{DbError, Result};
