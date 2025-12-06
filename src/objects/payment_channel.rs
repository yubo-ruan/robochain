//! Payment Channel object definition

use super::{ChainObject, ObjectId, ObjectType, PaymentChannelId, PrincipalId, RobotId, TaskId};
use crate::crypto::Hash;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

/// Status of a payment channel
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum ChannelStatus {
    /// Channel is open and streaming
    Active,
    /// Channel is paused
    Paused,
    /// Channel is being settled
    Settling,
    /// Channel has been closed
    Closed,
    /// Channel is disputed
    Disputed,
}

/// Condition for payment release
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum PaymentCondition {
    /// Time-based: pay per second/minute/hour
    TimeBased { rate_per_second: u64 },
    /// Milestone-based: pay on completion
    Milestone { milestone_id: String, amount: u64 },
    /// Proof-based: pay when valid proof submitted
    ProofBased { proof_type: String },
    /// Task completion
    TaskCompletion { task_id: TaskId },
}

/// Payee can be either a robot or a principal
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum Payee {
    Robot(RobotId),
    Principal(PrincipalId),
}

/// A streaming payment channel between parties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentChannel {
    /// Unique channel ID
    pub id: PaymentChannelId,
    /// Payer principal
    pub payer: PrincipalId,
    /// Payee (robot or principal)
    pub payee: Payee,
    /// Payment rate (micro-ROUSD per second)
    pub rate_per_second: u64,
    /// Total escrowed amount
    pub escrowed_amount: u64,
    /// Amount already withdrawn by payee
    pub withdrawn_amount: u64,
    /// Payment conditions
    pub conditions: Vec<PaymentCondition>,
    /// Current status
    pub status: ChannelStatus,
    /// Channel open timestamp
    pub opened_at: u64,
    /// Last withdrawal timestamp
    pub last_withdrawal_at: u64,
    /// Channel close timestamp (if closed)
    pub closed_at: Option<u64>,
    /// Pause timestamp (if paused)
    pub paused_at: Option<u64>,
    /// Total seconds channel has been active
    pub active_seconds: u64,
    /// Associated task (if any)
    pub task_id: Option<TaskId>,
    /// Object version
    version: u64,
}

impl PaymentChannel {
    /// Create a new payment channel
    pub fn new(
        payer: PrincipalId,
        payee: Payee,
        rate_per_second: u64,
        initial_escrow: u64,
        opened_at: u64,
    ) -> Self {
        // Derive channel ID
        let id_bytes = borsh::to_vec(&(&payer, &payee, opened_at)).unwrap_or_default();
        let id = PaymentChannelId::from_bytes(&id_bytes);

        PaymentChannel {
            id,
            payer,
            payee,
            rate_per_second,
            escrowed_amount: initial_escrow,
            withdrawn_amount: 0,
            conditions: Vec::new(),
            status: ChannelStatus::Active,
            opened_at,
            last_withdrawal_at: opened_at,
            closed_at: None,
            paused_at: None,
            active_seconds: 0,
            task_id: None,
            version: 1,
        }
    }

    /// Link to a task
    pub fn set_task(&mut self, task_id: TaskId) {
        self.task_id = Some(task_id);
        self.version += 1;
    }

    /// Add payment condition
    pub fn add_condition(&mut self, condition: PaymentCondition) {
        self.conditions.push(condition);
        self.version += 1;
    }

    /// Add more escrow
    pub fn extend_escrow(&mut self, amount: u64) {
        self.escrowed_amount = self.escrowed_amount.saturating_add(amount);
        self.version += 1;
    }

    /// Calculate amount available for withdrawal at given timestamp
    pub fn available_for_withdrawal(&self, current_time: u64) -> u64 {
        if self.status != ChannelStatus::Active {
            return 0;
        }

        let elapsed = current_time.saturating_sub(self.last_withdrawal_at);
        let earned = elapsed.saturating_mul(self.rate_per_second);
        let max_available = self.escrowed_amount.saturating_sub(self.withdrawn_amount);

        earned.min(max_available)
    }

    /// Withdraw earned amount
    pub fn withdraw(&mut self, current_time: u64) -> Result<u64, ChannelError> {
        if self.status != ChannelStatus::Active {
            return Err(ChannelError::NotActive);
        }

        let amount = self.available_for_withdrawal(current_time);
        if amount == 0 {
            return Err(ChannelError::NothingToWithdraw);
        }

        self.withdrawn_amount = self.withdrawn_amount.saturating_add(amount);
        self.active_seconds = self
            .active_seconds
            .saturating_add(current_time.saturating_sub(self.last_withdrawal_at));
        self.last_withdrawal_at = current_time;
        self.version += 1;

        Ok(amount)
    }

    /// Pause the channel
    pub fn pause(&mut self, timestamp: u64) -> Result<(), ChannelError> {
        if self.status != ChannelStatus::Active {
            return Err(ChannelError::NotActive);
        }

        // Calculate active time before pausing
        self.active_seconds = self
            .active_seconds
            .saturating_add(timestamp.saturating_sub(self.last_withdrawal_at));

        self.status = ChannelStatus::Paused;
        self.paused_at = Some(timestamp);
        self.version += 1;
        Ok(())
    }

    /// Resume the channel
    pub fn resume(&mut self, timestamp: u64) -> Result<(), ChannelError> {
        if self.status != ChannelStatus::Paused {
            return Err(ChannelError::NotPaused);
        }

        self.status = ChannelStatus::Active;
        self.last_withdrawal_at = timestamp;
        self.paused_at = None;
        self.version += 1;
        Ok(())
    }

    /// Close the channel and settle
    pub fn close(&mut self, timestamp: u64) -> Result<(u64, u64), ChannelError> {
        if self.status == ChannelStatus::Closed {
            return Err(ChannelError::AlreadyClosed);
        }

        // Calculate final amounts
        let final_payee_amount = if self.status == ChannelStatus::Active {
            self.withdrawn_amount
                .saturating_add(self.available_for_withdrawal(timestamp))
        } else {
            self.withdrawn_amount
        };
        let refund_to_payer = self.escrowed_amount.saturating_sub(final_payee_amount);

        self.withdrawn_amount = final_payee_amount;
        self.status = ChannelStatus::Closed;
        self.closed_at = Some(timestamp);
        self.version += 1;

        Ok((final_payee_amount, refund_to_payer))
    }

    /// Dispute the channel
    pub fn dispute(&mut self) -> Result<(), ChannelError> {
        if self.status == ChannelStatus::Closed {
            return Err(ChannelError::AlreadyClosed);
        }
        self.status = ChannelStatus::Disputed;
        self.version += 1;
        Ok(())
    }

    /// Check if channel is depleted
    pub fn is_depleted(&self) -> bool {
        self.withdrawn_amount >= self.escrowed_amount
    }

    /// Remaining balance in escrow
    pub fn remaining_balance(&self) -> u64 {
        self.escrowed_amount.saturating_sub(self.withdrawn_amount)
    }
}

/// Channel-related errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelError {
    NotActive,
    NotPaused,
    AlreadyClosed,
    NothingToWithdraw,
    InsufficientEscrow,
    Unauthorized,
}

impl ChainObject for PaymentChannel {
    fn object_id(&self) -> ObjectId {
        self.id.as_object_id()
    }

    fn object_type(&self) -> ObjectType {
        ObjectType::PaymentChannel
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn increment_version(&mut self) {
        self.version += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_channel() -> PaymentChannel {
        let payer = PrincipalId::from_bytes(b"payer");
        let robot = RobotId::from_bytes(b"robot");

        PaymentChannel::new(
            payer,
            Payee::Robot(robot),
            100,        // 100 micro-ROUSD per second
            1_000_000,  // 1 ROUSD escrow
            1000,       // opened at timestamp 1000
        )
    }

    #[test]
    fn test_channel_creation() {
        let channel = create_test_channel();
        assert_eq!(channel.status, ChannelStatus::Active);
        assert_eq!(channel.rate_per_second, 100);
        assert_eq!(channel.escrowed_amount, 1_000_000);
    }

    #[test]
    fn test_withdrawal() {
        let mut channel = create_test_channel();

        // After 100 seconds, should be able to withdraw 100 * 100 = 10000
        let available = channel.available_for_withdrawal(1100);
        assert_eq!(available, 10000);

        let withdrawn = channel.withdraw(1100).unwrap();
        assert_eq!(withdrawn, 10000);
        assert_eq!(channel.withdrawn_amount, 10000);
    }

    #[test]
    fn test_pause_resume() {
        let mut channel = create_test_channel();

        assert!(channel.pause(1100).is_ok());
        assert_eq!(channel.status, ChannelStatus::Paused);

        // Cannot withdraw while paused
        assert!(channel.withdraw(1200).is_err());

        // Resume
        assert!(channel.resume(1200).is_ok());
        assert_eq!(channel.status, ChannelStatus::Active);
    }

    #[test]
    fn test_close_settlement() {
        let mut channel = create_test_channel();

        // Withdraw some first
        channel.withdraw(1100).unwrap();

        // Close at timestamp 1200
        let (payee_amount, refund) = channel.close(1200).unwrap();

        // Payee gets: 10000 (already withdrawn) + 10000 (100 seconds * 100) = 20000
        assert_eq!(payee_amount, 20000);
        // Payer refund: 1000000 - 20000 = 980000
        assert_eq!(refund, 980000);
    }
}
