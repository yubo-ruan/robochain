//! Robot object definition

use super::{ChainObject, ObjectId, ObjectType, RobotId};
use crate::crypto::{Hash, PublicKey, Signature};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Capability that a robot can have
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum Capability {
    /// Navigation and movement
    Navigation,
    /// Object manipulation (picking, placing)
    Manipulation,
    /// Cleaning tasks
    Cleaning,
    /// Cooking/food preparation
    Cooking,
    /// Security/surveillance
    Security,
    /// Delivery within space
    Delivery,
    /// Elderly/medical assistance
    CareAssistance,
    /// Entertainment/companion
    Entertainment,
    /// Custom capability with string identifier
    Custom(String),
}

/// Hardware attestation from secure element
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct HardwareAttestation {
    /// Public key from the robot's secure element
    pub secure_element_pubkey: PublicKey,
    /// Attestation signature from manufacturer
    pub manufacturer_signature: Signature,
    /// Certificate chain hash (for verification)
    pub cert_chain_hash: Hash,
    /// Timestamp of attestation
    pub attested_at: u64,
}

/// Status of a robot on the chain
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum RobotStatus {
    /// Robot is active and can accept tasks
    Active,
    /// Robot is temporarily offline/unavailable
    Offline,
    /// Robot is suspended due to incidents
    Suspended,
    /// Robot has been decommissioned
    Decommissioned,
}

/// A registered home robot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Robot {
    /// Unique robot ID (derived from hardware attestation)
    pub id: RobotId,
    /// Manufacturer identifier
    pub manufacturer: String,
    /// Model name/number
    pub model: String,
    /// Hardware fingerprint from secure element
    pub hw_fingerprint: Hash,
    /// Current firmware hash
    pub firmware_hash: Hash,
    /// Set of capabilities this robot has
    pub capabilities: HashSet<Capability>,
    /// Hardware attestation proof
    pub hw_attestation: HardwareAttestation,
    /// Current status
    pub status: RobotStatus,
    /// Owner principal ID
    pub owner: Option<super::PrincipalId>,
    /// Registration timestamp
    pub registered_at: u64,
    /// Last activity timestamp
    pub last_active_at: u64,
    /// Object version for optimistic concurrency
    version: u64,
}

impl Robot {
    /// Create a new robot registration
    pub fn new(
        manufacturer: String,
        model: String,
        hw_attestation: HardwareAttestation,
        capabilities: HashSet<Capability>,
        registered_at: u64,
    ) -> Self {
        // Derive robot ID from hardware attestation
        let id_bytes = borsh::to_vec(&(
            &manufacturer,
            &model,
            &hw_attestation.secure_element_pubkey,
        ))
        .unwrap_or_default();
        let id = RobotId::from_bytes(&id_bytes);

        Robot {
            id,
            manufacturer,
            model,
            hw_fingerprint: Hash::from_bytes(hw_attestation.secure_element_pubkey.as_bytes()),
            firmware_hash: Hash::zero(),
            capabilities,
            hw_attestation,
            status: RobotStatus::Active,
            owner: None,
            registered_at,
            last_active_at: registered_at,
            version: 1,
        }
    }

    /// Update firmware hash
    pub fn update_firmware(&mut self, firmware_hash: Hash) {
        self.firmware_hash = firmware_hash;
        self.version += 1;
    }

    /// Set owner
    pub fn set_owner(&mut self, owner: super::PrincipalId) {
        self.owner = Some(owner);
        self.version += 1;
    }

    /// Update status
    pub fn set_status(&mut self, status: RobotStatus) {
        self.status = status;
        self.version += 1;
    }

    /// Check if robot has a capability
    pub fn has_capability(&self, capability: &Capability) -> bool {
        self.capabilities.contains(capability)
    }

    /// Add a capability
    pub fn add_capability(&mut self, capability: Capability) {
        self.capabilities.insert(capability);
        self.version += 1;
    }

    /// Record activity
    pub fn record_activity(&mut self, timestamp: u64) {
        self.last_active_at = timestamp;
    }
}

impl ChainObject for Robot {
    fn object_id(&self) -> ObjectId {
        self.id.as_object_id()
    }

    fn object_type(&self) -> ObjectType {
        ObjectType::Robot
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
    use crate::crypto::KeyPair;

    fn create_test_robot() -> Robot {
        let keypair = KeyPair::generate();
        let attestation = HardwareAttestation {
            secure_element_pubkey: keypair.public_key(),
            manufacturer_signature: keypair.sign(b"attestation"),
            cert_chain_hash: Hash::from_bytes(b"cert_chain"),
            attested_at: 1000,
        };

        let mut capabilities = HashSet::new();
        capabilities.insert(Capability::Navigation);
        capabilities.insert(Capability::Manipulation);

        Robot::new(
            "RobotCorp".to_string(),
            "HomeBot-X1".to_string(),
            attestation,
            capabilities,
            1000,
        )
    }

    #[test]
    fn test_robot_creation() {
        let robot = create_test_robot();
        assert_eq!(robot.manufacturer, "RobotCorp");
        assert_eq!(robot.model, "HomeBot-X1");
        assert_eq!(robot.status, RobotStatus::Active);
        assert_eq!(robot.version(), 1);
    }

    #[test]
    fn test_robot_capabilities() {
        let robot = create_test_robot();
        assert!(robot.has_capability(&Capability::Navigation));
        assert!(robot.has_capability(&Capability::Manipulation));
        assert!(!robot.has_capability(&Capability::Cleaning));
    }

    #[test]
    fn test_firmware_update() {
        let mut robot = create_test_robot();
        let new_firmware = Hash::from_bytes(b"new_firmware_v2");
        robot.update_firmware(new_firmware);
        assert_eq!(robot.firmware_hash, new_firmware);
        assert_eq!(robot.version(), 2);
    }
}
