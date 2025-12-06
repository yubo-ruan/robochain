//! Transaction execution and state effects

use super::{Transaction, TransactionPayload, TransactionEvent, TransactionReceipt};
use crate::crypto::Hash;
use crate::objects::*;
use crate::storage::{ObjectStore, StorageError};
use std::collections::HashSet;
use thiserror::Error;

/// Errors during transaction execution
#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Object not found: {0}")]
    ObjectNotFound(ObjectId),

    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
}

/// Result of executing a transaction
pub struct ExecutionResult {
    /// Whether execution succeeded
    pub success: bool,
    /// Gas consumed
    pub gas_used: u64,
    /// Objects created
    pub created: Vec<AnyObject>,
    /// Object IDs modified
    pub modified: Vec<ObjectId>,
    /// Events emitted
    pub events: Vec<TransactionEvent>,
    /// Error if failed
    pub error: Option<String>,
}

/// Transaction executor
pub struct TransactionExecutor;

impl TransactionExecutor {
    /// Execute a transaction and return effects
    pub fn execute(
        tx: &Transaction,
        store: &mut dyn ObjectStore,
        block_number: u64,
    ) -> ExecutionResult {
        let timestamp = tx.timestamp;

        match Self::execute_payload(&tx.payload, &tx.sender, store, timestamp) {
            Ok((created, modified, events, gas)) => ExecutionResult {
                success: true,
                gas_used: gas,
                created,
                modified,
                events,
                error: None,
            },
            Err(e) => ExecutionResult {
                success: false,
                gas_used: 10_000, // Base gas for failed tx
                created: vec![],
                modified: vec![],
                events: vec![],
                error: Some(e.to_string()),
            },
        }
    }

    fn execute_payload(
        payload: &TransactionPayload,
        sender: &crate::crypto::PublicKey,
        store: &mut dyn ObjectStore,
        timestamp: u64,
    ) -> Result<(Vec<AnyObject>, Vec<ObjectId>, Vec<TransactionEvent>, u64), ExecutionError> {
        let sender_id = PrincipalId::new(Hash::from_bytes(sender.as_bytes()));

        match payload {
            TransactionPayload::RegisterRobot(tx) => {
                let capabilities: HashSet<Capability> = tx.capabilities.iter().cloned().collect();
                let robot = Robot::new(
                    tx.manufacturer.clone(),
                    tx.model.clone(),
                    tx.hw_attestation.clone(),
                    capabilities,
                    timestamp,
                );

                let robot_id = robot.id;
                store.put_robot(robot.clone())?;

                let event = TransactionEvent {
                    event_type: "RobotRegistered".to_string(),
                    object_ids: vec![robot_id.as_object_id()],
                    data: vec![],
                };

                Ok((vec![AnyObject::Robot(robot)], vec![], vec![event], 50_000))
            }

            TransactionPayload::UpdateFirmware(tx) => {
                let mut robot = store.get_robot(&tx.robot_id)?;
                robot.update_firmware(tx.new_firmware_hash);
                store.put_robot(robot)?;

                let event = TransactionEvent {
                    event_type: "FirmwareUpdated".to_string(),
                    object_ids: vec![tx.robot_id.as_object_id()],
                    data: tx.new_firmware_hash.as_bytes().to_vec(),
                };

                Ok((vec![], vec![tx.robot_id.as_object_id()], vec![event], 30_000))
            }

            TransactionPayload::TransferRobot(tx) => {
                let mut robot = store.get_robot(&tx.robot_id)?;
                robot.set_owner(tx.new_owner);
                store.put_robot(robot)?;

                let event = TransactionEvent {
                    event_type: "RobotTransferred".to_string(),
                    object_ids: vec![tx.robot_id.as_object_id(), tx.new_owner.as_object_id()],
                    data: vec![],
                };

                Ok((vec![], vec![tx.robot_id.as_object_id()], vec![event], 25_000))
            }

            TransactionPayload::RegisterSpace(tx) => {
                let space = Space::new(
                    tx.name.clone(),
                    tx.space_type.clone(),
                    sender_id,
                    tx.geo_region.clone(),
                    timestamp,
                );

                let space_id = space.id;
                store.put_space(space.clone())?;

                let event = TransactionEvent {
                    event_type: "SpaceRegistered".to_string(),
                    object_ids: vec![space_id.as_object_id()],
                    data: vec![],
                };

                Ok((vec![AnyObject::Space(space)], vec![], vec![event], 40_000))
            }

            TransactionPayload::SetPolicy(tx) => {
                let mut space = store.get_space(&tx.space_id)?;
                space.set_policy_graph(tx.policy_graph_hash, timestamp);
                store.put_space(space)?;

                let event = TransactionEvent {
                    event_type: "PolicyUpdated".to_string(),
                    object_ids: vec![tx.space_id.as_object_id()],
                    data: tx.policy_graph_hash.as_bytes().to_vec(),
                };

                Ok((vec![], vec![tx.space_id.as_object_id()], vec![event], 20_000))
            }

            TransactionPayload::AddZone(tx) => {
                let mut space = store.get_space(&tx.space_id)?;
                space.add_zone(tx.zone.clone());
                store.put_space(space)?;

                let event = TransactionEvent {
                    event_type: "ZoneAdded".to_string(),
                    object_ids: vec![tx.space_id.as_object_id()],
                    data: vec![],
                };

                Ok((vec![], vec![tx.space_id.as_object_id()], vec![event], 20_000))
            }

            TransactionPayload::RegisterPrincipal(tx) => {
                let principal = Principal::new(
                    tx.display_name.clone(),
                    tx.principal_type.clone(),
                    *sender,
                    timestamp,
                );

                let principal_id = principal.id;
                store.put_principal(principal.clone())?;

                let event = TransactionEvent {
                    event_type: "PrincipalRegistered".to_string(),
                    object_ids: vec![principal_id.as_object_id()],
                    data: vec![],
                };

                Ok((vec![AnyObject::Principal(principal)], vec![], vec![event], 30_000))
            }

            TransactionPayload::GrantCapability(tx) => {
                let capabilities: HashSet<Capability> = tx.capabilities.iter().cloned().collect();

                let mut policy = Policy::new(
                    tx.space_id,
                    sender_id,
                    Some(tx.robot_id),
                    None,
                    capabilities,
                    PolicyGrantType::Allow,
                    tx.revocable,
                    timestamp,
                    tx.expires_at,
                );

                policy.set_zones(tx.zones.clone());
                policy.set_conditions(tx.conditions.clone());

                let policy_id = policy.id;
                store.put_policy(policy.clone())?;

                let event = TransactionEvent {
                    event_type: "CapabilityGranted".to_string(),
                    object_ids: vec![
                        policy_id.as_object_id(),
                        tx.robot_id.as_object_id(),
                        tx.space_id.as_object_id(),
                    ],
                    data: vec![],
                };

                Ok((vec![AnyObject::Policy(policy)], vec![], vec![event], 35_000))
            }

            TransactionPayload::RevokeCapability(tx) => {
                let mut policy = store.get_policy(&tx.policy_id)?;
                policy.revoke(timestamp).map_err(|e| {
                    ExecutionError::InvalidStateTransition(format!("{:?}", e))
                })?;
                store.put_policy(policy)?;

                let event = TransactionEvent {
                    event_type: "CapabilityRevoked".to_string(),
                    object_ids: vec![tx.policy_id.as_object_id()],
                    data: vec![],
                };

                Ok((vec![], vec![tx.policy_id.as_object_id()], vec![event], 20_000))
            }

            TransactionPayload::PostTask(tx) => {
                let required_capabilities: HashSet<Capability> =
                    tx.required_capabilities.iter().cloned().collect();

                let mut task = Task::new(
                    tx.space_id,
                    sender_id,
                    tx.description.clone(),
                    required_capabilities,
                    tx.reward,
                    tx.deadline,
                    timestamp,
                );

                for constraint in &tx.safety_constraints {
                    task.add_constraint(constraint.clone());
                }

                let task_id = task.id;
                store.put_task(task.clone())?;

                let event = TransactionEvent {
                    event_type: "TaskPosted".to_string(),
                    object_ids: vec![task_id.as_object_id(), tx.space_id.as_object_id()],
                    data: vec![],
                };

                Ok((vec![AnyObject::Task(task)], vec![], vec![event], 40_000))
            }

            TransactionPayload::AcceptTask(tx) => {
                let mut task = store.get_task(&tx.task_id)?;
                task.accept(tx.robot_id, timestamp).map_err(|e| {
                    ExecutionError::InvalidStateTransition(format!("{:?}", e))
                })?;
                store.put_task(task)?;

                let event = TransactionEvent {
                    event_type: "TaskAccepted".to_string(),
                    object_ids: vec![tx.task_id.as_object_id(), tx.robot_id.as_object_id()],
                    data: vec![],
                };

                Ok((vec![], vec![tx.task_id.as_object_id()], vec![event], 25_000))
            }

            TransactionPayload::StartTask(tx) => {
                let mut task = store.get_task(&tx.task_id)?;
                task.start().map_err(|e| {
                    ExecutionError::InvalidStateTransition(format!("{:?}", e))
                })?;
                store.put_task(task)?;

                let event = TransactionEvent {
                    event_type: "TaskStarted".to_string(),
                    object_ids: vec![tx.task_id.as_object_id()],
                    data: vec![],
                };

                Ok((vec![], vec![tx.task_id.as_object_id()], vec![event], 20_000))
            }

            TransactionPayload::SubmitCompletion(tx) => {
                let mut task = store.get_task(&tx.task_id)?;

                // Create dataset for evidence
                let robot_id = task.assigned_robot.ok_or_else(|| {
                    ExecutionError::InvalidStateTransition("No robot assigned".to_string())
                })?;

                let dataset = Dataset::new(
                    DatasetType::TaskEvidence,
                    robot_id,
                    sender_id,
                    tx.evidence_uri.clone(),
                    tx.evidence_merkle_root,
                    0, // Size unknown at submission
                    timestamp,
                );

                let dataset_id = dataset.id;
                store.put_dataset(dataset.clone())?;

                task.complete(dataset_id, timestamp).map_err(|e| {
                    ExecutionError::InvalidStateTransition(format!("{:?}", e))
                })?;
                store.put_task(task)?;

                let event = TransactionEvent {
                    event_type: "TaskCompleted".to_string(),
                    object_ids: vec![tx.task_id.as_object_id(), dataset_id.as_object_id()],
                    data: vec![],
                };

                Ok((
                    vec![AnyObject::Dataset(dataset)],
                    vec![tx.task_id.as_object_id()],
                    vec![event],
                    50_000,
                ))
            }

            TransactionPayload::CancelTask(tx) => {
                let mut task = store.get_task(&tx.task_id)?;
                task.cancel().map_err(|e| {
                    ExecutionError::InvalidStateTransition(format!("{:?}", e))
                })?;
                store.put_task(task)?;

                let event = TransactionEvent {
                    event_type: "TaskCancelled".to_string(),
                    object_ids: vec![tx.task_id.as_object_id()],
                    data: vec![],
                };

                Ok((vec![], vec![tx.task_id.as_object_id()], vec![event], 20_000))
            }

            TransactionPayload::OpenPaymentStream(tx) => {
                let channel = PaymentChannel::new(
                    tx.payer,
                    tx.payee.clone(),
                    tx.rate_per_second,
                    tx.initial_escrow,
                    timestamp,
                );

                let channel_id = channel.id;
                store.put_payment_channel(channel.clone())?;

                let event = TransactionEvent {
                    event_type: "PaymentStreamOpened".to_string(),
                    object_ids: vec![channel_id.as_object_id()],
                    data: vec![],
                };

                Ok((vec![AnyObject::PaymentChannel(channel)], vec![], vec![event], 35_000))
            }

            TransactionPayload::ExtendPaymentStream(tx) => {
                let mut channel = store.get_payment_channel(&tx.channel_id)?;
                channel.extend_escrow(tx.additional_escrow);
                store.put_payment_channel(channel)?;

                let event = TransactionEvent {
                    event_type: "PaymentStreamExtended".to_string(),
                    object_ids: vec![tx.channel_id.as_object_id()],
                    data: vec![],
                };

                Ok((vec![], vec![tx.channel_id.as_object_id()], vec![event], 20_000))
            }

            TransactionPayload::WithdrawPayment(tx) => {
                let mut channel = store.get_payment_channel(&tx.channel_id)?;
                let amount = channel.withdraw(timestamp).map_err(|e| {
                    ExecutionError::InvalidStateTransition(format!("{:?}", e))
                })?;
                store.put_payment_channel(channel)?;

                let event = TransactionEvent {
                    event_type: "PaymentWithdrawn".to_string(),
                    object_ids: vec![tx.channel_id.as_object_id()],
                    data: amount.to_le_bytes().to_vec(),
                };

                Ok((vec![], vec![tx.channel_id.as_object_id()], vec![event], 25_000))
            }

            TransactionPayload::ClosePaymentStream(tx) => {
                let mut channel = store.get_payment_channel(&tx.channel_id)?;
                let (payee_amount, refund) = channel.close(timestamp).map_err(|e| {
                    ExecutionError::InvalidStateTransition(format!("{:?}", e))
                })?;
                store.put_payment_channel(channel)?;

                let event = TransactionEvent {
                    event_type: "PaymentStreamClosed".to_string(),
                    object_ids: vec![tx.channel_id.as_object_id()],
                    data: vec![],
                };

                Ok((vec![], vec![tx.channel_id.as_object_id()], vec![event], 30_000))
            }

            TransactionPayload::SubmitDataset(tx) => {
                let mut dataset = Dataset::new(
                    tx.dataset_type.clone(),
                    tx.source_robot,
                    sender_id,
                    tx.off_chain_uri.clone(),
                    tx.merkle_root,
                    tx.size_bytes,
                    timestamp,
                );

                for proof in &tx.proofs {
                    dataset.add_proof(proof.clone());
                }

                let dataset_id = dataset.id;
                store.put_dataset(dataset.clone())?;

                let event = TransactionEvent {
                    event_type: "DatasetSubmitted".to_string(),
                    object_ids: vec![dataset_id.as_object_id()],
                    data: vec![],
                };

                Ok((vec![AnyObject::Dataset(dataset)], vec![], vec![event], 40_000))
            }

            TransactionPayload::ReportIncident(tx) => {
                // Create evidence dataset
                let dataset = Dataset::new(
                    DatasetType::IncidentEvidence,
                    tx.robot_id,
                    sender_id,
                    tx.evidence_uri.clone(),
                    tx.evidence_merkle_root,
                    0,
                    timestamp,
                );

                let dataset_id = dataset.id;
                store.put_dataset(dataset.clone())?;

                // Update task to disputed
                let mut task = store.get_task(&tx.task_id)?;
                let _ = task.dispute(); // Ignore error if already disputed
                store.put_task(task)?;

                // Update robot incident count
                let mut robot = store.get_robot(&tx.robot_id)?;
                // Incident tracking would go here
                store.put_robot(robot)?;

                let event = TransactionEvent {
                    event_type: "IncidentReported".to_string(),
                    object_ids: vec![
                        tx.task_id.as_object_id(),
                        tx.robot_id.as_object_id(),
                        dataset_id.as_object_id(),
                    ],
                    data: vec![],
                };

                Ok((
                    vec![AnyObject::Dataset(dataset)],
                    vec![tx.task_id.as_object_id(), tx.robot_id.as_object_id()],
                    vec![event],
                    60_000,
                ))
            }

            TransactionPayload::EmergencyRevoke(tx) => {
                // Emergency revocations
                let event = TransactionEvent {
                    event_type: "EmergencyRevoke".to_string(),
                    object_ids: vec![],
                    data: tx.reason.as_bytes().to_vec(),
                };

                // In production, would revoke all relevant policies
                Ok((vec![], vec![], vec![event], 100_000))
            }

            _ => Err(ExecutionError::ExecutionFailed(
                "Unhandled transaction type".to_string(),
            )),
        }
    }

    /// Create a receipt from execution result
    pub fn create_receipt(
        tx: &Transaction,
        result: &ExecutionResult,
        block_number: u64,
    ) -> TransactionReceipt {
        TransactionReceipt {
            tx_hash: tx.hash,
            success: result.success,
            gas_used: result.gas_used,
            block_number,
            error: result.error.clone(),
            events: result.events.clone(),
        }
    }
}
