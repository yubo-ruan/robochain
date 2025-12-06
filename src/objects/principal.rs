//! Principal (human/entity) object definition

use super::{ChainObject, ObjectId, ObjectType, PrincipalId};
use crate::crypto::{Hash, PublicKey};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

/// Type of principal
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum PrincipalType {
    /// Home/space owner
    Owner,
    /// Temporary guest
    Guest,
    /// Professional service provider
    ServiceProvider,
    /// Manufacturer/OEM
    Manufacturer,
    /// Insurance provider
    Insurer,
    /// Regulatory body
    Regulator,
}

/// KYC verification level
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum KycLevel {
    /// No verification
    None,
    /// Email verified
    EmailVerified,
    /// Phone verified
    PhoneVerified,
    /// Government ID verified
    IdentityVerified,
    /// Professional credentials verified
    ProfessionalVerified,
}

/// A human or entity that can interact with the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Principal {
    /// Unique principal ID
    pub id: PrincipalId,
    /// Display name
    pub display_name: String,
    /// Principal type
    pub principal_type: PrincipalType,
    /// Public key for authentication
    pub auth_pubkey: PublicKey,
    /// KYC verification level
    pub kyc_level: KycLevel,
    /// Optional DID (Decentralized Identifier) link
    pub did: Option<String>,
    /// Whether the principal is active
    pub is_active: bool,
    /// Registration timestamp
    pub registered_at: u64,
    /// Last activity timestamp
    pub last_active_at: u64,
    /// Reputation score (derived from on-chain history)
    pub reputation_score: u64,
    /// Total tasks completed (as requester)
    pub tasks_requested: u64,
    /// Total incidents reported against
    pub incidents_against: u64,
    /// Object version
    version: u64,
}

impl Principal {
    /// Create a new principal
    pub fn new(
        display_name: String,
        principal_type: PrincipalType,
        auth_pubkey: PublicKey,
        registered_at: u64,
    ) -> Self {
        // Derive principal ID from public key
        let id = PrincipalId::new(Hash::from_bytes(auth_pubkey.as_bytes()));

        Principal {
            id,
            display_name,
            principal_type,
            auth_pubkey,
            kyc_level: KycLevel::None,
            did: None,
            is_active: true,
            registered_at,
            last_active_at: registered_at,
            reputation_score: 100, // Start with base reputation
            tasks_requested: 0,
            incidents_against: 0,
            version: 1,
        }
    }

    /// Update KYC level
    pub fn set_kyc_level(&mut self, level: KycLevel) {
        self.kyc_level = level;
        self.version += 1;
    }

    /// Link a DID
    pub fn set_did(&mut self, did: String) {
        self.did = Some(did);
        self.version += 1;
    }

    /// Record task completion
    pub fn record_task_completed(&mut self) {
        self.tasks_requested += 1;
        self.reputation_score = self.reputation_score.saturating_add(1);
    }

    /// Record incident
    pub fn record_incident(&mut self) {
        self.incidents_against += 1;
        self.reputation_score = self.reputation_score.saturating_sub(10);
    }

    /// Update last activity
    pub fn record_activity(&mut self, timestamp: u64) {
        self.last_active_at = timestamp;
    }

    /// Deactivate principal
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.version += 1;
    }

    /// Check if principal can perform professional services
    pub fn can_provide_services(&self) -> bool {
        matches!(self.principal_type, PrincipalType::ServiceProvider | PrincipalType::Manufacturer)
            && self.kyc_level >= KycLevel::ProfessionalVerified
    }
}

impl ChainObject for Principal {
    fn object_id(&self) -> ObjectId {
        self.id.as_object_id()
    }

    fn object_type(&self) -> ObjectType {
        ObjectType::Principal
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

    fn create_test_principal() -> Principal {
        let keypair = KeyPair::generate();
        Principal::new(
            "Alice".to_string(),
            PrincipalType::Owner,
            keypair.public_key(),
            1000,
        )
    }

    #[test]
    fn test_principal_creation() {
        let principal = create_test_principal();
        assert_eq!(principal.display_name, "Alice");
        assert_eq!(principal.principal_type, PrincipalType::Owner);
        assert_eq!(principal.kyc_level, KycLevel::None);
        assert!(principal.is_active);
    }

    #[test]
    fn test_kyc_upgrade() {
        let mut principal = create_test_principal();
        principal.set_kyc_level(KycLevel::IdentityVerified);
        assert_eq!(principal.kyc_level, KycLevel::IdentityVerified);
    }

    #[test]
    fn test_reputation() {
        let mut principal = create_test_principal();
        let initial_rep = principal.reputation_score;

        principal.record_task_completed();
        assert_eq!(principal.reputation_score, initial_rep + 1);

        principal.record_incident();
        assert_eq!(principal.reputation_score, initial_rep + 1 - 10);
    }
}
