//! Main execution engine

use super::{
    BatchExecutionResult, BatchExecutor, BlockExecutionResult, ExecutionConfig, ExecutionStats,
    TransactionBatch, TransactionScheduler,
};
use crate::crypto::Hash;
use crate::storage::{MemoryStore, ObjectStore};
use crate::transactions::{Transaction, TransactionReceipt};
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Instant;

/// The main execution engine for RoboChain
///
/// Orchestrates transaction scheduling and parallel batch execution.
pub struct ExecutionEngine {
    /// Configuration
    config: ExecutionConfig,
    /// Transaction scheduler
    scheduler: TransactionScheduler,
    /// Batch executor
    batch_executor: BatchExecutor,
    /// Object store
    store: Arc<RwLock<Box<dyn ObjectStore>>>,
    /// Current block number
    current_block: u64,
    /// Execution statistics
    stats: ExecutionStats,
}

impl ExecutionEngine {
    /// Create a new execution engine with default memory store
    pub fn new(config: ExecutionConfig) -> Self {
        let store: Box<dyn ObjectStore> = Box::new(MemoryStore::new());

        ExecutionEngine {
            scheduler: TransactionScheduler::new(config.max_batch_size),
            batch_executor: BatchExecutor::new(config.num_threads),
            store: Arc::new(RwLock::new(store)),
            current_block: 0,
            stats: ExecutionStats::default(),
            config,
        }
    }

    /// Create with a custom store
    pub fn with_store(config: ExecutionConfig, store: Box<dyn ObjectStore>) -> Self {
        ExecutionEngine {
            scheduler: TransactionScheduler::new(config.max_batch_size),
            batch_executor: BatchExecutor::new(config.num_threads),
            store: Arc::new(RwLock::new(store)),
            current_block: 0,
            stats: ExecutionStats::default(),
            config,
        }
    }

    /// Execute a block of transactions
    pub fn execute_block(&mut self, transactions: Vec<Transaction>) -> BlockExecutionResult {
        let start = Instant::now();
        self.current_block += 1;
        let block_number = self.current_block;

        if transactions.is_empty() {
            return BlockExecutionResult {
                block_number,
                state_root: self.store.read().compute_state_root(),
                receipts: Vec::new(),
                total_gas_used: 0,
                successful_txs: 0,
                failed_txs: 0,
                execution_time_ms: 0,
            };
        }

        // Schedule transactions into batches
        let batches = self.scheduler.schedule(transactions);

        // Execute batches sequentially (batches can run in parallel internally)
        let mut all_receipts = Vec::new();
        let mut total_gas = 0u64;
        let mut successful = 0usize;
        let mut failed = 0usize;
        let mut max_parallelism = 0usize;

        for batch in &batches {
            max_parallelism = max_parallelism.max(batch.len());

            let batch_result =
                self.batch_executor
                    .execute_batch(batch, self.store.clone(), block_number);

            all_receipts.extend(batch_result.receipts);
            total_gas += batch_result.total_gas;
            successful += batch_result.successful;
            failed += batch_result.failed;
        }

        // Update stats
        let execution_time = start.elapsed();
        let total_txs = successful + failed;

        self.stats.batches += batches.len();
        self.stats.max_parallelism = self.stats.max_parallelism.max(max_parallelism);
        self.stats.avg_batch_size =
            (self.stats.avg_batch_size + (total_txs as f64 / batches.len().max(1) as f64)) / 2.0;
        self.stats.tps = total_txs as f64 / execution_time.as_secs_f64().max(0.001);

        // Compute state root
        let state_root = self.store.read().compute_state_root();

        BlockExecutionResult {
            block_number,
            state_root,
            receipts: all_receipts,
            total_gas_used: total_gas,
            successful_txs: successful,
            failed_txs: failed,
            execution_time_ms: execution_time.as_millis() as u64,
        }
    }

    /// Execute a single transaction
    pub fn execute_transaction(&mut self, tx: Transaction) -> TransactionReceipt {
        let result = self.execute_block(vec![tx]);
        result.receipts.into_iter().next().unwrap()
    }

    /// Get current block number
    pub fn current_block(&self) -> u64 {
        self.current_block
    }

    /// Get execution statistics
    pub fn stats(&self) -> &ExecutionStats {
        &self.stats
    }

    /// Get a reference to the object store
    pub fn store(&self) -> Arc<RwLock<Box<dyn ObjectStore>>> {
        self.store.clone()
    }

    /// Analyze potential parallelism for a set of transactions
    pub fn analyze_parallelism(
        &self,
        transactions: &[Transaction],
    ) -> super::scheduler::ParallelismAnalysis {
        self.scheduler.analyze_parallelism(transactions)
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = ExecutionStats::default();
    }

    /// Get object count in store
    pub fn object_count(&self) -> u64 {
        self.store.read().object_count()
    }
}

/// Builder for ExecutionEngine
pub struct ExecutionEngineBuilder {
    config: ExecutionConfig,
    store: Option<Box<dyn ObjectStore>>,
}

impl ExecutionEngineBuilder {
    pub fn new() -> Self {
        ExecutionEngineBuilder {
            config: ExecutionConfig::default(),
            store: None,
        }
    }

    pub fn with_config(mut self, config: ExecutionConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_max_batch_size(mut self, size: usize) -> Self {
        self.config.max_batch_size = size;
        self
    }

    pub fn with_num_threads(mut self, threads: usize) -> Self {
        self.config.num_threads = threads;
        self
    }

    pub fn with_store(mut self, store: Box<dyn ObjectStore>) -> Self {
        self.store = Some(store);
        self
    }

    pub fn build(self) -> ExecutionEngine {
        match self.store {
            Some(store) => ExecutionEngine::with_store(self.config, store),
            None => ExecutionEngine::new(self.config),
        }
    }
}

impl Default for ExecutionEngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::KeyPair;
    use crate::objects::*;
    use crate::transactions::{TransactionPayload, RegisterSpaceTx, RegisterPrincipalTx};

    fn create_register_space_tx(name: &str, keypair: &KeyPair) -> Transaction {
        let payload = TransactionPayload::RegisterSpace(RegisterSpaceTx {
            name: name.to_string(),
            space_type: SpaceType::Residential,
            geo_region: GeoRegion {
                country: "US".to_string(),
                region: None,
                city: None,
                approx_coords: None,
            },
        });

        let mut tx = Transaction::new(payload, keypair.public_key(), 1000, 1);
        tx.sign(keypair);
        tx
    }

    fn create_register_principal_tx(name: &str, keypair: &KeyPair) -> Transaction {
        let payload = TransactionPayload::RegisterPrincipal(RegisterPrincipalTx {
            display_name: name.to_string(),
            principal_type: PrincipalType::Owner,
        });

        let mut tx = Transaction::new(payload, keypair.public_key(), 1000, 1);
        tx.sign(keypair);
        tx
    }

    #[test]
    fn test_execute_single_transaction() {
        let config = ExecutionConfig::default();
        let mut engine = ExecutionEngine::new(config);

        let keypair = KeyPair::generate();
        let tx = create_register_space_tx("Test Space", &keypair);

        let receipt = engine.execute_transaction(tx);

        assert!(receipt.success);
        assert_eq!(engine.current_block(), 1);
        assert_eq!(engine.object_count(), 1);
    }

    #[test]
    fn test_execute_block() {
        let config = ExecutionConfig::default();
        let mut engine = ExecutionEngine::new(config);

        let keypair1 = KeyPair::generate();
        let keypair2 = KeyPair::generate();

        let transactions = vec![
            create_register_space_tx("Space 1", &keypair1),
            create_register_space_tx("Space 2", &keypair2),
            create_register_principal_tx("Alice", &keypair1),
            create_register_principal_tx("Bob", &keypair2),
        ];

        let result = engine.execute_block(transactions);

        assert_eq!(result.block_number, 1);
        assert_eq!(result.successful_txs, 4);
        assert_eq!(result.failed_txs, 0);
        assert_eq!(engine.object_count(), 4);
    }

    #[test]
    fn test_parallelism_analysis() {
        let config = ExecutionConfig::default();
        let engine = ExecutionEngine::new(config);

        let keypair = KeyPair::generate();
        let transactions: Vec<_> = (0..10)
            .map(|i| create_register_space_tx(&format!("Space {}", i), &keypair))
            .collect();

        let analysis = engine.analyze_parallelism(&transactions);

        assert_eq!(analysis.total_transactions, 10);
        // All should be parallelizable since they create different objects
        assert!(analysis.parallelism_factor >= 1.0);
    }

    #[test]
    fn test_execution_stats() {
        let config = ExecutionConfig::default();
        let mut engine = ExecutionEngine::new(config);

        let keypair = KeyPair::generate();
        let transactions: Vec<_> = (0..100)
            .map(|i| create_register_space_tx(&format!("Space {}", i), &keypair))
            .collect();

        engine.execute_block(transactions);

        let stats = engine.stats();
        assert!(stats.batches > 0);
        assert!(stats.tps > 0.0);
    }
}
