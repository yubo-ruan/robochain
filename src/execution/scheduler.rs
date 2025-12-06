//! Transaction scheduler for parallel execution
//!
//! Builds a dependency graph from transactions and schedules them
//! into batches that can be executed in parallel.

use crate::objects::ObjectId;
use crate::transactions::Transaction;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::Topo;
use std::collections::{HashMap, HashSet};

/// A batch of transactions that can be executed in parallel
#[derive(Debug, Clone)]
pub struct TransactionBatch {
    /// Transactions in this batch
    pub transactions: Vec<Transaction>,
    /// Objects locked by this batch (for debugging)
    pub locked_objects: HashSet<ObjectId>,
    /// Batch index
    pub batch_index: usize,
}

impl TransactionBatch {
    pub fn new(batch_index: usize) -> Self {
        TransactionBatch {
            transactions: Vec::new(),
            locked_objects: HashSet::new(),
            batch_index,
        }
    }

    pub fn add(&mut self, tx: Transaction) {
        // Add all objects touched by this transaction to locked set
        self.locked_objects.extend(tx.read_set());
        self.locked_objects.extend(tx.write_set());
        self.transactions.push(tx);
    }

    pub fn conflicts_with(&self, tx: &Transaction) -> bool {
        let tx_reads = tx.read_set();
        let tx_writes = tx.write_set();

        // Check if any of our locked objects are written by the new tx
        // or if any of the new tx's reads/writes conflict with our locks
        for obj in &self.locked_objects {
            if tx_writes.contains(obj) {
                return true;
            }
        }

        for obj in tx_writes.iter().chain(tx_reads.iter()) {
            if self.locked_objects.contains(obj) {
                // Only conflict if it's a write
                if tx_writes.contains(obj) {
                    return true;
                }
                // Read-read is fine, but read-write conflicts
                // Check if any tx in batch writes to this object
                for existing_tx in &self.transactions {
                    if existing_tx.write_set().contains(obj) {
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn len(&self) -> usize {
        self.transactions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }
}

/// Transaction scheduler that groups transactions into parallel batches
pub struct TransactionScheduler {
    /// Maximum transactions per batch
    max_batch_size: usize,
}

impl TransactionScheduler {
    pub fn new(max_batch_size: usize) -> Self {
        TransactionScheduler { max_batch_size }
    }

    /// Schedule transactions into parallel batches
    ///
    /// Uses a greedy algorithm:
    /// 1. For each transaction, try to add it to an existing batch
    /// 2. If it conflicts with all existing batches, create a new batch
    pub fn schedule(&self, transactions: Vec<Transaction>) -> Vec<TransactionBatch> {
        if transactions.is_empty() {
            return Vec::new();
        }

        let mut batches: Vec<TransactionBatch> = Vec::new();
        let mut batch_index = 0;

        for tx in transactions {
            // Try to find a batch that doesn't conflict
            let mut added = false;

            for batch in &mut batches {
                if batch.len() < self.max_batch_size && !batch.conflicts_with(&tx) {
                    batch.add(tx.clone());
                    added = true;
                    break;
                }
            }

            if !added {
                // Create new batch
                let mut new_batch = TransactionBatch::new(batch_index);
                new_batch.add(tx);
                batches.push(new_batch);
                batch_index += 1;
            }
        }

        batches
    }

    /// Schedule using dependency graph for optimal ordering
    ///
    /// Builds a DAG where edges represent dependencies,
    /// then uses topological sort to find parallel batches.
    pub fn schedule_with_dag(&self, transactions: Vec<Transaction>) -> Vec<TransactionBatch> {
        if transactions.is_empty() {
            return Vec::new();
        }

        // Build dependency graph
        let mut graph: DiGraph<usize, ()> = DiGraph::new();
        let mut tx_to_node: HashMap<usize, NodeIndex> = HashMap::new();

        // Add all transactions as nodes
        for (i, _) in transactions.iter().enumerate() {
            let node = graph.add_node(i);
            tx_to_node.insert(i, node);
        }

        // Add edges for dependencies
        for i in 0..transactions.len() {
            for j in (i + 1)..transactions.len() {
                if transactions[i].conflicts_with(&transactions[j]) {
                    // j depends on i (must come after)
                    graph.add_edge(tx_to_node[&i], tx_to_node[&j], ());
                }
            }
        }

        // Compute levels (distance from root)
        let mut levels: HashMap<usize, usize> = HashMap::new();
        let mut topo = Topo::new(&graph);

        while let Some(node) = topo.next(&graph) {
            let tx_idx = graph[node];
            let max_parent_level = graph
                .neighbors_directed(node, petgraph::Direction::Incoming)
                .map(|parent| levels.get(&graph[parent]).unwrap_or(&0) + 1)
                .max()
                .unwrap_or(0);
            levels.insert(tx_idx, max_parent_level);
        }

        // Group by level
        let max_level = levels.values().max().copied().unwrap_or(0);
        let mut batches: Vec<TransactionBatch> = Vec::new();

        for level in 0..=max_level {
            let mut batch = TransactionBatch::new(level);
            for (tx_idx, tx_level) in &levels {
                if *tx_level == level {
                    batch.add(transactions[*tx_idx].clone());
                }
            }
            if !batch.is_empty() {
                batches.push(batch);
            }
        }

        batches
    }

    /// Analyze parallelism potential
    pub fn analyze_parallelism(&self, transactions: &[Transaction]) -> ParallelismAnalysis {
        if transactions.is_empty() {
            return ParallelismAnalysis::default();
        }

        let batches = self.schedule(transactions.to_vec());

        let total_txs = transactions.len();
        let num_batches = batches.len();
        let max_batch_size = batches.iter().map(|b| b.len()).max().unwrap_or(0);
        let min_batch_size = batches.iter().map(|b| b.len()).min().unwrap_or(0);
        let avg_batch_size = total_txs as f64 / num_batches as f64;

        // Calculate conflict ratio
        let mut conflict_count = 0;
        for i in 0..transactions.len() {
            for j in (i + 1)..transactions.len() {
                if transactions[i].conflicts_with(&transactions[j]) {
                    conflict_count += 1;
                }
            }
        }
        let max_conflicts = (total_txs * (total_txs - 1)) / 2;
        let conflict_ratio = if max_conflicts > 0 {
            conflict_count as f64 / max_conflicts as f64
        } else {
            0.0
        };

        // Parallelism factor: how much speedup we could get
        let sequential_time = total_txs as f64;
        let parallel_time = num_batches as f64;
        let parallelism_factor = sequential_time / parallel_time;

        ParallelismAnalysis {
            total_transactions: total_txs,
            num_batches,
            max_batch_size,
            min_batch_size,
            avg_batch_size,
            conflict_ratio,
            parallelism_factor,
        }
    }
}

/// Analysis of parallelism potential
#[derive(Debug, Clone, Default)]
pub struct ParallelismAnalysis {
    pub total_transactions: usize,
    pub num_batches: usize,
    pub max_batch_size: usize,
    pub min_batch_size: usize,
    pub avg_batch_size: f64,
    pub conflict_ratio: f64,
    pub parallelism_factor: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{KeyPair, Hash};
    use crate::objects::*;
    use crate::transactions::{TransactionPayload, RegisterSpaceTx};

    fn create_test_tx(space_name: &str) -> Transaction {
        let keypair = KeyPair::generate();
        let payload = TransactionPayload::RegisterSpace(RegisterSpaceTx {
            name: space_name.to_string(),
            space_type: SpaceType::Residential,
            geo_region: GeoRegion {
                country: "US".to_string(),
                region: None,
                city: None,
                approx_coords: None,
            },
        });

        let mut tx = Transaction::new(payload, keypair.public_key(), 1000, 1);
        tx.sign(&keypair);
        tx
    }

    fn create_conflicting_tx(space_id: SpaceId) -> Transaction {
        let keypair = KeyPair::generate();
        let payload = TransactionPayload::SetPolicy(crate::transactions::SetPolicyTx {
            space_id,
            policy_graph_hash: Hash::from_bytes(b"policy"),
        });

        let mut tx = Transaction::new(payload, keypair.public_key(), 1000, 1);
        tx.sign(&keypair);
        tx
    }

    #[test]
    fn test_non_conflicting_batch() {
        let scheduler = TransactionScheduler::new(100);

        // Three non-conflicting transactions (creating different spaces)
        let txs = vec![
            create_test_tx("Space1"),
            create_test_tx("Space2"),
            create_test_tx("Space3"),
        ];

        let batches = scheduler.schedule(txs);

        // All should fit in one batch (no conflicts)
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].len(), 3);
    }

    #[test]
    fn test_conflicting_transactions() {
        let scheduler = TransactionScheduler::new(100);
        let space_id = SpaceId::from_bytes(b"shared_space");

        // Two transactions that both write to the same space
        let tx1 = create_conflicting_tx(space_id);
        let tx2 = create_conflicting_tx(space_id);

        let batches = scheduler.schedule(vec![tx1, tx2]);

        // Should be in separate batches due to conflict
        assert_eq!(batches.len(), 2);
    }

    #[test]
    fn test_parallelism_analysis() {
        let scheduler = TransactionScheduler::new(100);

        let txs = vec![
            create_test_tx("Space1"),
            create_test_tx("Space2"),
            create_test_tx("Space3"),
            create_test_tx("Space4"),
        ];

        let analysis = scheduler.analyze_parallelism(&txs);

        assert_eq!(analysis.total_transactions, 4);
        assert_eq!(analysis.num_batches, 1); // All non-conflicting
        assert!(analysis.parallelism_factor >= 4.0); // 4x speedup potential
    }
}
