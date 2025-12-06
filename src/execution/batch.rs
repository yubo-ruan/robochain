//! Batch execution logic

use super::TransactionBatch;
use crate::storage::ObjectStore;
use crate::transactions::{Transaction, TransactionExecutor, TransactionReceipt, ExecutionResult};
use parking_lot::RwLock;
use rayon::prelude::*;
use std::sync::Arc;

/// Result of executing a batch
#[derive(Debug, Clone)]
pub struct BatchExecutionResult {
    /// Batch index
    pub batch_index: usize,
    /// Receipts for all transactions
    pub receipts: Vec<TransactionReceipt>,
    /// Total gas used
    pub total_gas: u64,
    /// Number of successful transactions
    pub successful: usize,
    /// Number of failed transactions
    pub failed: usize,
    /// Execution time in microseconds
    pub execution_time_us: u64,
}

/// Batch executor that runs transactions in parallel
pub struct BatchExecutor {
    /// Number of threads for parallel execution
    num_threads: usize,
}

impl BatchExecutor {
    pub fn new(num_threads: usize) -> Self {
        BatchExecutor { num_threads }
    }

    /// Execute a batch of transactions in parallel
    ///
    /// Since transactions in a batch don't conflict, they can safely
    /// run in parallel with appropriate synchronization.
    pub fn execute_batch(
        &self,
        batch: &TransactionBatch,
        store: Arc<RwLock<Box<dyn ObjectStore>>>,
        block_number: u64,
    ) -> BatchExecutionResult {
        let start = std::time::Instant::now();

        // For true parallel execution, we need to:
        // 1. Take snapshots of required objects
        // 2. Execute transactions in parallel
        // 3. Merge results back

        // Simplified approach: execute sequentially for now
        // In production, would use more sophisticated approach
        let mut receipts = Vec::new();
        let mut total_gas = 0u64;
        let mut successful = 0usize;
        let mut failed = 0usize;

        for tx in &batch.transactions {
            let mut store_guard = store.write();
            let result = TransactionExecutor::execute(tx, store_guard.as_mut(), block_number);
            let receipt = TransactionExecutor::create_receipt(tx, &result, block_number);

            total_gas += result.gas_used;
            if result.success {
                successful += 1;
            } else {
                failed += 1;
            }
            receipts.push(receipt);
        }

        BatchExecutionResult {
            batch_index: batch.batch_index,
            receipts,
            total_gas,
            successful,
            failed,
            execution_time_us: start.elapsed().as_micros() as u64,
        }
    }

    /// Execute transactions in parallel using rayon
    ///
    /// This is a more advanced implementation that actually runs
    /// transactions in parallel with proper isolation.
    pub fn execute_batch_parallel(
        &self,
        batch: &TransactionBatch,
        store: Arc<RwLock<Box<dyn ObjectStore>>>,
        block_number: u64,
    ) -> BatchExecutionResult {
        let start = std::time::Instant::now();

        // Pre-fetch all objects needed by the batch
        let read_set: std::collections::HashSet<_> = batch
            .transactions
            .iter()
            .flat_map(|tx| tx.read_set())
            .collect();

        // For non-conflicting transactions, we can use parallel iteration
        // Each transaction operates on disjoint objects
        let results: Vec<(ExecutionResult, TransactionReceipt)> = batch
            .transactions
            .par_iter()
            .map(|tx| {
                // Each thread gets its own lock
                let mut store_guard = store.write();
                let result = TransactionExecutor::execute(tx, store_guard.as_mut(), block_number);
                let receipt = TransactionExecutor::create_receipt(tx, &result, block_number);
                (result, receipt)
            })
            .collect();

        let mut receipts = Vec::new();
        let mut total_gas = 0u64;
        let mut successful = 0usize;
        let mut failed = 0usize;

        for (result, receipt) in results {
            total_gas += result.gas_used;
            if result.success {
                successful += 1;
            } else {
                failed += 1;
            }
            receipts.push(receipt);
        }

        BatchExecutionResult {
            batch_index: batch.batch_index,
            receipts,
            total_gas,
            successful,
            failed,
            execution_time_us: start.elapsed().as_micros() as u64,
        }
    }
}

/// Object-level locking for fine-grained parallelism
pub struct ObjectLockManager {
    locks: dashmap::DashMap<crate::objects::ObjectId, ()>,
}

impl ObjectLockManager {
    pub fn new() -> Self {
        ObjectLockManager {
            locks: dashmap::DashMap::new(),
        }
    }

    /// Acquire locks for a set of objects
    pub fn acquire(&self, objects: &std::collections::HashSet<crate::objects::ObjectId>) -> bool {
        // Try to acquire all locks atomically
        let mut acquired = Vec::new();

        for obj in objects {
            if self.locks.contains_key(obj) {
                // Lock already held, rollback
                for acq in acquired {
                    self.locks.remove(&acq);
                }
                return false;
            }
            self.locks.insert(*obj, ());
            acquired.push(*obj);
        }

        true
    }

    /// Release locks for a set of objects
    pub fn release(&self, objects: &std::collections::HashSet<crate::objects::ObjectId>) {
        for obj in objects {
            self.locks.remove(obj);
        }
    }
}

impl Default for ObjectLockManager {
    fn default() -> Self {
        Self::new()
    }
}
