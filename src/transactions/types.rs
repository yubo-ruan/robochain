//! Transaction payload types

use crate::crypto::{Hash, PublicKey, Signature};
use crate::objects::*;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// All possible transaction types
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum TransactionPayload {
    // === Robot Lifecycle ===
    /// Register a new robot
    RegisterRobot(RegisterRobotTx),
    /// Update robot firmware
    UpdateFirmware(UpdateFirmwareTx),
    /// Transfer robot ownership
    TransferRobot(TransferRobotTx),
    /// Update robot status
    UpdateRobotStatus(UpdateRobotStatusTx),

    // === Space Management ===
    /// Register a new space
    RegisterSpace(RegisterSpaceTx),
    /// Update space policy graph
    SetPolicy(SetPolicyTx),
    /// Add zone to space
    AddZone(AddZoneTx),

    // === Principal Management ===
    /// Register a new principal
    RegisterPrincipal(RegisterPrincipalTx),
    /// Update KYC level
    UpdateKyc(UpdateKycTx),

    // === Capability/Permission Management ===
    /// Grant capability to robot
    GrantCapability(GrantCapabilityTx),
    /// Revoke capability from robot
    RevokeCapability(RevokeCapabilityTx),

    // === Task Lifecycle ===
    /// Post a new task
    PostTask(PostTaskTx),
    /// Accept a task
    AcceptTask(AcceptTaskTx),
    /// Start task execution
    StartTask(StartTaskTx),
    /// Submit task completion
    SubmitCompletion(SubmitCompletionTx),
    /// Cancel a task
    CancelTask(CancelTaskTx),

    // === Payment Channels ===
    /// Open a payment stream
    OpenPaymentStream(OpenPaymentStreamTx),
    /// Extend payment stream escrow
    ExtendPaymentStream(ExtendPaymentStreamTx),
    /// Withdraw from payment stream
    WithdrawPayment(WithdrawPaymentTx),
    /// Close payment stream
    ClosePaymentStream(ClosePaymentStreamTx),

    // === Data & Evidence ===
    /// Submit dataset reference
    SubmitDataset(SubmitDatasetTx),
    /// Report an incident
    ReportIncident(ReportIncidentTx),

    // === Emergency ===
    /// Emergency capability revoke
    EmergencyRevoke(EmergencyRevokeTx),
}

impl TransactionPayload {
    /// Get the read set for this transaction
    pub fn read_set(&self) -> HashSet<ObjectId> {
        match self {
            TransactionPayload::RegisterRobot(_) => HashSet::new(),
            TransactionPayload::UpdateFirmware(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.robot_id.as_object_id());
                set
            }
            TransactionPayload::TransferRobot(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.robot_id.as_object_id());
                set.insert(tx.new_owner.as_object_id());
                set
            }
            TransactionPayload::UpdateRobotStatus(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.robot_id.as_object_id());
                set
            }
            TransactionPayload::RegisterSpace(_) => HashSet::new(),
            TransactionPayload::SetPolicy(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.space_id.as_object_id());
                set
            }
            TransactionPayload::AddZone(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.space_id.as_object_id());
                set
            }
            TransactionPayload::RegisterPrincipal(_) => HashSet::new(),
            TransactionPayload::UpdateKyc(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.principal_id.as_object_id());
                set
            }
            TransactionPayload::GrantCapability(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.space_id.as_object_id());
                set.insert(tx.robot_id.as_object_id());
                set
            }
            TransactionPayload::RevokeCapability(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.policy_id.as_object_id());
                set
            }
            TransactionPayload::PostTask(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.space_id.as_object_id());
                set
            }
            TransactionPayload::AcceptTask(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.task_id.as_object_id());
                set.insert(tx.robot_id.as_object_id());
                set
            }
            TransactionPayload::StartTask(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.task_id.as_object_id());
                set
            }
            TransactionPayload::SubmitCompletion(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.task_id.as_object_id());
                set
            }
            TransactionPayload::CancelTask(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.task_id.as_object_id());
                set
            }
            TransactionPayload::OpenPaymentStream(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.payer.as_object_id());
                match &tx.payee {
                    Payee::Robot(r) => set.insert(r.as_object_id()),
                    Payee::Principal(p) => set.insert(p.as_object_id()),
                };
                set
            }
            TransactionPayload::ExtendPaymentStream(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.channel_id.as_object_id());
                set
            }
            TransactionPayload::WithdrawPayment(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.channel_id.as_object_id());
                set
            }
            TransactionPayload::ClosePaymentStream(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.channel_id.as_object_id());
                set
            }
            TransactionPayload::SubmitDataset(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.source_robot.as_object_id());
                set
            }
            TransactionPayload::ReportIncident(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.task_id.as_object_id());
                set.insert(tx.robot_id.as_object_id());
                set
            }
            TransactionPayload::EmergencyRevoke(tx) => {
                let mut set = HashSet::new();
                match &tx.scope {
                    EmergencyScope::SingleRobot(r) => {
                        set.insert(r.as_object_id());
                    }
                    EmergencyScope::AllInSpace(s) => {
                        set.insert(s.as_object_id());
                    }
                    EmergencyScope::Global => {}
                }
                set
            }
        }
    }

    /// Get the write set for this transaction
    pub fn write_set(&self) -> HashSet<ObjectId> {
        match self {
            TransactionPayload::RegisterRobot(tx) => {
                // New object will be created - return empty for now
                // The actual ID is computed during execution
                HashSet::new()
            }
            TransactionPayload::UpdateFirmware(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.robot_id.as_object_id());
                set
            }
            TransactionPayload::TransferRobot(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.robot_id.as_object_id());
                set
            }
            TransactionPayload::UpdateRobotStatus(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.robot_id.as_object_id());
                set
            }
            TransactionPayload::RegisterSpace(_) => HashSet::new(),
            TransactionPayload::SetPolicy(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.space_id.as_object_id());
                set
            }
            TransactionPayload::AddZone(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.space_id.as_object_id());
                set
            }
            TransactionPayload::RegisterPrincipal(_) => HashSet::new(),
            TransactionPayload::UpdateKyc(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.principal_id.as_object_id());
                set
            }
            TransactionPayload::GrantCapability(_) => {
                // Creates new policy object
                HashSet::new()
            }
            TransactionPayload::RevokeCapability(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.policy_id.as_object_id());
                set
            }
            TransactionPayload::PostTask(_) => HashSet::new(),
            TransactionPayload::AcceptTask(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.task_id.as_object_id());
                set
            }
            TransactionPayload::StartTask(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.task_id.as_object_id());
                set
            }
            TransactionPayload::SubmitCompletion(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.task_id.as_object_id());
                set
            }
            TransactionPayload::CancelTask(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.task_id.as_object_id());
                set
            }
            TransactionPayload::OpenPaymentStream(_) => HashSet::new(),
            TransactionPayload::ExtendPaymentStream(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.channel_id.as_object_id());
                set
            }
            TransactionPayload::WithdrawPayment(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.channel_id.as_object_id());
                set
            }
            TransactionPayload::ClosePaymentStream(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.channel_id.as_object_id());
                set
            }
            TransactionPayload::SubmitDataset(_) => HashSet::new(),
            TransactionPayload::ReportIncident(tx) => {
                let mut set = HashSet::new();
                set.insert(tx.task_id.as_object_id());
                set.insert(tx.robot_id.as_object_id());
                set
            }
            TransactionPayload::EmergencyRevoke(tx) => {
                let mut set = HashSet::new();
                match &tx.scope {
                    EmergencyScope::SingleRobot(r) => {
                        set.insert(r.as_object_id());
                    }
                    EmergencyScope::AllInSpace(s) => {
                        set.insert(s.as_object_id());
                    }
                    EmergencyScope::Global => {}
                }
                set
            }
        }
    }
}

// === Robot Transactions ===

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct RegisterRobotTx {
    pub manufacturer: String,
    pub model: String,
    pub hw_attestation: HardwareAttestation,
    pub capabilities: Vec<Capability>,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct UpdateFirmwareTx {
    pub robot_id: RobotId,
    pub new_firmware_hash: Hash,
    pub vendor_signature: Signature,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct TransferRobotTx {
    pub robot_id: RobotId,
    pub new_owner: PrincipalId,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct UpdateRobotStatusTx {
    pub robot_id: RobotId,
    pub new_status: RobotStatus,
}

// === Space Transactions ===

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct RegisterSpaceTx {
    pub name: String,
    pub space_type: SpaceType,
    pub geo_region: GeoRegion,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SetPolicyTx {
    pub space_id: SpaceId,
    pub policy_graph_hash: Hash,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct AddZoneTx {
    pub space_id: SpaceId,
    pub zone: Zone,
}

// === Principal Transactions ===

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct RegisterPrincipalTx {
    pub display_name: String,
    pub principal_type: PrincipalType,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct UpdateKycTx {
    pub principal_id: PrincipalId,
    pub new_kyc_level: KycLevel,
    /// Signature from KYC provider
    pub kyc_attestation: Signature,
}

// === Capability Transactions ===

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct GrantCapabilityTx {
    pub space_id: SpaceId,
    pub robot_id: RobotId,
    pub capabilities: Vec<Capability>,
    pub zones: Vec<String>,
    pub conditions: Vec<PolicyCondition>,
    pub revocable: bool,
    pub expires_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct RevokeCapabilityTx {
    pub policy_id: PolicyId,
}

// === Task Transactions ===

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct PostTaskTx {
    pub space_id: SpaceId,
    pub description: String,
    pub required_capabilities: Vec<Capability>,
    pub reward: u64,
    pub deadline: u64,
    pub safety_constraints: Vec<SafetyConstraint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct AcceptTaskTx {
    pub task_id: TaskId,
    pub robot_id: RobotId,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct StartTaskTx {
    pub task_id: TaskId,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SubmitCompletionTx {
    pub task_id: TaskId,
    pub evidence_merkle_root: Hash,
    pub evidence_uri: String,
    pub safety_proofs: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct CancelTaskTx {
    pub task_id: TaskId,
}

// === Payment Transactions ===

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct OpenPaymentStreamTx {
    pub payer: PrincipalId,
    pub payee: Payee,
    pub rate_per_second: u64,
    pub initial_escrow: u64,
    pub task_id: Option<TaskId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ExtendPaymentStreamTx {
    pub channel_id: PaymentChannelId,
    pub additional_escrow: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct WithdrawPaymentTx {
    pub channel_id: PaymentChannelId,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ClosePaymentStreamTx {
    pub channel_id: PaymentChannelId,
}

// === Data Transactions ===

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SubmitDatasetTx {
    pub dataset_type: DatasetType,
    pub source_robot: RobotId,
    pub off_chain_uri: String,
    pub merkle_root: Hash,
    pub size_bytes: u64,
    pub proofs: Vec<DataProof>,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct ReportIncidentTx {
    pub task_id: TaskId,
    pub robot_id: RobotId,
    pub evidence_merkle_root: Hash,
    pub evidence_uri: String,
    pub description: String,
}

// === Emergency Transactions ===

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum EmergencyScope {
    SingleRobot(RobotId),
    AllInSpace(SpaceId),
    Global,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum EmergencyTrigger {
    PanicButton,
    SafetyOracle,
    OwnerOverride,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct EmergencyRevokeTx {
    pub trigger: EmergencyTrigger,
    pub scope: EmergencyScope,
    pub reason: String,
}
