//! Core domain objects for RoboChain
//!
//! These are the fundamental entities that exist on the chain:
//! - Robot: A registered home robot
//! - Space: A home or physical space
//! - Principal: A human or entity (owner, guest, service provider)
//! - Task: A job to be performed by a robot
//! - Policy: Permission grants between principals and robots
//! - Dataset: Reference to off-chain data with on-chain commitment
//! - PaymentChannel: Streaming payment between parties

mod robot;
mod space;
mod principal;
mod task;
mod policy;
mod dataset;
mod payment_channel;
mod object_id;

pub use robot::*;
pub use space::*;
pub use principal::*;
pub use task::*;
pub use policy::*;
pub use dataset::*;
pub use payment_channel::*;
pub use object_id::*;

use crate::crypto::Hash;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

/// Trait for all chain objects
pub trait ChainObject: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync {
    /// Get the unique object ID
    fn object_id(&self) -> ObjectId;

    /// Get the object type
    fn object_type(&self) -> ObjectType;

    /// Get the current version
    fn version(&self) -> u64;

    /// Increment version (for updates)
    fn increment_version(&mut self);
}

/// Enum of all object types in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum ObjectType {
    Robot,
    Space,
    Principal,
    Task,
    Policy,
    Dataset,
    PaymentChannel,
}

impl ObjectType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ObjectType::Robot => "robot",
            ObjectType::Space => "space",
            ObjectType::Principal => "principal",
            ObjectType::Task => "task",
            ObjectType::Policy => "policy",
            ObjectType::Dataset => "dataset",
            ObjectType::PaymentChannel => "payment_channel",
        }
    }
}

/// Wrapper enum for any chain object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnyObject {
    Robot(Robot),
    Space(Space),
    Principal(Principal),
    Task(Task),
    Policy(Policy),
    Dataset(Dataset),
    PaymentChannel(PaymentChannel),
}

impl AnyObject {
    pub fn object_id(&self) -> ObjectId {
        match self {
            AnyObject::Robot(o) => o.object_id(),
            AnyObject::Space(o) => o.object_id(),
            AnyObject::Principal(o) => o.object_id(),
            AnyObject::Task(o) => o.object_id(),
            AnyObject::Policy(o) => o.object_id(),
            AnyObject::Dataset(o) => o.object_id(),
            AnyObject::PaymentChannel(o) => o.object_id(),
        }
    }

    pub fn object_type(&self) -> ObjectType {
        match self {
            AnyObject::Robot(_) => ObjectType::Robot,
            AnyObject::Space(_) => ObjectType::Space,
            AnyObject::Principal(_) => ObjectType::Principal,
            AnyObject::Task(_) => ObjectType::Task,
            AnyObject::Policy(_) => ObjectType::Policy,
            AnyObject::Dataset(_) => ObjectType::Dataset,
            AnyObject::PaymentChannel(_) => ObjectType::PaymentChannel,
        }
    }

    pub fn version(&self) -> u64 {
        match self {
            AnyObject::Robot(o) => o.version(),
            AnyObject::Space(o) => o.version(),
            AnyObject::Principal(o) => o.version(),
            AnyObject::Task(o) => o.version(),
            AnyObject::Policy(o) => o.version(),
            AnyObject::Dataset(o) => o.version(),
            AnyObject::PaymentChannel(o) => o.version(),
        }
    }
}
