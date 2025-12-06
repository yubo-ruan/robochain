//! API response types for the block explorer

use serde::Serialize;

/// Summary view of a block for list displays
#[derive(Debug, Clone, Serialize)]
pub struct BlockSummary {
    pub height: u64,
    pub hash: String,
    pub timestamp: u64,
    pub tx_count: usize,
    pub total_gas: u64,
    pub proposer: String,
}

/// Full block view with all details
#[derive(Debug, Clone, Serialize)]
pub struct BlockView {
    pub height: u64,
    pub hash: String,
    pub prev_hash: String,
    pub timestamp: u64,
    pub proposer: String,
    pub tx_count: usize,
    pub total_gas: u64,
    pub state_root: String,
    pub transactions: Vec<TransactionSummary>,
}

/// Summary view of a transaction
#[derive(Debug, Clone, Serialize)]
pub struct TransactionSummary {
    pub hash: String,
    pub tx_type: String,
    pub sender: String,
    pub timestamp: u64,
    pub gas_used: u64,
    pub success: bool,
}

/// Full transaction view
#[derive(Debug, Clone, Serialize)]
pub struct TransactionView {
    pub hash: String,
    pub tx_type: String,
    pub sender: String,
    pub timestamp: u64,
    pub nonce: u64,
    pub max_gas: u64,
    pub gas_used: u64,
    pub success: bool,
    pub block_height: Option<u64>,
    pub payload: serde_json::Value,
}

/// Robot view for explorer
#[derive(Debug, Clone, Serialize)]
pub struct RobotView {
    pub id: String,
    pub manufacturer: String,
    pub model: String,
    pub status: String,
    pub owner: Option<String>,
    pub capabilities: Vec<String>,
    pub registered_at: u64,
    pub last_active_at: u64,
    pub firmware_hash: String,
}

/// Space view for explorer
#[derive(Debug, Clone, Serialize)]
pub struct SpaceView {
    pub id: String,
    pub name: String,
    pub space_type: String,
    pub owners: Vec<String>,
    pub zone_count: usize,
    pub is_active: bool,
    pub created_at: u64,
    pub geo_region: GeoRegionView,
}

/// Geographic region view
#[derive(Debug, Clone, Serialize)]
pub struct GeoRegionView {
    pub country: String,
    pub region: Option<String>,
    pub city: Option<String>,
}

/// Task view for explorer
#[derive(Debug, Clone, Serialize)]
pub struct TaskView {
    pub id: String,
    pub space_id: String,
    pub requester: String,
    pub description: String,
    pub status: String,
    pub reward: u64,
    pub required_capabilities: Vec<String>,
    pub assigned_robot: Option<String>,
    pub created_at: u64,
    pub deadline: u64,
    pub accepted_at: Option<u64>,
    pub completed_at: Option<u64>,
}

/// Principal (user/entity) view
#[derive(Debug, Clone, Serialize)]
pub struct PrincipalView {
    pub id: String,
    pub display_name: String,
    pub principal_type: String,
    pub kyc_level: String,
    pub reputation_score: u64,
    pub is_active: bool,
    pub registered_at: u64,
    pub tasks_requested: u64,
}

/// Chain statistics
#[derive(Debug, Clone, Serialize)]
pub struct ChainStats {
    pub total_blocks: u64,
    pub total_transactions: u64,
    pub total_robots: u64,
    pub total_spaces: u64,
    pub total_tasks: u64,
    pub active_tasks: u64,
    pub total_principals: u64,
}

/// Search results
#[derive(Debug, Clone, Serialize)]
pub struct SearchResults {
    pub query: String,
    pub blocks: Vec<BlockSummary>,
    pub transactions: Vec<TransactionSummary>,
    pub robots: Vec<RobotView>,
    pub spaces: Vec<SpaceView>,
    pub tasks: Vec<TaskView>,
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
    pub has_more: bool,
}

/// API error response
#[derive(Debug, Clone, Serialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
}

impl ApiError {
    pub fn not_found(msg: &str) -> Self {
        ApiError {
            error: "not_found".to_string(),
            message: msg.to_string(),
        }
    }

    pub fn bad_request(msg: &str) -> Self {
        ApiError {
            error: "bad_request".to_string(),
            message: msg.to_string(),
        }
    }
}
