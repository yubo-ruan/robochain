//! Consensus engine implementation

use super::{
    Block, BlockProposal, CommitCertificate, ConsensusConfig, ConsensusMessage,
    FinalityStatus, RoundInfo, Vote, Validator, ValidatorSet,
};
use crate::crypto::{Hash, KeyPair, PublicKey};
use crate::execution::ExecutionEngine;
use crate::transactions::Transaction;
use parking_lot::RwLock;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// State of the consensus engine
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsensusState {
    /// Waiting for transactions
    Idle,
    /// Proposing a block
    Proposing,
    /// Voting on a proposal
    Voting,
    /// Committing a block
    Committing,
    /// Syncing with peers
    Syncing,
}

/// Consensus engine for RoboChain
pub struct ConsensusEngine {
    /// Configuration
    config: ConsensusConfig,
    /// Validator set
    validators: ValidatorSet,
    /// Our keypair
    keypair: KeyPair,
    /// Current state
    state: ConsensusState,
    /// Current round
    current_round: u64,
    /// Current height
    current_height: u64,
    /// Pending transactions (mempool)
    mempool: VecDeque<Transaction>,
    /// Committed blocks
    blocks: Vec<Block>,
    /// Commit certificates
    certificates: HashMap<Hash, CommitCertificate>,
    /// Current round votes
    current_votes: Vec<Vote>,
    /// Current proposal
    current_proposal: Option<BlockProposal>,
    /// Execution engine
    execution: Arc<RwLock<ExecutionEngine>>,
    /// Last block time
    last_block_time: Instant,
}

impl ConsensusEngine {
    /// Create a new consensus engine
    pub fn new(
        config: ConsensusConfig,
        keypair: KeyPair,
        execution: Arc<RwLock<ExecutionEngine>>,
    ) -> Self {
        let validators = ValidatorSet::new(
            1_000_000_000, // 1000 ROBO minimum
            100,           // max 100 validators
        );

        ConsensusEngine {
            config,
            validators,
            keypair,
            state: ConsensusState::Idle,
            current_round: 0,
            current_height: 0,
            mempool: VecDeque::new(),
            blocks: Vec::new(),
            certificates: HashMap::new(),
            current_votes: Vec::new(),
            current_proposal: None,
            execution,
            last_block_time: Instant::now(),
        }
    }

    /// Initialize with genesis block
    pub fn initialize_genesis(&mut self) {
        let genesis = super::block::create_genesis_block(&self.keypair);
        self.blocks.push(genesis);
        self.current_height = 1;
    }

    /// Add a validator
    pub fn add_validator(&mut self, validator: Validator) -> Result<(), super::ValidatorError> {
        self.validators.add_validator(validator)
    }

    /// Submit a transaction to the mempool
    pub fn submit_transaction(&mut self, tx: Transaction) {
        // Basic validation
        if tx.verify_signature() {
            self.mempool.push_back(tx);
        }
    }

    /// Get mempool size
    pub fn mempool_size(&self) -> usize {
        self.mempool.len()
    }

    /// Check if we are the leader for current round
    pub fn is_leader(&self) -> bool {
        self.validators
            .select_leader(self.current_round)
            .is_some_and(|leader| leader == self.keypair.public_key())
    }

    /// Propose a new block (if we are leader)
    pub fn propose_block(&mut self) -> Option<BlockProposal> {
        if !self.is_leader() {
            return None;
        }

        // Collect transactions from mempool
        let mut transactions = Vec::new();
        let max_txs = self.config.max_txs_per_block.min(self.mempool.len());

        for _ in 0..max_txs {
            if let Some(tx) = self.mempool.pop_front() {
                transactions.push(tx);
            }
        }

        // Get previous block hash
        let prev_hash = self
            .blocks
            .last()
            .map(|b| b.hash())
            .unwrap_or_else(Hash::zero);

        // Create block
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut block = Block::new(
            self.current_height,
            prev_hash,
            transactions,
            Hash::zero(), // State root computed after execution
            self.keypair.public_key(),
            timestamp,
        );

        // Execute transactions to get state root
        {
            let mut exec = self.execution.write();
            let result = exec.execute_block(block.transactions.clone());
            block.header.state_root = result.state_root;
        }

        block.sign(&self.keypair);

        let proposal = BlockProposal {
            block: block.clone(),
            signature: self.keypair.sign(block.hash().as_bytes()),
            round: self.current_round,
        };

        self.current_proposal = Some(proposal.clone());
        self.state = ConsensusState::Proposing;

        Some(proposal)
    }

    /// Handle a received proposal
    pub fn handle_proposal(&mut self, proposal: BlockProposal) -> Option<Vote> {
        // Validate proposal
        if !proposal.block.verify_signature() {
            return None;
        }

        // Check proposal is from expected leader
        let expected_leader = self.validators.select_leader(proposal.round)?;
        if expected_leader != proposal.block.header.proposer {
            return None;
        }

        // Check height matches
        if proposal.block.height() != self.current_height {
            return None;
        }

        // Store proposal
        self.current_proposal = Some(proposal.clone());
        self.state = ConsensusState::Voting;

        // Vote for the proposal
        let vote = self.create_vote(&proposal.block);
        self.current_votes.push(vote.clone());

        Some(vote)
    }

    /// Create a vote for a block
    fn create_vote(&self, block: &Block) -> Vote {
        let msg = block.hash();
        let signature = self.keypair.sign(msg.as_bytes());

        Vote::new(
            block.hash(),
            self.current_round,
            self.keypair.public_key(),
            signature,
        )
    }

    /// Handle a received vote
    pub fn handle_vote(&mut self, vote: Vote) -> Option<CommitCertificate> {
        // Validate vote
        if !vote.verify() {
            return None;
        }

        // Check vote is for current round
        if vote.round != self.current_round {
            return None;
        }

        // Check vote is from known validator
        if self.validators.get(&vote.voter).is_none() {
            return None;
        }

        // Add vote
        self.current_votes.push(vote);

        // Check if we have quorum
        let voters: Vec<_> = self.current_votes.iter().map(|v| v.voter).collect();
        if self.validators.has_quorum(&voters, self.config.quorum_threshold) {
            return self.commit_block();
        }

        None
    }

    /// Commit the current block
    fn commit_block(&mut self) -> Option<CommitCertificate> {
        let proposal = self.current_proposal.take()?;
        let block = proposal.block;

        // Create commit certificate
        let certificate = CommitCertificate {
            block_hash: block.hash(),
            height: block.height(),
            votes: self.current_votes.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        // Store block and certificate
        self.blocks.push(block);
        self.certificates.insert(certificate.block_hash, certificate.clone());

        // Move to next round
        self.current_height += 1;
        self.current_round += 1;
        self.current_votes.clear();
        self.state = ConsensusState::Idle;
        self.last_block_time = Instant::now();

        Some(certificate)
    }

    /// Process consensus tick (called periodically)
    pub fn tick(&mut self) -> Option<ConsensusMessage> {
        let elapsed = self.last_block_time.elapsed();
        let target_duration = Duration::from_millis(self.config.block_time_ms);

        match self.state {
            ConsensusState::Idle => {
                // Check if it's time for a new block
                if elapsed >= target_duration && !self.mempool.is_empty() {
                    if self.is_leader() {
                        if let Some(proposal) = self.propose_block() {
                            return Some(ConsensusMessage::Proposal(proposal));
                        }
                    }
                }
                None
            }
            ConsensusState::Voting => {
                // Check for timeout
                if elapsed >= target_duration * 3 {
                    // Timeout, move to next round
                    self.current_round += 1;
                    self.current_votes.clear();
                    self.current_proposal = None;
                    self.state = ConsensusState::Idle;
                }
                None
            }
            _ => None,
        }
    }

    /// Get current round info
    pub fn round_info(&self) -> RoundInfo {
        RoundInfo {
            round: self.current_round,
            height: self.current_height,
            leader: self
                .validators
                .select_leader(self.current_round)
                .unwrap_or_else(|| self.keypair.public_key()),
            started_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            votes: self.current_votes.clone(),
        }
    }

    /// Get finality status of a block
    pub fn finality_status(&self, block_hash: &Hash) -> FinalityStatus {
        if let Some(cert) = self.certificates.get(block_hash) {
            if cert.verify(&self.validators.all().into_iter().cloned().collect::<Vec<_>>(), self.config.quorum_threshold) {
                return FinalityStatus::Finalized;
            }
            return FinalityStatus::SoftFinality;
        }
        FinalityStatus::Pending
    }

    /// Get block by height
    pub fn get_block(&self, height: u64) -> Option<&Block> {
        self.blocks.get(height as usize)
    }

    /// Get latest block
    pub fn latest_block(&self) -> Option<&Block> {
        self.blocks.last()
    }

    /// Get current height
    pub fn height(&self) -> u64 {
        self.current_height
    }

    /// Get state
    pub fn state(&self) -> &ConsensusState {
        &self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::ExecutionConfig;

    fn create_test_engine() -> ConsensusEngine {
        let keypair = KeyPair::generate();
        let config = ConsensusConfig::default();
        let exec_config = ExecutionConfig::default();
        let execution = Arc::new(RwLock::new(ExecutionEngine::new(exec_config)));

        ConsensusEngine::new(config, keypair, execution)
    }

    #[test]
    fn test_engine_creation() {
        let engine = create_test_engine();
        assert_eq!(*engine.state(), ConsensusState::Idle);
        assert_eq!(engine.height(), 0);
    }

    #[test]
    fn test_genesis_initialization() {
        let mut engine = create_test_engine();
        engine.initialize_genesis();

        assert_eq!(engine.height(), 1);
        assert!(engine.latest_block().is_some());
    }

    #[test]
    fn test_transaction_submission() {
        let mut engine = create_test_engine();

        let keypair = KeyPair::generate();
        let payload = crate::transactions::TransactionPayload::RegisterPrincipal(
            crate::transactions::RegisterPrincipalTx {
                display_name: "Test".to_string(),
                principal_type: crate::objects::PrincipalType::Owner,
            },
        );

        let mut tx = crate::transactions::Transaction::new(
            payload,
            keypair.public_key(),
            1000,
            1,
        );
        tx.sign(&keypair);

        engine.submit_transaction(tx);
        assert_eq!(engine.mempool_size(), 1);
    }
}
