//! Database persistence layer for Murmuration
//!
//! This crate provides SQLite-based persistence for:
//! - Agent run history
//! - Issue state tracking
//! - Conversation logs
//!
//! The database is stored at `~/.cache/murmur/murmur.db`

pub mod connection;
pub mod error;
pub mod models;
pub mod repos;

pub use connection::Database;
pub use error::{Error, Result};
