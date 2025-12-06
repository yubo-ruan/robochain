// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title IRobotRegistry
 * @notice Interface for the Robot Identity Registry
 * @dev Manages robot identities with TEE-based attestation
 */
interface IRobotRegistry {
    // ============ Enums ============

    enum RobotStatus {
        Inactive,
        Active,
        Suspended,
        Retired
    }

    // ============ Structs ============

    struct Robot {
        uint256 id;
        address owner;
        bytes32 hardwareFingerprint;    // Unique hardware identifier from TEE
        bytes32 firmwareHash;           // Hash of approved firmware
        string model;                   // Robot model identifier
        uint256 registeredAt;
        uint256 lastAttestationAt;
        RobotStatus status;
        uint256 reputationScore;        // Accumulated reputation
    }

    struct Attestation {
        bytes32 firmwareHash;
        uint256 timestamp;
        bytes signature;                // TEE signature
    }

    // ============ Events ============

    event RobotRegistered(
        uint256 indexed robotId,
        address indexed owner,
        bytes32 hardwareFingerprint,
        string model
    );

    event RobotTransferred(
        uint256 indexed robotId,
        address indexed from,
        address indexed to
    );

    event RobotStatusChanged(
        uint256 indexed robotId,
        RobotStatus oldStatus,
        RobotStatus newStatus
    );

    event AttestationSubmitted(
        uint256 indexed robotId,
        bytes32 firmwareHash,
        uint256 timestamp
    );

    event ReputationUpdated(
        uint256 indexed robotId,
        uint256 oldScore,
        uint256 newScore
    );

    // ============ Functions ============

    /**
     * @notice Register a new robot with TEE attestation
     * @param hardwareFingerprint Unique hardware identifier from TEE
     * @param firmwareHash Hash of the current firmware
     * @param model Robot model identifier
     * @param attestationSignature TEE signature proving authenticity
     * @return robotId The ID of the newly registered robot
     */
    function registerRobot(
        bytes32 hardwareFingerprint,
        bytes32 firmwareHash,
        string calldata model,
        bytes calldata attestationSignature
    ) external returns (uint256 robotId);

    /**
     * @notice Transfer robot ownership to a new address
     * @param robotId The robot to transfer
     * @param newOwner The new owner address
     */
    function transferOwnership(uint256 robotId, address newOwner) external;

    /**
     * @notice Update robot status
     * @param robotId The robot to update
     * @param newStatus The new status
     */
    function setRobotStatus(uint256 robotId, RobotStatus newStatus) external;

    /**
     * @notice Submit a fresh attestation proof
     * @param robotId The robot submitting attestation
     * @param firmwareHash Current firmware hash
     * @param signature TEE signature
     */
    function submitAttestation(
        uint256 robotId,
        bytes32 firmwareHash,
        bytes calldata signature
    ) external;

    /**
     * @notice Update robot reputation score (called by authorized contracts)
     * @param robotId The robot to update
     * @param delta Reputation change (positive or negative)
     */
    function updateReputation(uint256 robotId, int256 delta) external;

    /**
     * @notice Check if a firmware hash is approved
     * @param firmwareHash The firmware hash to check
     * @return Whether the firmware is approved
     */
    function isApprovedFirmware(bytes32 firmwareHash) external view returns (bool);

    /**
     * @notice Get robot details
     * @param robotId The robot ID
     * @return The robot struct
     */
    function getRobot(uint256 robotId) external view returns (Robot memory);

    /**
     * @notice Get robot ID by hardware fingerprint
     * @param hardwareFingerprint The hardware fingerprint
     * @return The robot ID (0 if not found)
     */
    function getRobotByFingerprint(bytes32 hardwareFingerprint) external view returns (uint256);

    /**
     * @notice Check if an address owns a specific robot
     * @param owner The address to check
     * @param robotId The robot ID
     * @return Whether the address owns the robot
     */
    function isOwner(address owner, uint256 robotId) external view returns (bool);

    /**
     * @notice Get all robots owned by an address
     * @param owner The owner address
     * @return Array of robot IDs
     */
    function getRobotsByOwner(address owner) external view returns (uint256[] memory);

    /**
     * @notice Check if robot attestation is fresh (within validity period)
     * @param robotId The robot to check
     * @return Whether attestation is still valid
     */
    function isAttestationValid(uint256 robotId) external view returns (bool);
}
