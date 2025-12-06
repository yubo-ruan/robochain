//! Explorer service - core logic for querying blockchain data

use super::types::*;
use crate::execution::ExecutionEngine;
use crate::objects::TaskStatus;
use crate::storage::ObjectStore;
use parking_lot::RwLock;
use std::sync::Arc;

/// The main explorer service
pub struct ExplorerService {
    /// Reference to the execution engine
    execution: Arc<RwLock<ExecutionEngine>>,
}

impl ExplorerService {
    /// Create a new explorer service
    pub fn new(execution: Arc<RwLock<ExecutionEngine>>) -> Self {
        ExplorerService { execution }
    }

    /// Get chain statistics
    pub fn get_stats(&self) -> ChainStats {
        let exec = self.execution.read();
        let store = exec.store();
        let store_read = store.read();

        // Count active tasks
        let active_statuses = [
            TaskStatus::Posted,
            TaskStatus::Accepted,
            TaskStatus::InProgress,
        ];
        let mut active_tasks = 0u64;
        for status in &active_statuses {
            active_tasks += store_read.get_tasks_by_status(status).len() as u64;
        }

        ChainStats {
            total_blocks: exec.current_block(),
            total_transactions: exec.stats().batches as u64, // Approximation
            total_robots: 0,    // Would need to track in store
            total_spaces: 0,    // Would need to track in store
            total_tasks: 0,     // Would need to track in store
            active_tasks,
            total_principals: 0, // Would need to track in store
        }
    }

    /// Get recent blocks
    pub fn get_recent_blocks(&self, limit: usize) -> Vec<BlockSummary> {
        // In V1 with in-memory execution, we don't store full blocks
        // This would need integration with ConsensusEngine
        // For now, return mock data based on current block height
        let exec = self.execution.read();
        let current = exec.current_block();

        let mut blocks = Vec::new();
        let start = if current > limit as u64 {
            current - limit as u64
        } else {
            0
        };

        for height in (start..=current).rev() {
            blocks.push(BlockSummary {
                height,
                hash: format!("0x{:016x}", height), // Placeholder
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    - (current - height),
                tx_count: 1,
                total_gas: 30000,
                proposer: "0x...".to_string(),
            });
        }

        blocks
    }

    /// Get block by height
    pub fn get_block(&self, height: u64) -> Option<BlockView> {
        let exec = self.execution.read();
        if height > exec.current_block() {
            return None;
        }

        // In V1, we return a placeholder since we don't store full blocks
        Some(BlockView {
            height,
            hash: format!("0x{:016x}", height),
            prev_hash: if height > 0 {
                format!("0x{:016x}", height - 1)
            } else {
                "0x0000000000000000".to_string()
            },
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            proposer: "0x...".to_string(),
            tx_count: 1,
            total_gas: 30000,
            state_root: "0x...".to_string(),
            transactions: vec![],
        })
    }

    /// Get all robots
    pub fn get_robots(&self) -> Vec<RobotView> {
        // Would need to iterate through store
        // For now, return empty - store doesn't have list_all_robots
        vec![]
    }

    /// Get all spaces
    pub fn get_spaces(&self) -> Vec<SpaceView> {
        vec![]
    }

    /// Get all tasks
    pub fn get_tasks(&self, status_filter: Option<String>) -> Vec<TaskView> {
        let exec = self.execution.read();
        let store = exec.store();
        let store_read = store.read();

        let status = match status_filter.as_deref() {
            Some("posted") => Some(TaskStatus::Posted),
            Some("accepted") => Some(TaskStatus::Accepted),
            Some("in_progress") => Some(TaskStatus::InProgress),
            Some("completed") => Some(TaskStatus::Completed),
            Some("failed") => Some(TaskStatus::Failed),
            Some("cancelled") => Some(TaskStatus::Cancelled),
            Some("disputed") => Some(TaskStatus::Disputed),
            _ => None,
        };

        let tasks = if let Some(ref s) = status {
            store_read.get_tasks_by_status(s)
        } else {
            // Get all tasks by querying each status
            let mut all = Vec::new();
            for s in &[
                TaskStatus::Posted,
                TaskStatus::Accepted,
                TaskStatus::InProgress,
                TaskStatus::Completed,
                TaskStatus::Failed,
                TaskStatus::Cancelled,
                TaskStatus::Disputed,
            ] {
                all.extend(store_read.get_tasks_by_status(s));
            }
            all
        };

        tasks
            .into_iter()
            .map(|t| TaskView {
                id: format!("{:?}", t.id),
                space_id: format!("{:?}", t.space_id),
                requester: format!("{:?}", t.requester),
                description: t.description.clone(),
                status: format!("{:?}", t.status),
                reward: t.reward,
                required_capabilities: t
                    .required_capabilities
                    .iter()
                    .map(|c| format!("{:?}", c))
                    .collect(),
                assigned_robot: t.assigned_robot.map(|r| format!("{:?}", r)),
                created_at: t.created_at,
                deadline: t.deadline,
                accepted_at: t.accepted_at,
                completed_at: t.completed_at,
            })
            .collect()
    }

    /// Search across all entity types
    pub fn search(&self, query: &str) -> SearchResults {
        let query_lower = query.to_lowercase();

        // Search tasks by description
        let tasks = self.get_tasks(None);
        let matching_tasks: Vec<_> = tasks
            .into_iter()
            .filter(|t| t.description.to_lowercase().contains(&query_lower))
            .collect();

        SearchResults {
            query: query.to_string(),
            blocks: vec![],
            transactions: vec![],
            robots: vec![],
            spaces: vec![],
            tasks: matching_tasks,
        }
    }
}
