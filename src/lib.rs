//! RoboChain: A specialized L1 blockchain for home robot coordination
//!
//! This crate provides the core implementation of RoboChain, including:
//! - Domain objects (Robot, Space, Task, Policy, etc.)
//! - Transaction types for robot lifecycle events
//! - Parallel execution engine
//! - BFT consensus layer

pub mod objects;
pub mod transactions;
pub mod execution;
pub mod consensus;
pub mod storage;
pub mod crypto;
pub mod explorer;

// Re-export commonly used types
pub use objects::*;
pub use transactions::Transaction;
pub use execution::ExecutionEngine;
pub use consensus::ConsensusEngine;
pub use storage::ObjectStore;
pub use crypto::{KeyPair, PublicKey, Signature};
