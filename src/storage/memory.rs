//! In-memory object store implementation

use super::{ObjectStore, StorageError};
use crate::crypto::Hash;
use crate::objects::*;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Thread-safe in-memory object store
pub struct MemoryStore {
    robots: DashMap<RobotId, Robot>,
    spaces: DashMap<SpaceId, Space>,
    principals: DashMap<PrincipalId, Principal>,
    tasks: DashMap<TaskId, Task>,
    policies: DashMap<PolicyId, Policy>,
    datasets: DashMap<DatasetId, Dataset>,
    payment_channels: DashMap<PaymentChannelId, PaymentChannel>,
    object_counter: AtomicU64,
}

impl MemoryStore {
    pub fn new() -> Self {
        MemoryStore {
            robots: DashMap::new(),
            spaces: DashMap::new(),
            principals: DashMap::new(),
            tasks: DashMap::new(),
            policies: DashMap::new(),
            datasets: DashMap::new(),
            payment_channels: DashMap::new(),
            object_counter: AtomicU64::new(0),
        }
    }

    fn increment_counter(&self) {
        self.object_counter.fetch_add(1, Ordering::SeqCst);
    }

    fn decrement_counter(&self) {
        self.object_counter.fetch_sub(1, Ordering::SeqCst);
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ObjectStore for MemoryStore {
    // === Robot Operations ===
    fn get_robot(&self, id: &RobotId) -> Result<Robot, StorageError> {
        self.robots
            .get(id)
            .map(|r| r.clone())
            .ok_or_else(|| StorageError::NotFound(id.as_object_id()))
    }

    fn put_robot(&mut self, robot: Robot) -> Result<(), StorageError> {
        let is_new = !self.robots.contains_key(&robot.id);
        self.robots.insert(robot.id, robot);
        if is_new {
            self.increment_counter();
        }
        Ok(())
    }

    fn delete_robot(&mut self, id: &RobotId) -> Result<(), StorageError> {
        if self.robots.remove(id).is_some() {
            self.decrement_counter();
        }
        Ok(())
    }

    // === Space Operations ===
    fn get_space(&self, id: &SpaceId) -> Result<Space, StorageError> {
        self.spaces
            .get(id)
            .map(|s| s.clone())
            .ok_or_else(|| StorageError::NotFound(id.as_object_id()))
    }

    fn put_space(&mut self, space: Space) -> Result<(), StorageError> {
        let is_new = !self.spaces.contains_key(&space.id);
        self.spaces.insert(space.id, space);
        if is_new {
            self.increment_counter();
        }
        Ok(())
    }

    fn delete_space(&mut self, id: &SpaceId) -> Result<(), StorageError> {
        if self.spaces.remove(id).is_some() {
            self.decrement_counter();
        }
        Ok(())
    }

    // === Principal Operations ===
    fn get_principal(&self, id: &PrincipalId) -> Result<Principal, StorageError> {
        self.principals
            .get(id)
            .map(|p| p.clone())
            .ok_or_else(|| StorageError::NotFound(id.as_object_id()))
    }

    fn put_principal(&mut self, principal: Principal) -> Result<(), StorageError> {
        let is_new = !self.principals.contains_key(&principal.id);
        self.principals.insert(principal.id, principal);
        if is_new {
            self.increment_counter();
        }
        Ok(())
    }

    fn delete_principal(&mut self, id: &PrincipalId) -> Result<(), StorageError> {
        if self.principals.remove(id).is_some() {
            self.decrement_counter();
        }
        Ok(())
    }

    // === Task Operations ===
    fn get_task(&self, id: &TaskId) -> Result<Task, StorageError> {
        self.tasks
            .get(id)
            .map(|t| t.clone())
            .ok_or_else(|| StorageError::NotFound(id.as_object_id()))
    }

    fn put_task(&mut self, task: Task) -> Result<(), StorageError> {
        let is_new = !self.tasks.contains_key(&task.id);
        self.tasks.insert(task.id, task);
        if is_new {
            self.increment_counter();
        }
        Ok(())
    }

    fn delete_task(&mut self, id: &TaskId) -> Result<(), StorageError> {
        if self.tasks.remove(id).is_some() {
            self.decrement_counter();
        }
        Ok(())
    }

    // === Policy Operations ===
    fn get_policy(&self, id: &PolicyId) -> Result<Policy, StorageError> {
        self.policies
            .get(id)
            .map(|p| p.clone())
            .ok_or_else(|| StorageError::NotFound(id.as_object_id()))
    }

    fn put_policy(&mut self, policy: Policy) -> Result<(), StorageError> {
        let is_new = !self.policies.contains_key(&policy.id);
        self.policies.insert(policy.id, policy);
        if is_new {
            self.increment_counter();
        }
        Ok(())
    }

    fn delete_policy(&mut self, id: &PolicyId) -> Result<(), StorageError> {
        if self.policies.remove(id).is_some() {
            self.decrement_counter();
        }
        Ok(())
    }

    // === Dataset Operations ===
    fn get_dataset(&self, id: &DatasetId) -> Result<Dataset, StorageError> {
        self.datasets
            .get(id)
            .map(|d| d.clone())
            .ok_or_else(|| StorageError::NotFound(id.as_object_id()))
    }

    fn put_dataset(&mut self, dataset: Dataset) -> Result<(), StorageError> {
        let is_new = !self.datasets.contains_key(&dataset.id);
        self.datasets.insert(dataset.id, dataset);
        if is_new {
            self.increment_counter();
        }
        Ok(())
    }

    fn delete_dataset(&mut self, id: &DatasetId) -> Result<(), StorageError> {
        if self.datasets.remove(id).is_some() {
            self.decrement_counter();
        }
        Ok(())
    }

    // === Payment Channel Operations ===
    fn get_payment_channel(&self, id: &PaymentChannelId) -> Result<PaymentChannel, StorageError> {
        self.payment_channels
            .get(id)
            .map(|c| c.clone())
            .ok_or_else(|| StorageError::NotFound(id.as_object_id()))
    }

    fn put_payment_channel(&mut self, channel: PaymentChannel) -> Result<(), StorageError> {
        let is_new = !self.payment_channels.contains_key(&channel.id);
        self.payment_channels.insert(channel.id, channel);
        if is_new {
            self.increment_counter();
        }
        Ok(())
    }

    fn delete_payment_channel(&mut self, id: &PaymentChannelId) -> Result<(), StorageError> {
        if self.payment_channels.remove(id).is_some() {
            self.decrement_counter();
        }
        Ok(())
    }

    // === Generic Object Operations ===
    fn get_object(&self, id: &ObjectId) -> Result<AnyObject, StorageError> {
        // Try each type
        if let Some(r) = self.robots.iter().find(|r| r.id.as_object_id() == *id) {
            return Ok(AnyObject::Robot(r.clone()));
        }
        if let Some(s) = self.spaces.iter().find(|s| s.id.as_object_id() == *id) {
            return Ok(AnyObject::Space(s.clone()));
        }
        if let Some(p) = self.principals.iter().find(|p| p.id.as_object_id() == *id) {
            return Ok(AnyObject::Principal(p.clone()));
        }
        if let Some(t) = self.tasks.iter().find(|t| t.id.as_object_id() == *id) {
            return Ok(AnyObject::Task(t.clone()));
        }
        if let Some(p) = self.policies.iter().find(|p| p.id.as_object_id() == *id) {
            return Ok(AnyObject::Policy(p.clone()));
        }
        if let Some(d) = self.datasets.iter().find(|d| d.id.as_object_id() == *id) {
            return Ok(AnyObject::Dataset(d.clone()));
        }
        if let Some(c) = self.payment_channels.iter().find(|c| c.id.as_object_id() == *id) {
            return Ok(AnyObject::PaymentChannel(c.clone()));
        }
        Err(StorageError::NotFound(*id))
    }

    fn object_exists(&self, id: &ObjectId) -> bool {
        self.get_object(id).is_ok()
    }

    fn compute_state_root(&self) -> Hash {
        // Simple state root computation
        let mut data = Vec::new();

        // Collect all object hashes in deterministic order
        let mut robot_ids: Vec<_> = self.robots.iter().map(|r| r.id).collect();
        robot_ids.sort_by_key(|id| id.0.0.0);
        for id in robot_ids {
            data.extend_from_slice(id.0.0.as_bytes());
        }

        let mut space_ids: Vec<_> = self.spaces.iter().map(|s| s.id).collect();
        space_ids.sort_by_key(|id| id.0.0.0);
        for id in space_ids {
            data.extend_from_slice(id.0.0.as_bytes());
        }

        // ... similar for other types

        Hash::from_bytes(&data)
    }

    fn object_count(&self) -> u64 {
        self.object_counter.load(Ordering::SeqCst)
    }

    // === Queries ===
    fn get_tasks_by_space(&self, space_id: &SpaceId) -> Vec<Task> {
        self.tasks
            .iter()
            .filter(|t| t.space_id == *space_id)
            .map(|t| t.clone())
            .collect()
    }

    fn get_tasks_by_status(&self, status: &TaskStatus) -> Vec<Task> {
        self.tasks
            .iter()
            .filter(|t| t.status == *status)
            .map(|t| t.clone())
            .collect()
    }

    fn get_policies_by_robot(&self, robot_id: &RobotId) -> Vec<Policy> {
        self.policies
            .iter()
            .filter(|p| p.grantee_robot.as_ref() == Some(robot_id))
            .map(|p| p.clone())
            .collect()
    }

    fn get_policies_by_space(&self, space_id: &SpaceId) -> Vec<Policy> {
        self.policies
            .iter()
            .filter(|p| p.space_id == *space_id)
            .map(|p| p.clone())
            .collect()
    }

    fn get_robots_by_owner(&self, owner: &PrincipalId) -> Vec<Robot> {
        self.robots
            .iter()
            .filter(|r| r.owner.as_ref() == Some(owner))
            .map(|r| r.clone())
            .collect()
    }

    fn get_active_channels_by_payer(&self, payer: &PrincipalId) -> Vec<PaymentChannel> {
        self.payment_channels
            .iter()
            .filter(|c| c.payer == *payer && c.status == ChannelStatus::Active)
            .map(|c| c.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::KeyPair;
    use std::collections::HashSet;

    #[test]
    fn test_robot_crud() {
        let mut store = MemoryStore::new();

        let keypair = KeyPair::generate();
        let attestation = HardwareAttestation {
            secure_element_pubkey: keypair.public_key(),
            manufacturer_signature: keypair.sign(b"attestation"),
            cert_chain_hash: Hash::from_bytes(b"cert"),
            attested_at: 1000,
        };

        let robot = Robot::new(
            "TestCorp".to_string(),
            "TestBot".to_string(),
            attestation,
            HashSet::new(),
            1000,
        );

        let robot_id = robot.id;

        // Create
        store.put_robot(robot).unwrap();
        assert_eq!(store.object_count(), 1);

        // Read
        let retrieved = store.get_robot(&robot_id).unwrap();
        assert_eq!(retrieved.manufacturer, "TestCorp");

        // Delete
        store.delete_robot(&robot_id).unwrap();
        assert!(store.get_robot(&robot_id).is_err());
        assert_eq!(store.object_count(), 0);
    }

    #[test]
    fn test_task_queries() {
        let mut store = MemoryStore::new();

        let space_id = SpaceId::from_bytes(b"space1");
        let requester = PrincipalId::from_bytes(b"requester");

        let task1 = Task::new(
            space_id,
            requester,
            "Task 1".to_string(),
            HashSet::new(),
            1000,
            2000,
            1000,
        );

        let task2 = Task::new(
            space_id,
            requester,
            "Task 2".to_string(),
            HashSet::new(),
            2000,
            3000,
            1100,
        );

        store.put_task(task1).unwrap();
        store.put_task(task2).unwrap();

        let tasks = store.get_tasks_by_space(&space_id);
        assert_eq!(tasks.len(), 2);

        let posted_tasks = store.get_tasks_by_status(&TaskStatus::Posted);
        assert_eq!(posted_tasks.len(), 2);
    }
}
