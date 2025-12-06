//! Storage layer for RoboChain
//!
//! Provides object storage with support for:
//! - In-memory storage (for testing)
//! - RocksDB persistence (for production)

mod memory;
mod traits;

pub use memory::*;
pub use traits::*;

use crate::crypto::Hash;
use crate::objects::*;
use thiserror::Error;

/// Storage errors
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Object not found: {0:?}")]
    NotFound(ObjectId),

    #[error("Version mismatch: expected {expected}, got {got}")]
    VersionMismatch { expected: u64, got: u64 },

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Database error: {0}")]
    Database(String),
}

/// State root of the object store
#[derive(Debug, Clone)]
pub struct StateRoot {
    /// Merkle root of all objects
    pub root_hash: Hash,
    /// Block number
    pub block_number: u64,
    /// Number of objects
    pub object_count: u64,
}
