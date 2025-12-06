//! Dataset (off-chain data reference) object definition

use super::{ChainObject, DatasetId, ObjectId, ObjectType, PolicyId, PrincipalId, RobotId};
use crate::crypto::{Hash, Signature};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

/// Type of dataset
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum DatasetType {
    /// Task completion evidence
    TaskEvidence,
    /// Robot trajectory data
    Trajectory,
    /// Sensor logs (IMU, etc.)
    SensorLogs,
    /// Video/image data
    Visual,
    /// Safety attestation proof
    SafetyProof,
    /// Incident report evidence
    IncidentEvidence,
    /// Training data sample
    TrainingData,
}

/// Proof that data satisfies certain criteria
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct DataProof {
    /// Type of proof
    pub proof_type: ProofType,
    /// Serialized proof data
    pub proof_data: Vec<u8>,
    /// Verifier/auditor signature
    pub verifier_signature: Option<Signature>,
}

/// Types of proofs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum ProofType {
    /// Zero-knowledge proof
    ZkProof,
    /// Merkle inclusion proof
    MerkleProof,
    /// Signed attestation
    SignedAttestation,
    /// Hash commitment
    HashCommitment,
}

/// Access control rule for dataset
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct AccessRule {
    /// Who can access
    pub accessor: AccessorType,
    /// Under what conditions
    pub condition: AccessCondition,
    /// What level of access
    pub access_level: AccessLevel,
}

/// Who can access the data
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum AccessorType {
    /// The data owner
    Owner,
    /// Specific principal
    Principal(PrincipalId),
    /// Any insurer
    AnyInsurer,
    /// The manufacturer of the robot
    Manufacturer,
    /// Anyone
    Public,
}

/// Conditions for access
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum AccessCondition {
    /// Always allowed
    Always,
    /// Requires incident report
    IncidentReported,
    /// Requires multi-sig approval
    MultiSigApproval(Vec<PrincipalId>),
    /// Within time period
    TimeLimited { until: u64 },
}

/// Level of access granted
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum AccessLevel {
    /// Full access to raw data
    Full,
    /// Access to redacted/anonymized data
    Redacted,
    /// Access to summary/statistics only
    SummaryOnly,
    /// Access to proof verification only
    ProofOnly,
}

/// A reference to off-chain data with on-chain commitment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    /// Unique dataset ID
    pub id: DatasetId,
    /// Type of dataset
    pub dataset_type: DatasetType,
    /// Robot that generated the data
    pub source_robot: RobotId,
    /// Owner of the data
    pub owner: PrincipalId,
    /// Off-chain storage URI (e.g., IPFS, S3)
    pub off_chain_uri: String,
    /// Merkle root of the data
    pub merkle_root: Hash,
    /// Size in bytes
    pub size_bytes: u64,
    /// Creation timestamp
    pub created_at: u64,
    /// Associated proofs
    pub proofs: Vec<DataProof>,
    /// Access control rules
    pub access_rules: Vec<AccessRule>,
    /// Whether data is encrypted
    pub is_encrypted: bool,
    /// Encryption key ID (if encrypted)
    pub encryption_key_id: Option<Hash>,
    /// Object version
    version: u64,
}

impl Dataset {
    /// Create a new dataset reference
    pub fn new(
        dataset_type: DatasetType,
        source_robot: RobotId,
        owner: PrincipalId,
        off_chain_uri: String,
        merkle_root: Hash,
        size_bytes: u64,
        created_at: u64,
    ) -> Self {
        // Derive dataset ID
        let id_bytes = borsh::to_vec(&(&merkle_root, &source_robot, created_at)).unwrap_or_default();
        let id = DatasetId::from_bytes(&id_bytes);

        Dataset {
            id,
            dataset_type,
            source_robot,
            owner,
            off_chain_uri,
            merkle_root,
            size_bytes,
            created_at,
            proofs: Vec::new(),
            access_rules: Vec::new(),
            is_encrypted: false,
            encryption_key_id: None,
            version: 1,
        }
    }

    /// Add a proof
    pub fn add_proof(&mut self, proof: DataProof) {
        self.proofs.push(proof);
        self.version += 1;
    }

    /// Add an access rule
    pub fn add_access_rule(&mut self, rule: AccessRule) {
        self.access_rules.push(rule);
        self.version += 1;
    }

    /// Set encryption
    pub fn set_encrypted(&mut self, key_id: Hash) {
        self.is_encrypted = true;
        self.encryption_key_id = Some(key_id);
        self.version += 1;
    }

    /// Check if principal can access at given level
    pub fn can_access(&self, principal: &PrincipalId, level: &AccessLevel, timestamp: u64) -> bool {
        // Owner always has full access
        if principal == &self.owner && *level == AccessLevel::Full {
            return true;
        }

        for rule in &self.access_rules {
            let accessor_matches = match &rule.accessor {
                AccessorType::Owner => principal == &self.owner,
                AccessorType::Principal(p) => principal == p,
                AccessorType::Public => true,
                _ => false, // Other types need external verification
            };

            if !accessor_matches {
                continue;
            }

            let condition_met = match &rule.condition {
                AccessCondition::Always => true,
                AccessCondition::TimeLimited { until } => timestamp < *until,
                _ => false, // Other conditions need external verification
            };

            if condition_met && rule.access_level == *level {
                return true;
            }
        }
        false
    }

    /// Get all proofs of a specific type
    pub fn get_proofs_by_type(&self, proof_type: &ProofType) -> Vec<&DataProof> {
        self.proofs.iter().filter(|p| &p.proof_type == proof_type).collect()
    }
}

impl ChainObject for Dataset {
    fn object_id(&self) -> ObjectId {
        self.id.as_object_id()
    }

    fn object_type(&self) -> ObjectType {
        ObjectType::Dataset
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

    fn create_test_dataset() -> Dataset {
        let robot_id = RobotId::from_bytes(b"test_robot");
        let owner = PrincipalId::from_bytes(b"owner");
        let merkle_root = Hash::from_bytes(b"merkle_root");

        Dataset::new(
            DatasetType::TaskEvidence,
            robot_id,
            owner,
            "ipfs://QmTest123".to_string(),
            merkle_root,
            1024 * 1024, // 1MB
            1000,
        )
    }

    #[test]
    fn test_dataset_creation() {
        let dataset = create_test_dataset();
        assert_eq!(dataset.dataset_type, DatasetType::TaskEvidence);
        assert_eq!(dataset.size_bytes, 1024 * 1024);
        assert!(!dataset.is_encrypted);
    }

    #[test]
    fn test_add_proof() {
        let mut dataset = create_test_dataset();
        let proof = DataProof {
            proof_type: ProofType::ZkProof,
            proof_data: vec![1, 2, 3, 4],
            verifier_signature: None,
        };
        dataset.add_proof(proof);
        assert_eq!(dataset.proofs.len(), 1);
    }

    #[test]
    fn test_access_control() {
        let mut dataset = create_test_dataset();
        let owner = PrincipalId::from_bytes(b"owner");
        let other = PrincipalId::from_bytes(b"other");

        // Owner always has full access
        assert!(dataset.can_access(&owner, &AccessLevel::Full, 1000));

        // Other has no access by default
        assert!(!dataset.can_access(&other, &AccessLevel::Full, 1000));

        // Add public access rule
        dataset.add_access_rule(AccessRule {
            accessor: AccessorType::Public,
            condition: AccessCondition::Always,
            access_level: AccessLevel::SummaryOnly,
        });

        assert!(dataset.can_access(&other, &AccessLevel::SummaryOnly, 1000));
    }
}
