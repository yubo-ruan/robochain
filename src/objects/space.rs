//! Space (home/location) object definition

use super::{ChainObject, ObjectId, ObjectType, PrincipalId, SpaceId};
use crate::crypto::Hash;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Type of space
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum SpaceType {
    /// Residential home
    Residential,
    /// Commercial office
    Commercial,
    /// Healthcare facility
    Healthcare,
    /// Educational institution
    Educational,
    /// Industrial facility
    Industrial,
    /// Public space
    Public,
}

/// Geographic region for the space
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct GeoRegion {
    /// Country code (ISO 3166-1 alpha-2)
    pub country: String,
    /// State/province code
    pub region: Option<String>,
    /// City name
    pub city: Option<String>,
    /// Approximate coordinates (lat, lon) - not exact for privacy
    pub approx_coords: Option<(f64, f64)>,
}

/// A room or zone within a space
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Zone {
    /// Zone identifier (unique within space)
    pub zone_id: String,
    /// Zone name (e.g., "Living Room", "Kitchen")
    pub name: String,
    /// Privacy level (higher = more restricted)
    pub privacy_level: u8,
    /// Whether robots can enter by default
    pub robot_accessible: bool,
}

/// A home or physical space where robots operate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Space {
    /// Unique space ID
    pub id: SpaceId,
    /// Human-readable name
    pub name: String,
    /// Type of space
    pub space_type: SpaceType,
    /// Owner principal IDs
    pub owners: HashSet<PrincipalId>,
    /// Geographic region
    pub geo_region: GeoRegion,
    /// Zones within the space
    pub zones: Vec<Zone>,
    /// Hash of the full policy graph (stored off-chain)
    pub policy_graph_hash: Hash,
    /// Whether the space is active
    pub is_active: bool,
    /// Creation timestamp
    pub created_at: u64,
    /// Last update timestamp
    pub updated_at: u64,
    /// Object version
    version: u64,
}

impl Space {
    /// Create a new space
    pub fn new(
        name: String,
        space_type: SpaceType,
        owner: PrincipalId,
        geo_region: GeoRegion,
        created_at: u64,
    ) -> Self {
        // Derive space ID from creation params
        let id_bytes = borsh::to_vec(&(&name, &owner, created_at)).unwrap_or_default();
        let id = SpaceId::from_bytes(&id_bytes);

        let mut owners = HashSet::new();
        owners.insert(owner);

        Space {
            id,
            name,
            space_type,
            owners,
            geo_region,
            zones: Vec::new(),
            policy_graph_hash: Hash::zero(),
            is_active: true,
            created_at,
            updated_at: created_at,
            version: 1,
        }
    }

    /// Add an owner
    pub fn add_owner(&mut self, owner: PrincipalId) {
        self.owners.insert(owner);
        self.version += 1;
    }

    /// Remove an owner
    pub fn remove_owner(&mut self, owner: &PrincipalId) -> bool {
        if self.owners.len() > 1 {
            self.owners.remove(owner);
            self.version += 1;
            true
        } else {
            false // Cannot remove last owner
        }
    }

    /// Check if principal is an owner
    pub fn is_owner(&self, principal: &PrincipalId) -> bool {
        self.owners.contains(principal)
    }

    /// Add a zone
    pub fn add_zone(&mut self, zone: Zone) {
        self.zones.push(zone);
        self.version += 1;
    }

    /// Get zone by ID
    pub fn get_zone(&self, zone_id: &str) -> Option<&Zone> {
        self.zones.iter().find(|z| z.zone_id == zone_id)
    }

    /// Update policy graph hash
    pub fn set_policy_graph(&mut self, hash: Hash, timestamp: u64) {
        self.policy_graph_hash = hash;
        self.updated_at = timestamp;
        self.version += 1;
    }

    /// Deactivate space
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.version += 1;
    }
}

impl ChainObject for Space {
    fn object_id(&self) -> ObjectId {
        self.id.as_object_id()
    }

    fn object_type(&self) -> ObjectType {
        ObjectType::Space
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
    use crate::crypto::Hash;

    fn create_test_space() -> Space {
        let owner = PrincipalId::from_bytes(b"test_owner");
        let geo = GeoRegion {
            country: "US".to_string(),
            region: Some("CA".to_string()),
            city: Some("San Francisco".to_string()),
            approx_coords: Some((37.7749, -122.4194)),
        };
        Space::new(
            "My Home".to_string(),
            SpaceType::Residential,
            owner,
            geo,
            1000,
        )
    }

    #[test]
    fn test_space_creation() {
        let space = create_test_space();
        assert_eq!(space.name, "My Home");
        assert_eq!(space.space_type, SpaceType::Residential);
        assert!(space.is_active);
        assert_eq!(space.owners.len(), 1);
    }

    #[test]
    fn test_add_zone() {
        let mut space = create_test_space();
        let zone = Zone {
            zone_id: "living_room".to_string(),
            name: "Living Room".to_string(),
            privacy_level: 1,
            robot_accessible: true,
        };
        space.add_zone(zone);
        assert_eq!(space.zones.len(), 1);
        assert!(space.get_zone("living_room").is_some());
    }

    #[test]
    fn test_owner_management() {
        let mut space = create_test_space();
        let second_owner = PrincipalId::from_bytes(b"second_owner");

        space.add_owner(second_owner);
        assert_eq!(space.owners.len(), 2);

        assert!(space.remove_owner(&second_owner));
        assert_eq!(space.owners.len(), 1);

        // Cannot remove last owner
        let first_owner = PrincipalId::from_bytes(b"test_owner");
        assert!(!space.remove_owner(&first_owner));
    }
}
