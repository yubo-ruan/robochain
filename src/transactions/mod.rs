//! Transaction types for RoboChain
//!
//! Each transaction represents a meaningful event in the robot lifecycle.
//! Transactions declare which objects they touch for parallel execution.

mod types;
mod validation;
mod effects;

pub use types::*;
pub use validation::*;
pub use effects::*;

use crate::crypto::{Hash, PublicKey, Signature};
use crate::objects::ObjectId;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A signed transaction on the chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Transaction payload
    pub payload: TransactionPayload,
    /// Sender's public key
    pub sender: PublicKey,
    /// Signature over the payload
    pub signature: Signature,
    /// Transaction hash (computed)
    pub hash: Hash,
    /// Timestamp when transaction was created
    pub timestamp: u64,
    /// Nonce for replay protection
    pub nonce: u64,
    /// Max gas willing to pay
    pub max_gas: u64,
}

impl Transaction {
    /// Create a new unsigned transaction
    pub fn new(payload: TransactionPayload, sender: PublicKey, timestamp: u64, nonce: u64) -> Self {
        let hash = Self::compute_hash(&payload, &sender, timestamp, nonce);
        Transaction {
            payload,
            sender,
            signature: Signature::default(),
            hash,
            timestamp,
            nonce,
            max_gas: 1_000_000, // Default max gas
        }
    }

    /// Compute transaction hash
    fn compute_hash(
        payload: &TransactionPayload,
        sender: &PublicKey,
        timestamp: u64,
        nonce: u64,
    ) -> Hash {
        let bytes = borsh::to_vec(&(payload, sender, timestamp, nonce)).unwrap_or_default();
        Hash::from_bytes(&bytes)
    }

    /// Sign the transaction
    pub fn sign(&mut self, keypair: &crate::crypto::KeyPair) {
        let message = self.signing_message();
        self.signature = keypair.sign(&message);
    }

    /// Get the message to be signed
    pub fn signing_message(&self) -> Vec<u8> {
        borsh::to_vec(&(&self.payload, &self.sender, self.timestamp, self.nonce)).unwrap_or_default()
    }

    /// Verify the signature
    pub fn verify_signature(&self) -> bool {
        let message = self.signing_message();
        crate::crypto::KeyPair::verify(&self.sender, &message, &self.signature)
    }

    /// Get all object IDs that this transaction reads
    pub fn read_set(&self) -> HashSet<ObjectId> {
        self.payload.read_set()
    }

    /// Get all object IDs that this transaction writes
    pub fn write_set(&self) -> HashSet<ObjectId> {
        self.payload.write_set()
    }

    /// Check if this transaction conflicts with another
    pub fn conflicts_with(&self, other: &Transaction) -> bool {
        // Two transactions conflict if one's write set intersects with the other's read or write set
        let my_writes = self.write_set();
        let other_reads = other.read_set();
        let other_writes = other.write_set();

        !my_writes.is_disjoint(&other_reads)
            || !my_writes.is_disjoint(&other_writes)
            || !self.read_set().is_disjoint(&other_writes)
    }
}

/// Wrapper for transaction results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionReceipt {
    /// Transaction hash
    pub tx_hash: Hash,
    /// Whether transaction succeeded
    pub success: bool,
    /// Gas used
    pub gas_used: u64,
    /// Block number where transaction was included
    pub block_number: u64,
    /// Error message if failed
    pub error: Option<String>,
    /// Events emitted
    pub events: Vec<TransactionEvent>,
}

/// Events emitted by transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionEvent {
    /// Event type
    pub event_type: String,
    /// Object IDs involved
    pub object_ids: Vec<ObjectId>,
    /// Event data
    pub data: Vec<u8>,
}
