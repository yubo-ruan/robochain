//! Parallel Execution Engine for RoboChain
//!
//! Executes transactions in parallel batches based on their object dependencies.
//! Transactions that don't conflict (touch different objects) run in parallel.

mod scheduler;
mod executor;
mod batch;

pub use scheduler::*;
pub use executor::*;
pub use batch::*;

use crate::crypto::Hash;
use crate::objects::ObjectId;
use crate::transactions::{Transaction, TransactionReceipt};
use std::collections::HashSet;

/// Configuration for the execution engine
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    /// Maximum transactions per batch
    pub max_batch_size: usize,
    /// Number of parallel execution threads
    pub num_threads: usize,
    /// Maximum gas per block
    pub max_block_gas: u64,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        ExecutionConfig {
            max_batch_size: 1000,
            num_threads: num_cpus::get(),
            max_block_gas: 100_000_000,
        }
    }
}

/// Result of block execution
#[derive(Debug, Clone)]
pub struct BlockExecutionResult {
    /// Block number
    pub block_number: u64,
    /// State root after execution
    pub state_root: Hash,
    /// Transaction receipts
    pub receipts: Vec<TransactionReceipt>,
    /// Total gas used
    pub total_gas_used: u64,
    /// Number of successful transactions
    pub successful_txs: usize,
    /// Number of failed transactions
    pub failed_txs: usize,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}

/// Statistics about execution
#[derive(Debug, Clone, Default)]
pub struct ExecutionStats {
    /// Number of batches executed
    pub batches: usize,
    /// Average batch size
    pub avg_batch_size: f64,
    /// Maximum parallelism achieved
    pub max_parallelism: usize,
    /// Transactions per second
    pub tps: f64,
}

// Helper to check if we have the num_cpus crate
fn num_cpus_get() -> usize {
    // Default to 4 if num_cpus not available
    4
}

mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    }
}
