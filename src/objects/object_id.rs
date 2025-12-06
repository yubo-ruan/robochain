//! Object ID types for RoboChain

use crate::crypto::Hash;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for any chain object
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ObjectId(pub Hash);

impl ObjectId {
    pub fn new(hash: Hash) -> Self {
        ObjectId(hash)
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        ObjectId(Hash::from_bytes(bytes))
    }

    pub fn as_hash(&self) -> &Hash {
        &self.0
    }
}

impl fmt::Debug for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ObjectId({:?})", self.0)
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Robot-specific ID
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct RobotId(pub ObjectId);

impl RobotId {
    pub fn new(hash: Hash) -> Self {
        RobotId(ObjectId::new(hash))
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        RobotId(ObjectId::from_bytes(bytes))
    }

    pub fn as_object_id(&self) -> ObjectId {
        self.0
    }
}

impl fmt::Debug for RobotId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RobotId({:?})", self.0)
    }
}

/// Space-specific ID
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SpaceId(pub ObjectId);

impl SpaceId {
    pub fn new(hash: Hash) -> Self {
        SpaceId(ObjectId::new(hash))
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        SpaceId(ObjectId::from_bytes(bytes))
    }

    pub fn as_object_id(&self) -> ObjectId {
        self.0
    }
}

impl fmt::Debug for SpaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SpaceId({:?})", self.0)
    }
}

/// Principal-specific ID
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct PrincipalId(pub ObjectId);

impl PrincipalId {
    pub fn new(hash: Hash) -> Self {
        PrincipalId(ObjectId::new(hash))
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        PrincipalId(ObjectId::from_bytes(bytes))
    }

    pub fn as_object_id(&self) -> ObjectId {
        self.0
    }
}

impl fmt::Debug for PrincipalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PrincipalId({:?})", self.0)
    }
}

/// Task-specific ID
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct TaskId(pub ObjectId);

impl TaskId {
    pub fn new(hash: Hash) -> Self {
        TaskId(ObjectId::new(hash))
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        TaskId(ObjectId::from_bytes(bytes))
    }

    pub fn as_object_id(&self) -> ObjectId {
        self.0
    }
}

impl fmt::Debug for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TaskId({:?})", self.0)
    }
}

/// Policy-specific ID
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct PolicyId(pub ObjectId);

impl PolicyId {
    pub fn new(hash: Hash) -> Self {
        PolicyId(ObjectId::new(hash))
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        PolicyId(ObjectId::from_bytes(bytes))
    }

    pub fn as_object_id(&self) -> ObjectId {
        self.0
    }
}

impl fmt::Debug for PolicyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PolicyId({:?})", self.0)
    }
}

/// Dataset-specific ID
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct DatasetId(pub ObjectId);

impl DatasetId {
    pub fn new(hash: Hash) -> Self {
        DatasetId(ObjectId::new(hash))
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        DatasetId(ObjectId::from_bytes(bytes))
    }

    pub fn as_object_id(&self) -> ObjectId {
        self.0
    }
}

impl fmt::Debug for DatasetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DatasetId({:?})", self.0)
    }
}

/// PaymentChannel-specific ID
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct PaymentChannelId(pub ObjectId);

impl PaymentChannelId {
    pub fn new(hash: Hash) -> Self {
        PaymentChannelId(ObjectId::new(hash))
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        PaymentChannelId(ObjectId::from_bytes(bytes))
    }

    pub fn as_object_id(&self) -> ObjectId {
        self.0
    }
}

impl fmt::Debug for PaymentChannelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PaymentChannelId({:?})", self.0)
    }
}
