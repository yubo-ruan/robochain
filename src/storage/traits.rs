//! Storage trait definitions

use super::StorageError;
use crate::crypto::Hash;
use crate::objects::*;

/// Object store trait for accessing chain objects
pub trait ObjectStore: Send + Sync {
    // === Robot Operations ===
    fn get_robot(&self, id: &RobotId) -> Result<Robot, StorageError>;
    fn put_robot(&mut self, robot: Robot) -> Result<(), StorageError>;
    fn delete_robot(&mut self, id: &RobotId) -> Result<(), StorageError>;

    // === Space Operations ===
    fn get_space(&self, id: &SpaceId) -> Result<Space, StorageError>;
    fn put_space(&mut self, space: Space) -> Result<(), StorageError>;
    fn delete_space(&mut self, id: &SpaceId) -> Result<(), StorageError>;

    // === Principal Operations ===
    fn get_principal(&self, id: &PrincipalId) -> Result<Principal, StorageError>;
    fn put_principal(&mut self, principal: Principal) -> Result<(), StorageError>;
    fn delete_principal(&mut self, id: &PrincipalId) -> Result<(), StorageError>;

    // === Task Operations ===
    fn get_task(&self, id: &TaskId) -> Result<Task, StorageError>;
    fn put_task(&mut self, task: Task) -> Result<(), StorageError>;
    fn delete_task(&mut self, id: &TaskId) -> Result<(), StorageError>;

    // === Policy Operations ===
    fn get_policy(&self, id: &PolicyId) -> Result<Policy, StorageError>;
    fn put_policy(&mut self, policy: Policy) -> Result<(), StorageError>;
    fn delete_policy(&mut self, id: &PolicyId) -> Result<(), StorageError>;

    // === Dataset Operations ===
    fn get_dataset(&self, id: &DatasetId) -> Result<Dataset, StorageError>;
    fn put_dataset(&mut self, dataset: Dataset) -> Result<(), StorageError>;
    fn delete_dataset(&mut self, id: &DatasetId) -> Result<(), StorageError>;

    // === Payment Channel Operations ===
    fn get_payment_channel(&self, id: &PaymentChannelId) -> Result<PaymentChannel, StorageError>;
    fn put_payment_channel(&mut self, channel: PaymentChannel) -> Result<(), StorageError>;
    fn delete_payment_channel(&mut self, id: &PaymentChannelId) -> Result<(), StorageError>;

    // === Generic Object Operations ===
    fn get_object(&self, id: &ObjectId) -> Result<AnyObject, StorageError>;
    fn object_exists(&self, id: &ObjectId) -> bool;

    // === State Management ===
    fn compute_state_root(&self) -> Hash;
    fn object_count(&self) -> u64;

    // === Queries ===
    fn get_tasks_by_space(&self, space_id: &SpaceId) -> Vec<Task>;
    fn get_tasks_by_status(&self, status: &TaskStatus) -> Vec<Task>;
    fn get_policies_by_robot(&self, robot_id: &RobotId) -> Vec<Policy>;
    fn get_policies_by_space(&self, space_id: &SpaceId) -> Vec<Policy>;
    fn get_robots_by_owner(&self, owner: &PrincipalId) -> Vec<Robot>;
    fn get_active_channels_by_payer(&self, payer: &PrincipalId) -> Vec<PaymentChannel>;
}

/// Trait for atomic batch operations
pub trait BatchStore: ObjectStore {
    /// Begin a new batch
    fn begin_batch(&mut self);

    /// Commit the current batch
    fn commit_batch(&mut self) -> Result<(), StorageError>;

    /// Rollback the current batch
    fn rollback_batch(&mut self);
}

/// Trait for snapshot/checkpoint operations
pub trait SnapshotStore: ObjectStore {
    /// Create a snapshot at current state
    fn create_snapshot(&self) -> Result<Hash, StorageError>;

    /// Restore from a snapshot
    fn restore_snapshot(&mut self, snapshot_hash: &Hash) -> Result<(), StorageError>;

    /// List available snapshots
    fn list_snapshots(&self) -> Vec<Hash>;
}
