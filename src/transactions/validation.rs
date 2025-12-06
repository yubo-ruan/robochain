//! Transaction validation logic

use super::{Transaction, TransactionPayload};
use crate::objects::*;
use crate::storage::ObjectStore;
use thiserror::Error;

/// Errors during transaction validation
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Transaction expired")]
    Expired,

    #[error("Invalid nonce: expected {expected}, got {got}")]
    InvalidNonce { expected: u64, got: u64 },

    #[error("Insufficient gas: max {max}, required {required}")]
    InsufficientGas { max: u64, required: u64 },

    #[error("Object not found: {0}")]
    ObjectNotFound(ObjectId),

    #[error("Unauthorized: sender cannot perform this action")]
    Unauthorized,

    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(String),

    #[error("Object version mismatch: expected {expected}, got {got}")]
    VersionMismatch { expected: u64, got: u64 },

    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: u64, available: u64 },

    #[error("Task expired")]
    TaskExpired,

    #[error("Invalid capability: robot lacks required capability")]
    InvalidCapability,

    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    #[error("Invalid proof: {0}")]
    InvalidProof(String),
}

/// Transaction validator
pub struct TransactionValidator;

impl TransactionValidator {
    /// Validate a transaction against current state
    pub fn validate(
        tx: &Transaction,
        store: &dyn ObjectStore,
        current_time: u64,
    ) -> Result<(), ValidationError> {
        // 1. Verify signature
        if !tx.verify_signature() {
            return Err(ValidationError::InvalidSignature);
        }

        // 2. Check timestamp (allow 5 minute window)
        let time_window = 300; // 5 minutes
        if tx.timestamp > current_time + time_window {
            return Err(ValidationError::Expired);
        }

        // 3. Validate payload-specific rules
        Self::validate_payload(&tx.payload, &tx.sender, store, current_time)
    }

    /// Validate payload-specific rules
    fn validate_payload(
        payload: &TransactionPayload,
        sender: &crate::crypto::PublicKey,
        store: &dyn ObjectStore,
        current_time: u64,
    ) -> Result<(), ValidationError> {
        match payload {
            TransactionPayload::RegisterRobot(tx) => {
                // Validate hardware attestation signature
                // In production, verify against manufacturer's known public key
                Ok(())
            }

            TransactionPayload::UpdateFirmware(tx) => {
                // Check robot exists
                let robot = store
                    .get_robot(&tx.robot_id)
                    .map_err(|_| ValidationError::ObjectNotFound(tx.robot_id.as_object_id()))?;

                // Only manufacturer can update firmware
                // Verify vendor signature matches manufacturer
                Ok(())
            }

            TransactionPayload::TransferRobot(tx) => {
                let robot = store
                    .get_robot(&tx.robot_id)
                    .map_err(|_| ValidationError::ObjectNotFound(tx.robot_id.as_object_id()))?;

                // Check sender is current owner
                if let Some(owner) = &robot.owner {
                    let sender_id = PrincipalId::new(crate::crypto::Hash::from_bytes(sender.as_bytes()));
                    if owner != &sender_id {
                        return Err(ValidationError::Unauthorized);
                    }
                }
                Ok(())
            }

            TransactionPayload::SetPolicy(tx) => {
                let space = store
                    .get_space(&tx.space_id)
                    .map_err(|_| ValidationError::ObjectNotFound(tx.space_id.as_object_id()))?;

                // Check sender is owner
                let sender_id = PrincipalId::new(crate::crypto::Hash::from_bytes(sender.as_bytes()));
                if !space.is_owner(&sender_id) {
                    return Err(ValidationError::Unauthorized);
                }
                Ok(())
            }

            TransactionPayload::GrantCapability(tx) => {
                let space = store
                    .get_space(&tx.space_id)
                    .map_err(|_| ValidationError::ObjectNotFound(tx.space_id.as_object_id()))?;

                let robot = store
                    .get_robot(&tx.robot_id)
                    .map_err(|_| ValidationError::ObjectNotFound(tx.robot_id.as_object_id()))?;

                // Check sender is space owner
                let sender_id = PrincipalId::new(crate::crypto::Hash::from_bytes(sender.as_bytes()));
                if !space.is_owner(&sender_id) {
                    return Err(ValidationError::Unauthorized);
                }

                // Check robot is active
                if robot.status != RobotStatus::Active {
                    return Err(ValidationError::InvalidStateTransition(
                        "Robot is not active".to_string(),
                    ));
                }

                Ok(())
            }

            TransactionPayload::RevokeCapability(tx) => {
                let policy = store
                    .get_policy(&tx.policy_id)
                    .map_err(|_| ValidationError::ObjectNotFound(tx.policy_id.as_object_id()))?;

                // Check sender is grantor
                let sender_id = PrincipalId::new(crate::crypto::Hash::from_bytes(sender.as_bytes()));
                if policy.grantor != sender_id {
                    return Err(ValidationError::Unauthorized);
                }

                // Check policy is revocable
                if !policy.revocable {
                    return Err(ValidationError::PolicyViolation(
                        "Policy is not revocable".to_string(),
                    ));
                }

                Ok(())
            }

            TransactionPayload::AcceptTask(tx) => {
                let task = store
                    .get_task(&tx.task_id)
                    .map_err(|_| ValidationError::ObjectNotFound(tx.task_id.as_object_id()))?;

                let robot = store
                    .get_robot(&tx.robot_id)
                    .map_err(|_| ValidationError::ObjectNotFound(tx.robot_id.as_object_id()))?;

                // Check task is still posted
                if task.status != TaskStatus::Posted {
                    return Err(ValidationError::InvalidStateTransition(
                        "Task is not available".to_string(),
                    ));
                }

                // Check task is not expired
                if !task.is_valid(current_time) {
                    return Err(ValidationError::TaskExpired);
                }

                // Check robot has required capabilities
                if !task.robot_can_perform(&robot.capabilities) {
                    return Err(ValidationError::InvalidCapability);
                }

                Ok(())
            }

            TransactionPayload::SubmitCompletion(tx) => {
                let task = store
                    .get_task(&tx.task_id)
                    .map_err(|_| ValidationError::ObjectNotFound(tx.task_id.as_object_id()))?;

                // Check task is in progress
                if task.status != TaskStatus::InProgress {
                    return Err(ValidationError::InvalidStateTransition(
                        "Task is not in progress".to_string(),
                    ));
                }

                // Verify safety proofs (simplified)
                // In production, verify ZK proofs here
                Ok(())
            }

            TransactionPayload::OpenPaymentStream(tx) => {
                // Check payer has sufficient funds (would check balance module)
                // For now, just check payer exists
                let _payer = store
                    .get_principal(&tx.payer)
                    .map_err(|_| ValidationError::ObjectNotFound(tx.payer.as_object_id()))?;

                Ok(())
            }

            TransactionPayload::ClosePaymentStream(tx) => {
                let channel = store
                    .get_payment_channel(&tx.channel_id)
                    .map_err(|_| ValidationError::ObjectNotFound(tx.channel_id.as_object_id()))?;

                // Check channel is not already closed
                if channel.status == ChannelStatus::Closed {
                    return Err(ValidationError::InvalidStateTransition(
                        "Channel already closed".to_string(),
                    ));
                }

                // Check sender is payer or payee
                let sender_id = PrincipalId::new(crate::crypto::Hash::from_bytes(sender.as_bytes()));
                let is_payer = channel.payer == sender_id;
                let is_payee = match &channel.payee {
                    Payee::Principal(p) => p == &sender_id,
                    Payee::Robot(_) => false, // Robot would need separate auth
                };

                if !is_payer && !is_payee {
                    return Err(ValidationError::Unauthorized);
                }

                Ok(())
            }

            TransactionPayload::EmergencyRevoke(tx) => {
                // Emergency revokes have special authorization
                // Typically require multi-sig or safety oracle signature
                Ok(())
            }

            // Default case for other transaction types
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests would go here with mock ObjectStore
}
