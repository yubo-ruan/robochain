//! Policy (permission grant) object definition

use super::{Capability, ChainObject, ObjectId, ObjectType, PolicyId, PrincipalId, RobotId, SpaceId};
use crate::crypto::Hash;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Condition that must be met for policy to be active
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum PolicyCondition {
    /// Policy is always active
    Always,
    /// Policy active during time range
    TimeRange { start: u64, end: u64 },
    /// Policy active only on specific days (0 = Sunday)
    DaysOfWeek(Vec<u8>),
    /// Policy requires additional approval each time
    RequiresApproval,
    /// Policy active only when specific principal is present
    PrincipalPresent(PrincipalId),
    /// Custom condition with string identifier
    Custom(String),
}

/// Type of policy grant
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum PolicyGrantType {
    /// Allow the capability
    Allow,
    /// Explicitly deny the capability
    Deny,
}

/// A permission grant from a principal to a robot/entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    /// Unique policy ID
    pub id: PolicyId,
    /// Space this policy applies to
    pub space_id: SpaceId,
    /// Principal granting the permission
    pub grantor: PrincipalId,
    /// Robot receiving the permission
    pub grantee_robot: Option<RobotId>,
    /// Principal receiving the permission (for delegation)
    pub grantee_principal: Option<PrincipalId>,
    /// Capabilities being granted
    pub capabilities: HashSet<Capability>,
    /// Grant type (allow/deny)
    pub grant_type: PolicyGrantType,
    /// Zones where policy applies (empty = all zones)
    pub zones: Vec<String>,
    /// Conditions for policy activation
    pub conditions: Vec<PolicyCondition>,
    /// Whether policy can be revoked
    pub revocable: bool,
    /// Whether policy is currently active
    pub is_active: bool,
    /// Creation timestamp
    pub created_at: u64,
    /// Expiration timestamp (0 = never)
    pub expires_at: u64,
    /// Revocation timestamp (if revoked)
    pub revoked_at: Option<u64>,
    /// Object version
    version: u64,
}

impl Policy {
    /// Create a new policy
    pub fn new(
        space_id: SpaceId,
        grantor: PrincipalId,
        grantee_robot: Option<RobotId>,
        grantee_principal: Option<PrincipalId>,
        capabilities: HashSet<Capability>,
        grant_type: PolicyGrantType,
        revocable: bool,
        created_at: u64,
        expires_at: u64,
    ) -> Self {
        // Derive policy ID
        let id_bytes = borsh::to_vec(&(
            &space_id,
            &grantor,
            &grantee_robot,
            &grantee_principal,
            created_at,
        ))
        .unwrap_or_default();
        let id = PolicyId::from_bytes(&id_bytes);

        Policy {
            id,
            space_id,
            grantor,
            grantee_robot,
            grantee_principal,
            capabilities,
            grant_type,
            zones: Vec::new(),
            conditions: vec![PolicyCondition::Always],
            revocable,
            is_active: true,
            created_at,
            expires_at,
            revoked_at: None,
            version: 1,
        }
    }

    /// Restrict policy to specific zones
    pub fn set_zones(&mut self, zones: Vec<String>) {
        self.zones = zones;
        self.version += 1;
    }

    /// Set conditions
    pub fn set_conditions(&mut self, conditions: Vec<PolicyCondition>) {
        self.conditions = conditions;
        self.version += 1;
    }

    /// Revoke the policy
    pub fn revoke(&mut self, timestamp: u64) -> Result<(), PolicyError> {
        if !self.revocable {
            return Err(PolicyError::NotRevocable);
        }
        if !self.is_active {
            return Err(PolicyError::AlreadyRevoked);
        }
        self.is_active = false;
        self.revoked_at = Some(timestamp);
        self.version += 1;
        Ok(())
    }

    /// Check if policy is valid at given time
    pub fn is_valid_at(&self, timestamp: u64) -> bool {
        if !self.is_active {
            return false;
        }
        if self.expires_at > 0 && timestamp >= self.expires_at {
            return false;
        }

        // Check time-based conditions
        for condition in &self.conditions {
            match condition {
                PolicyCondition::TimeRange { start, end } => {
                    if timestamp < *start || timestamp >= *end {
                        return false;
                    }
                }
                PolicyCondition::Always => {}
                _ => {} // Other conditions checked elsewhere
            }
        }
        true
    }

    /// Check if policy grants a specific capability
    pub fn grants_capability(&self, capability: &Capability) -> bool {
        self.is_active && self.grant_type == PolicyGrantType::Allow && self.capabilities.contains(capability)
    }

    /// Check if policy applies to a specific zone
    pub fn applies_to_zone(&self, zone_id: &str) -> bool {
        self.zones.is_empty() || self.zones.iter().any(|z| z == zone_id)
    }

    /// Check if policy applies to a specific robot
    pub fn applies_to_robot(&self, robot_id: &RobotId) -> bool {
        self.grantee_robot.as_ref().is_some_and(|r| r == robot_id)
    }
}

/// Policy-related errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyError {
    NotRevocable,
    AlreadyRevoked,
    Expired,
    Unauthorized,
}

impl ChainObject for Policy {
    fn object_id(&self) -> ObjectId {
        self.id.as_object_id()
    }

    fn object_type(&self) -> ObjectType {
        ObjectType::Policy
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

    fn create_test_policy() -> Policy {
        let space_id = SpaceId::from_bytes(b"test_space");
        let grantor = PrincipalId::from_bytes(b"owner");
        let robot_id = RobotId::from_bytes(b"robot");
        let mut capabilities = HashSet::new();
        capabilities.insert(Capability::Cleaning);
        capabilities.insert(Capability::Navigation);

        Policy::new(
            space_id,
            grantor,
            Some(robot_id),
            None,
            capabilities,
            PolicyGrantType::Allow,
            true,
            1000,
            0, // never expires
        )
    }

    #[test]
    fn test_policy_creation() {
        let policy = create_test_policy();
        assert!(policy.is_active);
        assert!(policy.revocable);
        assert_eq!(policy.grant_type, PolicyGrantType::Allow);
    }

    #[test]
    fn test_policy_grants_capability() {
        let policy = create_test_policy();
        assert!(policy.grants_capability(&Capability::Cleaning));
        assert!(policy.grants_capability(&Capability::Navigation));
        assert!(!policy.grants_capability(&Capability::Cooking));
    }

    #[test]
    fn test_policy_revocation() {
        let mut policy = create_test_policy();
        assert!(policy.revoke(2000).is_ok());
        assert!(!policy.is_active);
        assert_eq!(policy.revoked_at, Some(2000));

        // Cannot revoke again
        assert!(policy.revoke(3000).is_err());
    }

    #[test]
    fn test_zone_restriction() {
        let mut policy = create_test_policy();

        // Initially applies to all zones
        assert!(policy.applies_to_zone("living_room"));
        assert!(policy.applies_to_zone("kitchen"));

        // Restrict to specific zones
        policy.set_zones(vec!["living_room".to_string()]);
        assert!(policy.applies_to_zone("living_room"));
        assert!(!policy.applies_to_zone("kitchen"));
    }
}
