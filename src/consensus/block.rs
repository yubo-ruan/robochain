//! Block structure for RoboChain

use crate::crypto::{Hash, PublicKey, Signature};
use crate::transactions::Transaction;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

/// Block header
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct BlockHeader {
    /// Block height
    pub height: u64,
    /// Previous block hash
    pub prev_hash: Hash,
    /// Merkle root of transactions
    pub tx_root: Hash,
    /// State root after execution
    pub state_root: Hash,
    /// Off-chain commitments root
    pub offchain_root: Hash,
    /// Block timestamp
    pub timestamp: u64,
    /// Proposer's public key
    pub proposer: PublicKey,
    /// Proposer's signature
    pub signature: Signature,
}

impl BlockHeader {
    /// Compute the block hash
    pub fn hash(&self) -> Hash {
        let bytes = borsh::to_vec(self).unwrap_or_default();
        Hash::from_bytes(&bytes)
    }

    /// Get the signing message (everything except signature)
    pub fn signing_message(&self) -> Vec<u8> {
        let mut msg = Vec::new();
        msg.extend_from_slice(&self.height.to_le_bytes());
        msg.extend_from_slice(self.prev_hash.as_bytes());
        msg.extend_from_slice(self.tx_root.as_bytes());
        msg.extend_from_slice(self.state_root.as_bytes());
        msg.extend_from_slice(self.offchain_root.as_bytes());
        msg.extend_from_slice(&self.timestamp.to_le_bytes());
        msg.extend_from_slice(self.proposer.as_bytes());
        msg
    }

    /// Verify the proposer's signature
    pub fn verify_signature(&self) -> bool {
        let msg = self.signing_message();
        crate::crypto::KeyPair::verify(&self.proposer, &msg, &self.signature)
    }
}

/// Off-chain data commitment
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct OffchainCommitment {
    /// Dataset hash
    pub dataset_hash: Hash,
    /// Type of data
    pub data_type: String,
    /// Size in bytes
    pub size: u64,
}

/// A block in the RoboChain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Block header
    pub header: BlockHeader,
    /// Transactions in the block
    pub transactions: Vec<Transaction>,
    /// Off-chain data commitments
    pub offchain_commitments: Vec<OffchainCommitment>,
}

impl Block {
    /// Create a new block
    pub fn new(
        height: u64,
        prev_hash: Hash,
        transactions: Vec<Transaction>,
        state_root: Hash,
        proposer: PublicKey,
        timestamp: u64,
    ) -> Self {
        let tx_root = Self::compute_tx_root(&transactions);
        let offchain_commitments = Vec::new();
        let offchain_root = Hash::zero();

        Block {
            header: BlockHeader {
                height,
                prev_hash,
                tx_root,
                state_root,
                offchain_root,
                timestamp,
                proposer,
                signature: Signature::default(),
            },
            transactions,
            offchain_commitments,
        }
    }

    /// Compute merkle root of transactions
    fn compute_tx_root(transactions: &[Transaction]) -> Hash {
        if transactions.is_empty() {
            return Hash::zero();
        }

        // Simple hash of all transaction hashes
        let mut data = Vec::new();
        for tx in transactions {
            data.extend_from_slice(tx.hash.as_bytes());
        }
        Hash::from_bytes(&data)
    }

    /// Get block hash
    pub fn hash(&self) -> Hash {
        self.header.hash()
    }

    /// Get block height
    pub fn height(&self) -> u64 {
        self.header.height
    }

    /// Sign the block
    pub fn sign(&mut self, keypair: &crate::crypto::KeyPair) {
        let msg = self.header.signing_message();
        self.header.signature = keypair.sign(&msg);
    }

    /// Verify block signature
    pub fn verify_signature(&self) -> bool {
        self.header.verify_signature()
    }

    /// Add off-chain commitment
    pub fn add_offchain_commitment(&mut self, commitment: OffchainCommitment) {
        self.offchain_commitments.push(commitment);
        // Recompute offchain root
        self.header.offchain_root = self.compute_offchain_root();
    }

    fn compute_offchain_root(&self) -> Hash {
        if self.offchain_commitments.is_empty() {
            return Hash::zero();
        }

        let mut data = Vec::new();
        for c in &self.offchain_commitments {
            data.extend_from_slice(c.dataset_hash.as_bytes());
        }
        Hash::from_bytes(&data)
    }

    /// Get number of transactions
    pub fn tx_count(&self) -> usize {
        self.transactions.len()
    }

    /// Get total gas from all transactions
    pub fn total_gas(&self) -> u64 {
        self.transactions.iter().map(|tx| tx.max_gas).sum()
    }

    /// Validate block structure
    pub fn validate(&self) -> Result<(), BlockValidationError> {
        // Check transaction root
        let computed_tx_root = Self::compute_tx_root(&self.transactions);
        if computed_tx_root != self.header.tx_root {
            return Err(BlockValidationError::InvalidTxRoot);
        }

        // Check signature
        if !self.verify_signature() {
            return Err(BlockValidationError::InvalidSignature);
        }

        // Check all transactions have valid signatures
        for tx in &self.transactions {
            if !tx.verify_signature() {
                return Err(BlockValidationError::InvalidTxSignature);
            }
        }

        Ok(())
    }
}

/// Block validation errors
#[derive(Debug, Clone)]
pub enum BlockValidationError {
    InvalidTxRoot,
    InvalidSignature,
    InvalidTxSignature,
    InvalidStateRoot,
    InvalidHeight,
    InvalidTimestamp,
    BlockTooLarge,
    TooManyTransactions,
}

/// Genesis block
pub fn create_genesis_block(proposer: &crate::crypto::KeyPair) -> Block {
    let mut block = Block::new(
        0,
        Hash::zero(),
        Vec::new(),
        Hash::zero(),
        proposer.public_key(),
        0,
    );
    block.sign(proposer);
    block
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::KeyPair;

    #[test]
    fn test_genesis_block() {
        let keypair = KeyPair::generate();
        let genesis = create_genesis_block(&keypair);

        assert_eq!(genesis.height(), 0);
        assert_eq!(genesis.header.prev_hash, Hash::zero());
        assert!(genesis.verify_signature());
    }

    #[test]
    fn test_block_hash() {
        let keypair = KeyPair::generate();
        let block1 = create_genesis_block(&keypair);
        let block2 = create_genesis_block(&keypair);

        // Different blocks (different keypairs create different hashes)
        let keypair2 = KeyPair::generate();
        let block3 = create_genesis_block(&keypair2);

        assert_ne!(block1.hash(), block3.hash());
    }

    #[test]
    fn test_block_validation() {
        let keypair = KeyPair::generate();
        let block = create_genesis_block(&keypair);

        assert!(block.validate().is_ok());
    }
}
