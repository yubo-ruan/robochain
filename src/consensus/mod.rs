//! Consensus Layer for RoboChain
//!
//! Implements a BFT-style consensus optimized for:
//! - Sub-second block times (target: 500ms)
//! - High throughput for small structured messages
//! - Strong finality for capability revocations
//! - Graceful handling of partitioned robots

mod block;
mod validator;
mod engine;
mod offline;

pub use block::*;
pub use validator::*;
pub use engine::*;
pub use offline::*;

use crate::crypto::{Hash, PublicKey, Signature};
use serde::{Deserialize, Serialize};

/// Consensus configuration
#[derive(Debug, Clone)]
pub struct ConsensusConfig {
    /// Target block time in milliseconds
    pub block_time_ms: u64,
    /// Minimum validators required for consensus
    pub min_validators: usize,
    /// Quorum threshold (e.g., 2/3)
    pub quorum_threshold: f64,
    /// Maximum transactions per block
    pub max_txs_per_block: usize,
    /// Maximum block size in bytes
    pub max_block_size: usize,
    /// Finality depth (blocks)
    pub finality_depth: u64,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        ConsensusConfig {
            block_time_ms: 500,      // 500ms target
            min_validators: 4,
            quorum_threshold: 0.67,  // 2/3 + 1
            max_txs_per_block: 10000,
            max_block_size: 10 * 1024 * 1024, // 10MB
            finality_depth: 2,
        }
    }
}

/// Consensus round information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundInfo {
    /// Current round number
    pub round: u64,
    /// Current block height
    pub height: u64,
    /// Round leader (proposer)
    pub leader: PublicKey,
    /// Round start time
    pub started_at: u64,
    /// Votes collected
    pub votes: Vec<Vote>,
}

/// Vote for a block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    /// Block hash being voted on
    pub block_hash: Hash,
    /// Round number
    pub round: u64,
    /// Voter's public key
    pub voter: PublicKey,
    /// Vote signature
    pub signature: Signature,
    /// Vote timestamp
    pub timestamp: u64,
}

impl Vote {
    pub fn new(block_hash: Hash, round: u64, voter: PublicKey, signature: Signature) -> Self {
        Vote {
            block_hash,
            round,
            voter,
            signature,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Verify the vote signature
    pub fn verify(&self) -> bool {
        let message = self.signing_message();
        crate::crypto::KeyPair::verify(&self.voter, &message, &self.signature)
    }

    fn signing_message(&self) -> Vec<u8> {
        let mut msg = Vec::new();
        msg.extend_from_slice(self.block_hash.as_bytes());
        msg.extend_from_slice(&self.round.to_le_bytes());
        msg.extend_from_slice(self.voter.as_bytes());
        msg
    }
}

/// Consensus message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusMessage {
    /// Proposal for a new block
    Proposal(BlockProposal),
    /// Vote for a proposed block
    Vote(Vote),
    /// Commit notification (quorum reached)
    Commit(CommitCertificate),
    /// Request for sync
    SyncRequest(SyncRequest),
    /// Sync response with blocks
    SyncResponse(SyncResponse),
}

/// Block proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockProposal {
    /// Proposed block
    pub block: Block,
    /// Proposer signature
    pub signature: Signature,
    /// Round number
    pub round: u64,
}

/// Commit certificate (proof of finality)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitCertificate {
    /// Block hash
    pub block_hash: Hash,
    /// Block height
    pub height: u64,
    /// Aggregated votes
    pub votes: Vec<Vote>,
    /// Certificate timestamp
    pub timestamp: u64,
}

impl CommitCertificate {
    /// Verify the certificate has enough valid votes
    pub fn verify(&self, validators: &[Validator], quorum_threshold: f64) -> bool {
        // Check we have enough votes
        let required_votes = (validators.len() as f64 * quorum_threshold).ceil() as usize;
        if self.votes.len() < required_votes {
            return false;
        }

        // Verify each vote
        for vote in &self.votes {
            if !vote.verify() {
                return false;
            }
            if vote.block_hash != self.block_hash {
                return false;
            }
        }

        // Check votes are from known validators
        let validator_keys: std::collections::HashSet<_> =
            validators.iter().map(|v| v.public_key).collect();

        for vote in &self.votes {
            if !validator_keys.contains(&vote.voter) {
                return false;
            }
        }

        true
    }
}

/// Sync request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequest {
    /// Requester's current height
    pub from_height: u64,
    /// Target height (0 = latest)
    pub to_height: u64,
    /// Requester's public key
    pub requester: PublicKey,
}

/// Sync response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    /// Blocks to sync
    pub blocks: Vec<Block>,
    /// Commit certificates for each block
    pub certificates: Vec<CommitCertificate>,
    /// Responder's current height
    pub current_height: u64,
}

/// Finality status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FinalityStatus {
    /// Block is pending (not yet finalized)
    Pending,
    /// Block has soft finality (enough votes but not committed)
    SoftFinality,
    /// Block is fully finalized
    Finalized,
}
