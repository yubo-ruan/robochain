//! Task object definition

use super::{Capability, ChainObject, DatasetId, ObjectId, ObjectType, PrincipalId, RobotId, SpaceId, TaskId};
use crate::crypto::Hash;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Status of a task
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum TaskStatus {
    /// Task is posted and awaiting robot acceptance
    Posted,
    /// Task has been accepted by a robot
    Accepted,
    /// Task is currently being executed
    InProgress,
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed,
    /// Task was cancelled
    Cancelled,
    /// Task is disputed
    Disputed,
}

/// Safety constraint for a task
#[derive(Debug, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct SafetyConstraint {
    /// Constraint type
    pub constraint_type: ConstraintType,
    /// Constraint parameters (serialized)
    pub parameters: Vec<u8>,
    /// Whether violation should halt the task
    pub halt_on_violation: bool,
}

/// Types of safety constraints
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub enum ConstraintType {
    /// Robot must not enter specified zones
    ZoneExclusion,
    /// Maximum velocity limit
    VelocityLimit,
    /// Maximum force limit
    ForceLimit,
    /// Time window constraint
    TimeWindow,
    /// No interaction with specific objects
    ObjectExclusion,
    /// Custom constraint with string identifier
    Custom(String),
}

/// A job to be performed by a robot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique task ID
    pub id: TaskId,
    /// Space where task should be performed
    pub space_id: SpaceId,
    /// Principal who requested the task
    pub requester: PrincipalId,
    /// Required capabilities
    pub required_capabilities: HashSet<Capability>,
    /// Reward amount (in micro-ROUSD)
    pub reward: u64,
    /// Task description
    pub description: String,
    /// Safety constraints
    pub safety_constraints: Vec<SafetyConstraint>,
    /// Current status
    pub status: TaskStatus,
    /// Robot assigned to task (if any)
    pub assigned_robot: Option<RobotId>,
    /// Evidence commitment for completion
    pub completion_evidence: Option<DatasetId>,
    /// Created timestamp
    pub created_at: u64,
    /// Deadline timestamp
    pub deadline: u64,
    /// Accepted timestamp
    pub accepted_at: Option<u64>,
    /// Completed timestamp
    pub completed_at: Option<u64>,
    /// Object version
    version: u64,
}

impl Task {
    /// Create a new task
    pub fn new(
        space_id: SpaceId,
        requester: PrincipalId,
        description: String,
        required_capabilities: HashSet<Capability>,
        reward: u64,
        deadline: u64,
        created_at: u64,
    ) -> Self {
        // Derive task ID from creation params
        let id_bytes = borsh::to_vec(&(&space_id, &requester, &description, created_at)).unwrap_or_default();
        let id = TaskId::from_bytes(&id_bytes);

        Task {
            id,
            space_id,
            requester,
            required_capabilities,
            reward,
            description,
            safety_constraints: Vec::new(),
            status: TaskStatus::Posted,
            assigned_robot: None,
            completion_evidence: None,
            created_at,
            deadline,
            accepted_at: None,
            completed_at: None,
            version: 1,
        }
    }

    /// Add a safety constraint
    pub fn add_constraint(&mut self, constraint: SafetyConstraint) {
        self.safety_constraints.push(constraint);
        self.version += 1;
    }

    /// Accept task by a robot
    pub fn accept(&mut self, robot_id: RobotId, timestamp: u64) -> Result<(), TaskError> {
        if self.status != TaskStatus::Posted {
            return Err(TaskError::InvalidStatusTransition);
        }
        self.status = TaskStatus::Accepted;
        self.assigned_robot = Some(robot_id);
        self.accepted_at = Some(timestamp);
        self.version += 1;
        Ok(())
    }

    /// Start task execution
    pub fn start(&mut self) -> Result<(), TaskError> {
        if self.status != TaskStatus::Accepted {
            return Err(TaskError::InvalidStatusTransition);
        }
        self.status = TaskStatus::InProgress;
        self.version += 1;
        Ok(())
    }

    /// Complete task with evidence
    pub fn complete(&mut self, evidence_id: DatasetId, timestamp: u64) -> Result<(), TaskError> {
        if self.status != TaskStatus::InProgress {
            return Err(TaskError::InvalidStatusTransition);
        }
        self.status = TaskStatus::Completed;
        self.completion_evidence = Some(evidence_id);
        self.completed_at = Some(timestamp);
        self.version += 1;
        Ok(())
    }

    /// Fail task
    pub fn fail(&mut self, timestamp: u64) -> Result<(), TaskError> {
        if !matches!(self.status, TaskStatus::Accepted | TaskStatus::InProgress) {
            return Err(TaskError::InvalidStatusTransition);
        }
        self.status = TaskStatus::Failed;
        self.completed_at = Some(timestamp);
        self.version += 1;
        Ok(())
    }

    /// Cancel task
    pub fn cancel(&mut self) -> Result<(), TaskError> {
        if !matches!(self.status, TaskStatus::Posted | TaskStatus::Accepted) {
            return Err(TaskError::InvalidStatusTransition);
        }
        self.status = TaskStatus::Cancelled;
        self.version += 1;
        Ok(())
    }

    /// Mark as disputed
    pub fn dispute(&mut self) -> Result<(), TaskError> {
        if self.status != TaskStatus::Completed {
            return Err(TaskError::InvalidStatusTransition);
        }
        self.status = TaskStatus::Disputed;
        self.version += 1;
        Ok(())
    }

    /// Check if task is still valid (not expired)
    pub fn is_valid(&self, current_time: u64) -> bool {
        self.status == TaskStatus::Posted && current_time < self.deadline
    }

    /// Check if robot has required capabilities
    pub fn robot_can_perform(&self, robot_capabilities: &HashSet<Capability>) -> bool {
        self.required_capabilities.is_subset(robot_capabilities)
    }
}

/// Task-related errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskError {
    InvalidStatusTransition,
    TaskExpired,
    InsufficientCapabilities,
    Unauthorized,
}

impl ChainObject for Task {
    fn object_id(&self) -> ObjectId {
        self.id.as_object_id()
    }

    fn object_type(&self) -> ObjectType {
        ObjectType::Task
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

    fn create_test_task() -> Task {
        let space_id = SpaceId::from_bytes(b"test_space");
        let requester = PrincipalId::from_bytes(b"test_requester");
        let mut capabilities = HashSet::new();
        capabilities.insert(Capability::Cleaning);

        Task::new(
            space_id,
            requester,
            "Clean the living room".to_string(),
            capabilities,
            1000000, // 1 ROUSD
            2000,    // deadline
            1000,    // created_at
        )
    }

    #[test]
    fn test_task_creation() {
        let task = create_test_task();
        assert_eq!(task.status, TaskStatus::Posted);
        assert_eq!(task.reward, 1000000);
        assert!(task.assigned_robot.is_none());
    }

    #[test]
    fn test_task_lifecycle() {
        let mut task = create_test_task();
        let robot_id = RobotId::from_bytes(b"test_robot");
        let dataset_id = DatasetId::from_bytes(b"evidence");

        // Accept
        assert!(task.accept(robot_id, 1100).is_ok());
        assert_eq!(task.status, TaskStatus::Accepted);
        assert_eq!(task.assigned_robot, Some(robot_id));

        // Start
        assert!(task.start().is_ok());
        assert_eq!(task.status, TaskStatus::InProgress);

        // Complete
        assert!(task.complete(dataset_id, 1200).is_ok());
        assert_eq!(task.status, TaskStatus::Completed);
        assert_eq!(task.completion_evidence, Some(dataset_id));
    }

    #[test]
    fn test_invalid_transitions() {
        let mut task = create_test_task();

        // Cannot start without accepting
        assert!(task.start().is_err());

        // Cannot complete without starting
        let dataset_id = DatasetId::from_bytes(b"evidence");
        assert!(task.complete(dataset_id, 1200).is_err());
    }
}
