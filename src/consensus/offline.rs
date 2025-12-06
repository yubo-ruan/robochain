//! Offline Reconciliation Protocol for RoboChain
//!
//! Handles robots that operate offline and need to reconcile
//! their local logs when connectivity is restored.

use crate::crypto::{Hash, KeyPair, PublicKey, Signature};
use crate::objects::RobotId;
use crate::transactions::Transaction;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An event recorded by a robot while offline
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct OfflineEvent {
    /// Sequence number (monotonically increasing per robot)
    pub sequence: u64,
    /// Robot that generated this event
    pub robot_id: RobotId,
    /// Event timestamp (from robot's clock)
    pub timestamp: u64,
    /// Event type
    pub event_type: OfflineEventType,
    /// Event data
    pub data: Vec<u8>,
    /// Robot's signature over the event
    pub signature: Signature,
}

/// Types of offline events
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum OfflineEventType {
    /// Task was started
    TaskStarted { task_id_hash: Hash },
    /// Task progress checkpoint
    TaskProgress { task_id_hash: Hash, progress: u8 },
    /// Task was completed
    TaskCompleted { task_id_hash: Hash, evidence_hash: Hash },
    /// Safety event occurred
    SafetyEvent { severity: u8, description: String },
    /// Sensor reading
    SensorReading { sensor_type: String, value: Vec<u8> },
    /// Location update
    LocationUpdate { zone_id: String },
    /// Custom event
    Custom { event_name: String },
}

impl OfflineEvent {
    /// Create a new offline event
    pub fn new(
        robot_id: RobotId,
        sequence: u64,
        event_type: OfflineEventType,
        data: Vec<u8>,
        keypair: &KeyPair,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut event = OfflineEvent {
            sequence,
            robot_id,
            timestamp,
            event_type,
            data,
            signature: Signature::default(),
        };

        event.sign(keypair);
        event
    }

    /// Sign the event
    pub fn sign(&mut self, keypair: &KeyPair) {
        let msg = self.signing_message();
        self.signature = keypair.sign(&msg);
    }

    /// Get the signing message
    fn signing_message(&self) -> Vec<u8> {
        let mut msg = Vec::new();
        msg.extend_from_slice(&self.sequence.to_le_bytes());
        msg.extend_from_slice(&self.timestamp.to_le_bytes());
        // Add event type hash
        let type_bytes = borsh::to_vec(&self.event_type).unwrap_or_default();
        msg.extend_from_slice(&type_bytes);
        msg.extend_from_slice(&self.data);
        msg
    }

    /// Verify the signature
    pub fn verify(&self, robot_pubkey: &PublicKey) -> bool {
        let msg = self.signing_message();
        KeyPair::verify(robot_pubkey, &msg, &self.signature)
    }
}

/// A batch of offline events for reconciliation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineEventBatch {
    /// Robot ID
    pub robot_id: RobotId,
    /// Robot's secure element public key
    pub robot_pubkey: PublicKey,
    /// Events in the batch
    pub events: Vec<OfflineEvent>,
    /// Last known on-chain sequence for this robot
    pub last_on_chain_sequence: u64,
    /// Batch merkle root
    pub batch_root: Hash,
    /// Timestamp proof (e.g., from trusted timestamping service)
    pub timestamp_proof: Option<TimestampProof>,
}

/// Proof of timestamp from trusted service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampProof {
    /// Service that provided the timestamp
    pub service: String,
    /// The timestamped hash
    pub hash: Hash,
    /// Timestamp
    pub timestamp: u64,
    /// Service's signature
    pub signature: Vec<u8>,
}

impl OfflineEventBatch {
    /// Create a new batch
    pub fn new(robot_id: RobotId, robot_pubkey: PublicKey, events: Vec<OfflineEvent>) -> Self {
        let batch_root = Self::compute_batch_root(&events);
        let last_on_chain_sequence = 0; // Fetched from chain

        OfflineEventBatch {
            robot_id,
            robot_pubkey,
            events,
            last_on_chain_sequence,
            batch_root,
            timestamp_proof: None,
        }
    }

    /// Compute merkle root of events
    fn compute_batch_root(events: &[OfflineEvent]) -> Hash {
        if events.is_empty() {
            return Hash::zero();
        }

        let mut data = Vec::new();
        for event in events {
            let event_bytes = borsh::to_vec(event).unwrap_or_default();
            data.extend_from_slice(&Hash::from_bytes(&event_bytes).0);
        }
        Hash::from_bytes(&data)
    }

    /// Validate the batch
    pub fn validate(&self) -> Result<(), ReconciliationError> {
        // Check events are ordered by sequence
        let mut last_seq = self.last_on_chain_sequence;
        for event in &self.events {
            if event.sequence != last_seq + 1 {
                return Err(ReconciliationError::SequenceGap {
                    expected: last_seq + 1,
                    got: event.sequence,
                });
            }
            last_seq = event.sequence;

            // Verify signature
            if !event.verify(&self.robot_pubkey) {
                return Err(ReconciliationError::InvalidSignature);
            }

            // Check robot ID matches
            if event.robot_id != self.robot_id {
                return Err(ReconciliationError::RobotMismatch);
            }
        }

        // Verify batch root
        let computed_root = Self::compute_batch_root(&self.events);
        if computed_root != self.batch_root {
            return Err(ReconciliationError::InvalidBatchRoot);
        }

        Ok(())
    }

    /// Add timestamp proof
    pub fn set_timestamp_proof(&mut self, proof: TimestampProof) {
        self.timestamp_proof = Some(proof);
    }
}

/// Reconciliation errors
#[derive(Debug, Clone)]
pub enum ReconciliationError {
    SequenceGap { expected: u64, got: u64 },
    InvalidSignature,
    RobotMismatch,
    InvalidBatchRoot,
    EventTooOld { max_age_secs: u64 },
    InvalidTimestampProof,
    ConflictingEvents,
}

/// Reconciliation result
#[derive(Debug, Clone)]
pub struct ReconciliationResult {
    /// Number of events processed
    pub events_processed: usize,
    /// Number of events rejected
    pub events_rejected: usize,
    /// Transactions generated from events
    pub transactions: Vec<Transaction>,
    /// Updated robot sequence
    pub new_sequence: u64,
    /// Any disputes that need resolution
    pub disputes: Vec<ReconciliationDispute>,
}

/// A dispute during reconciliation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationDispute {
    /// Robot involved
    pub robot_id: RobotId,
    /// Type of dispute
    pub dispute_type: DisputeType,
    /// Evidence hashes
    pub evidence: Vec<Hash>,
    /// Description
    pub description: String,
}

/// Types of disputes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisputeType {
    /// Events conflict with on-chain state
    StateConflict,
    /// Time-based constraint violated
    TimeoutViolation,
    /// Safety constraint was violated
    SafetyViolation,
    /// Payment dispute
    PaymentDispute,
}

/// Offline reconciliation manager
pub struct OfflineReconciler {
    /// Maximum age of events (in seconds)
    max_event_age: u64,
    /// Last known sequence per robot
    robot_sequences: HashMap<RobotId, u64>,
}

impl OfflineReconciler {
    pub fn new(max_event_age: u64) -> Self {
        OfflineReconciler {
            max_event_age,
            robot_sequences: HashMap::new(),
        }
    }

    /// Process a batch of offline events
    pub fn process_batch(
        &mut self,
        batch: &OfflineEventBatch,
        current_time: u64,
    ) -> Result<ReconciliationResult, ReconciliationError> {
        // Validate batch
        batch.validate()?;

        // Check events are not too old
        for event in &batch.events {
            if current_time > event.timestamp + self.max_event_age {
                return Err(ReconciliationError::EventTooOld {
                    max_age_secs: self.max_event_age,
                });
            }
        }

        // Check sequence continuity with on-chain state
        let last_known_seq = self
            .robot_sequences
            .get(&batch.robot_id)
            .copied()
            .unwrap_or(0);

        if !batch.events.is_empty() && batch.events[0].sequence != last_known_seq + 1 {
            return Err(ReconciliationError::SequenceGap {
                expected: last_known_seq + 1,
                got: batch.events[0].sequence,
            });
        }

        // Process events and generate transactions
        let mut transactions = Vec::new();
        let mut disputes = Vec::new();
        let mut processed = 0;
        let mut rejected = 0;

        for event in &batch.events {
            match self.process_event(event) {
                Ok(Some(tx)) => {
                    transactions.push(tx);
                    processed += 1;
                }
                Ok(None) => {
                    processed += 1;
                }
                Err(e) => {
                    rejected += 1;
                    disputes.push(ReconciliationDispute {
                        robot_id: batch.robot_id,
                        dispute_type: DisputeType::StateConflict,
                        evidence: vec![Hash::from_bytes(
                            &borsh::to_vec(event).unwrap_or_default(),
                        )],
                        description: format!("{:?}", e),
                    });
                }
            }
        }

        // Update robot sequence
        let new_sequence = batch
            .events
            .last()
            .map(|e| e.sequence)
            .unwrap_or(last_known_seq);
        self.robot_sequences.insert(batch.robot_id, new_sequence);

        Ok(ReconciliationResult {
            events_processed: processed,
            events_rejected: rejected,
            transactions,
            new_sequence,
            disputes,
        })
    }

    /// Process a single event
    fn process_event(&self, event: &OfflineEvent) -> Result<Option<Transaction>, ReconciliationError> {
        // Convert event to transaction based on type
        match &event.event_type {
            OfflineEventType::TaskCompleted { task_id_hash, evidence_hash } => {
                // Would create a SubmitCompletion transaction
                // For now, just validate
                Ok(None)
            }
            OfflineEventType::SafetyEvent { severity, description } => {
                if *severity > 5 {
                    // Serious safety event, flag for review
                    return Err(ReconciliationError::ConflictingEvents);
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    /// Get the last known sequence for a robot
    pub fn get_sequence(&self, robot_id: &RobotId) -> u64 {
        self.robot_sequences.get(robot_id).copied().unwrap_or(0)
    }

    /// Set the sequence for a robot (from on-chain state)
    pub fn set_sequence(&mut self, robot_id: RobotId, sequence: u64) {
        self.robot_sequences.insert(robot_id, sequence);
    }
}

/// Local event log for a robot
pub struct LocalEventLog {
    /// Robot ID
    robot_id: RobotId,
    /// Robot keypair
    keypair: KeyPair,
    /// Events
    events: Vec<OfflineEvent>,
    /// Current sequence
    current_sequence: u64,
}

impl LocalEventLog {
    pub fn new(robot_id: RobotId, keypair: KeyPair, start_sequence: u64) -> Self {
        LocalEventLog {
            robot_id,
            keypair,
            events: Vec::new(),
            current_sequence: start_sequence,
        }
    }

    /// Record a new event
    pub fn record(&mut self, event_type: OfflineEventType, data: Vec<u8>) -> &OfflineEvent {
        self.current_sequence += 1;
        let event = OfflineEvent::new(
            self.robot_id,
            self.current_sequence,
            event_type,
            data,
            &self.keypair,
        );
        self.events.push(event);
        self.events.last().unwrap()
    }

    /// Create a batch for reconciliation
    pub fn create_batch(&self, from_sequence: u64) -> OfflineEventBatch {
        let events: Vec<_> = self
            .events
            .iter()
            .filter(|e| e.sequence > from_sequence)
            .cloned()
            .collect();

        OfflineEventBatch::new(self.robot_id, self.keypair.public_key(), events)
    }

    /// Clear events up to sequence (after successful reconciliation)
    pub fn clear_until(&mut self, sequence: u64) {
        self.events.retain(|e| e.sequence > sequence);
    }

    /// Get event count
    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_robot() -> (RobotId, KeyPair) {
        let keypair = KeyPair::generate();
        let robot_id = RobotId::from_bytes(keypair.public_key().as_bytes());
        (robot_id, keypair)
    }

    #[test]
    fn test_local_event_log() {
        let (robot_id, keypair) = create_test_robot();
        let mut log = LocalEventLog::new(robot_id, keypair, 0);

        log.record(
            OfflineEventType::TaskStarted {
                task_id_hash: Hash::from_bytes(b"task1"),
            },
            vec![],
        );

        log.record(
            OfflineEventType::TaskProgress {
                task_id_hash: Hash::from_bytes(b"task1"),
                progress: 50,
            },
            vec![],
        );

        assert_eq!(log.event_count(), 2);
    }

    #[test]
    fn test_batch_validation() {
        let (robot_id, keypair) = create_test_robot();
        let mut log = LocalEventLog::new(robot_id, keypair, 0);

        log.record(
            OfflineEventType::TaskStarted {
                task_id_hash: Hash::from_bytes(b"task1"),
            },
            vec![],
        );

        log.record(
            OfflineEventType::TaskCompleted {
                task_id_hash: Hash::from_bytes(b"task1"),
                evidence_hash: Hash::from_bytes(b"evidence"),
            },
            vec![],
        );

        let batch = log.create_batch(0);
        assert!(batch.validate().is_ok());
    }

    #[test]
    fn test_reconciliation() {
        let (robot_id, keypair) = create_test_robot();
        let mut log = LocalEventLog::new(robot_id, keypair, 0);

        log.record(
            OfflineEventType::LocationUpdate {
                zone_id: "living_room".to_string(),
            },
            vec![],
        );

        let batch = log.create_batch(0);

        let mut reconciler = OfflineReconciler::new(3600); // 1 hour max age
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let result = reconciler.process_batch(&batch, current_time).unwrap();
        assert_eq!(result.events_processed, 1);
        assert_eq!(result.new_sequence, 1);
    }
}
