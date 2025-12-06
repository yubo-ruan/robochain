// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "../interfaces/IRobotRegistry.sol";

/**
 * @title RobotRegistry
 * @notice Manages robot identities with TEE-based attestation
 * @dev Robots must prove they run approved firmware via TEE signatures
 */
contract RobotRegistry is IRobotRegistry, AccessControl, Pausable, ReentrancyGuard {
    // ============ Constants ============

    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    bytes32 public constant FIRMWARE_MANAGER_ROLE = keccak256("FIRMWARE_MANAGER_ROLE");
    bytes32 public constant REPUTATION_MANAGER_ROLE = keccak256("REPUTATION_MANAGER_ROLE");

    uint256 public constant ATTESTATION_VALIDITY_PERIOD = 30 days;
    uint256 public constant MAX_REPUTATION = 10000;
    uint256 public constant INITIAL_REPUTATION = 1000;

    // ============ State Variables ============

    // Robot ID counter
    uint256 private _nextRobotId = 1;

    // Robot ID => Robot data
    mapping(uint256 => Robot) private _robots;

    // Hardware fingerprint => Robot ID (to prevent duplicate registrations)
    mapping(bytes32 => uint256) private _fingerprintToRobotId;

    // Owner => Robot IDs
    mapping(address => uint256[]) private _ownerRobots;

    // Owner => Robot ID => Index in _ownerRobots array
    mapping(address => mapping(uint256 => uint256)) private _ownerRobotIndex;

    // Approved firmware hashes
    mapping(bytes32 => bool) private _approvedFirmware;

    // Robot ID => Attestation history
    mapping(uint256 => Attestation[]) private _attestationHistory;

    // TEE public keys for verification (manufacturer => pubkey)
    mapping(bytes32 => bytes) private _teePubKeys;

    // ============ Constructor ============

    constructor(address admin) {
        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _grantRole(ADMIN_ROLE, admin);
        _grantRole(FIRMWARE_MANAGER_ROLE, admin);
        _grantRole(REPUTATION_MANAGER_ROLE, admin);
    }

    // ============ External Functions ============

    /**
     * @inheritdoc IRobotRegistry
     */
    function registerRobot(
        bytes32 hardwareFingerprint,
        bytes32 firmwareHash,
        string calldata model,
        bytes calldata attestationSignature
    ) external whenNotPaused nonReentrant returns (uint256 robotId) {
        // Check fingerprint not already registered
        require(
            _fingerprintToRobotId[hardwareFingerprint] == 0,
            "RobotRegistry: fingerprint already registered"
        );

        // Verify firmware is approved
        require(
            _approvedFirmware[firmwareHash],
            "RobotRegistry: firmware not approved"
        );

        // Verify TEE attestation signature
        require(
            _verifyAttestation(hardwareFingerprint, firmwareHash, attestationSignature),
            "RobotRegistry: invalid attestation"
        );

        // Create robot
        robotId = _nextRobotId++;

        _robots[robotId] = Robot({
            id: robotId,
            owner: msg.sender,
            hardwareFingerprint: hardwareFingerprint,
            firmwareHash: firmwareHash,
            model: model,
            registeredAt: block.timestamp,
            lastAttestationAt: block.timestamp,
            status: RobotStatus.Active,
            reputationScore: INITIAL_REPUTATION
        });

        // Store attestation
        _attestationHistory[robotId].push(Attestation({
            firmwareHash: firmwareHash,
            timestamp: block.timestamp,
            signature: attestationSignature
        }));

        // Update mappings
        _fingerprintToRobotId[hardwareFingerprint] = robotId;
        _ownerRobots[msg.sender].push(robotId);
        _ownerRobotIndex[msg.sender][robotId] = _ownerRobots[msg.sender].length - 1;

        emit RobotRegistered(robotId, msg.sender, hardwareFingerprint, model);
    }

    /**
     * @inheritdoc IRobotRegistry
     */
    function transferOwnership(uint256 robotId, address newOwner) external whenNotPaused {
        require(newOwner != address(0), "RobotRegistry: invalid new owner");

        Robot storage robot = _robots[robotId];
        require(robot.id != 0, "RobotRegistry: robot not found");
        require(robot.owner == msg.sender, "RobotRegistry: not owner");
        require(robot.status != RobotStatus.Retired, "RobotRegistry: robot retired");

        address oldOwner = robot.owner;

        // Remove from old owner's list
        _removeFromOwnerList(oldOwner, robotId);

        // Add to new owner's list
        _ownerRobots[newOwner].push(robotId);
        _ownerRobotIndex[newOwner][robotId] = _ownerRobots[newOwner].length - 1;

        // Update owner
        robot.owner = newOwner;

        emit RobotTransferred(robotId, oldOwner, newOwner);
    }

    /**
     * @inheritdoc IRobotRegistry
     */
    function setRobotStatus(uint256 robotId, RobotStatus newStatus) external {
        Robot storage robot = _robots[robotId];
        require(robot.id != 0, "RobotRegistry: robot not found");
        require(
            robot.owner == msg.sender || hasRole(ADMIN_ROLE, msg.sender),
            "RobotRegistry: not authorized"
        );

        // Can't un-retire a robot
        require(
            robot.status != RobotStatus.Retired || hasRole(ADMIN_ROLE, msg.sender),
            "RobotRegistry: cannot change retired status"
        );

        RobotStatus oldStatus = robot.status;
        robot.status = newStatus;

        emit RobotStatusChanged(robotId, oldStatus, newStatus);
    }

    /**
     * @inheritdoc IRobotRegistry
     */
    function submitAttestation(
        uint256 robotId,
        bytes32 firmwareHash,
        bytes calldata signature
    ) external whenNotPaused {
        Robot storage robot = _robots[robotId];
        require(robot.id != 0, "RobotRegistry: robot not found");
        require(robot.owner == msg.sender, "RobotRegistry: not owner");
        require(robot.status == RobotStatus.Active, "RobotRegistry: robot not active");
        require(_approvedFirmware[firmwareHash], "RobotRegistry: firmware not approved");

        // Verify attestation
        require(
            _verifyAttestation(robot.hardwareFingerprint, firmwareHash, signature),
            "RobotRegistry: invalid attestation"
        );

        // Update robot
        robot.firmwareHash = firmwareHash;
        robot.lastAttestationAt = block.timestamp;

        // Store attestation
        _attestationHistory[robotId].push(Attestation({
            firmwareHash: firmwareHash,
            timestamp: block.timestamp,
            signature: signature
        }));

        emit AttestationSubmitted(robotId, firmwareHash, block.timestamp);
    }

    /**
     * @inheritdoc IRobotRegistry
     */
    function updateReputation(uint256 robotId, int256 delta) external onlyRole(REPUTATION_MANAGER_ROLE) {
        Robot storage robot = _robots[robotId];
        require(robot.id != 0, "RobotRegistry: robot not found");

        uint256 oldScore = robot.reputationScore;

        if (delta >= 0) {
            robot.reputationScore = _min(robot.reputationScore + uint256(delta), MAX_REPUTATION);
        } else {
            uint256 decrease = uint256(-delta);
            robot.reputationScore = decrease >= robot.reputationScore ? 0 : robot.reputationScore - decrease;
        }

        emit ReputationUpdated(robotId, oldScore, robot.reputationScore);
    }

    // ============ Admin Functions ============

    /**
     * @notice Approve a firmware hash
     * @param firmwareHash The firmware hash to approve
     */
    function approveFirmware(bytes32 firmwareHash) external onlyRole(FIRMWARE_MANAGER_ROLE) {
        _approvedFirmware[firmwareHash] = true;
    }

    /**
     * @notice Revoke a firmware hash
     * @param firmwareHash The firmware hash to revoke
     */
    function revokeFirmware(bytes32 firmwareHash) external onlyRole(FIRMWARE_MANAGER_ROLE) {
        _approvedFirmware[firmwareHash] = false;
    }

    /**
     * @notice Register a TEE public key for a manufacturer
     * @param manufacturerId Manufacturer identifier
     * @param pubKey The public key bytes
     */
    function registerTeePubKey(
        bytes32 manufacturerId,
        bytes calldata pubKey
    ) external onlyRole(ADMIN_ROLE) {
        _teePubKeys[manufacturerId] = pubKey;
    }

    /**
     * @notice Pause the contract
     */
    function pause() external onlyRole(ADMIN_ROLE) {
        _pause();
    }

    /**
     * @notice Unpause the contract
     */
    function unpause() external onlyRole(ADMIN_ROLE) {
        _unpause();
    }

    // ============ View Functions ============

    /**
     * @inheritdoc IRobotRegistry
     */
    function isApprovedFirmware(bytes32 firmwareHash) external view returns (bool) {
        return _approvedFirmware[firmwareHash];
    }

    /**
     * @inheritdoc IRobotRegistry
     */
    function getRobot(uint256 robotId) external view returns (Robot memory) {
        require(_robots[robotId].id != 0, "RobotRegistry: robot not found");
        return _robots[robotId];
    }

    /**
     * @inheritdoc IRobotRegistry
     */
    function getRobotByFingerprint(bytes32 hardwareFingerprint) external view returns (uint256) {
        return _fingerprintToRobotId[hardwareFingerprint];
    }

    /**
     * @inheritdoc IRobotRegistry
     */
    function isOwner(address owner, uint256 robotId) external view returns (bool) {
        return _robots[robotId].owner == owner;
    }

    /**
     * @inheritdoc IRobotRegistry
     */
    function getRobotsByOwner(address owner) external view returns (uint256[] memory) {
        return _ownerRobots[owner];
    }

    /**
     * @inheritdoc IRobotRegistry
     */
    function isAttestationValid(uint256 robotId) external view returns (bool) {
        Robot storage robot = _robots[robotId];
        if (robot.id == 0) return false;

        return block.timestamp <= robot.lastAttestationAt + ATTESTATION_VALIDITY_PERIOD;
    }

    /**
     * @notice Get attestation history for a robot
     * @param robotId The robot ID
     * @return Array of attestations
     */
    function getAttestationHistory(uint256 robotId) external view returns (Attestation[] memory) {
        return _attestationHistory[robotId];
    }

    /**
     * @notice Get total registered robots count
     * @return Total count
     */
    function totalRobots() external view returns (uint256) {
        return _nextRobotId - 1;
    }

    // ============ Internal Functions ============

    /**
     * @notice Remove a robot from owner's list
     */
    function _removeFromOwnerList(address owner, uint256 robotId) internal {
        uint256[] storage robots = _ownerRobots[owner];
        uint256 index = _ownerRobotIndex[owner][robotId];
        uint256 lastIndex = robots.length - 1;

        if (index != lastIndex) {
            uint256 lastRobotId = robots[lastIndex];
            robots[index] = lastRobotId;
            _ownerRobotIndex[owner][lastRobotId] = index;
        }

        robots.pop();
        delete _ownerRobotIndex[owner][robotId];
    }

    /**
     * @notice Verify TEE attestation signature
     * @dev In production, this would verify against registered TEE public keys
     */
    function _verifyAttestation(
        bytes32 hardwareFingerprint,
        bytes32 firmwareHash,
        bytes calldata signature
    ) internal view returns (bool) {
        // TODO: Implement actual TEE signature verification
        // This would typically:
        // 1. Extract manufacturer ID from fingerprint
        // 2. Get registered public key for manufacturer
        // 3. Verify signature over (fingerprint || firmwareHash || timestamp)

        // For now, accept non-empty signatures (placeholder)
        return signature.length > 0;
    }

    /**
     * @notice Return minimum of two values
     */
    function _min(uint256 a, uint256 b) internal pure returns (uint256) {
        return a < b ? a : b;
    }
}
