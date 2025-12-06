// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/cryptography/MerkleProof.sol";
import "../interfaces/IPermissionManager.sol";
import "../interfaces/IRobotRegistry.sol";

/**
 * @title PermissionManager
 * @notice Controls what actions robots can perform and who can command them
 * @dev Supports offline verification via merkle proofs
 */
contract PermissionManager is IPermissionManager, AccessControl, Pausable, ReentrancyGuard {
    // ============ Constants ============

    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");

    // ============ State Variables ============

    IRobotRegistry public immutable robotRegistry;

    // Guest token counter
    uint256 private _nextGuestTokenId = 1;

    // Robot ID => User => Action Hash => Permission
    mapping(uint256 => mapping(address => mapping(bytes32 => Permission))) private _permissions;

    // Robot ID => User => Is Admin
    mapping(uint256 => mapping(address => bool)) private _robotAdmins;

    // Robot ID => Emergency stop status
    mapping(uint256 => bool) private _emergencyStopped;

    // Robot ID => Emergency contacts
    mapping(uint256 => EmergencyContact[]) private _emergencyContacts;

    // Robot ID => Address => Is emergency contact
    mapping(uint256 => mapping(address => bool)) private _isEmergencyContact;

    // Guest token ID => Guest token data
    mapping(uint256 => GuestToken) private _guestTokens;

    // Robot ID => Permission merkle root (for offline verification)
    mapping(uint256 => bytes32) private _permissionRoots;

    // Robot ID => Default permission for unset actions
    mapping(uint256 => PermissionType) private _defaultPermission;

    // ============ Constructor ============

    constructor(address admin, address _robotRegistry) {
        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _grantRole(ADMIN_ROLE, admin);
        robotRegistry = IRobotRegistry(_robotRegistry);
    }

    // ============ Modifiers ============

    modifier onlyRobotOwnerOrAdmin(uint256 robotId) {
        require(
            robotRegistry.isOwner(msg.sender, robotId) || _robotAdmins[robotId][msg.sender],
            "PermissionManager: not owner or admin"
        );
        _;
    }

    modifier onlyRobotOwner(uint256 robotId) {
        require(
            robotRegistry.isOwner(msg.sender, robotId),
            "PermissionManager: not owner"
        );
        _;
    }

    // ============ Permission Functions ============

    /**
     * @inheritdoc IPermissionManager
     */
    function setPermission(
        uint256 robotId,
        address user,
        bytes32 actionHash,
        Permission calldata permission
    ) external whenNotPaused onlyRobotOwnerOrAdmin(robotId) {
        _permissions[robotId][user][actionHash] = permission;
        _updatePermissionRoot(robotId);

        emit PermissionGranted(robotId, user, actionHash, permission.permType);
    }

    /**
     * @inheritdoc IPermissionManager
     */
    function batchSetPermissions(
        uint256 robotId,
        address[] calldata users,
        bytes32[] calldata actionHashes,
        Permission[] calldata permissions
    ) external whenNotPaused onlyRobotOwnerOrAdmin(robotId) {
        require(
            users.length == actionHashes.length && actionHashes.length == permissions.length,
            "PermissionManager: array length mismatch"
        );

        for (uint256 i = 0; i < users.length; i++) {
            _permissions[robotId][users[i]][actionHashes[i]] = permissions[i];
            emit PermissionGranted(robotId, users[i], actionHashes[i], permissions[i].permType);
        }

        _updatePermissionRoot(robotId);
    }

    /**
     * @inheritdoc IPermissionManager
     */
    function revokePermission(
        uint256 robotId,
        address user,
        bytes32 actionHash
    ) external whenNotPaused onlyRobotOwnerOrAdmin(robotId) {
        delete _permissions[robotId][user][actionHash];
        _updatePermissionRoot(robotId);

        emit PermissionRevoked(robotId, user, actionHash);
    }

    /**
     * @inheritdoc IPermissionManager
     */
    function setAdmin(uint256 robotId, address user, bool isAdmin) external onlyRobotOwner(robotId) {
        _robotAdmins[robotId][user] = isAdmin;
        emit AdminSet(robotId, user, isAdmin);
    }

    /**
     * @notice Set default permission for a robot
     * @param robotId The robot ID
     * @param defaultPerm Default permission type
     */
    function setDefaultPermission(
        uint256 robotId,
        PermissionType defaultPerm
    ) external onlyRobotOwner(robotId) {
        _defaultPermission[robotId] = defaultPerm;
    }

    // ============ Guest Access Functions ============

    /**
     * @inheritdoc IPermissionManager
     */
    function createGuestToken(
        uint256 robotId,
        address guest,
        uint256 validFrom,
        uint256 validUntil,
        bytes32[] calldata allowedActions
    ) external whenNotPaused onlyRobotOwnerOrAdmin(robotId) returns (uint256 tokenId) {
        require(validUntil > validFrom, "PermissionManager: invalid time range");
        require(validUntil > block.timestamp, "PermissionManager: token already expired");
        require(allowedActions.length > 0, "PermissionManager: no actions specified");

        tokenId = _nextGuestTokenId++;

        _guestTokens[tokenId] = GuestToken({
            robotId: robotId,
            guest: guest,
            validFrom: validFrom,
            validUntil: validUntil,
            allowedActions: allowedActions,
            used: false
        });

        emit GuestTokenCreated(tokenId, robotId, guest, validUntil);
    }

    /**
     * @inheritdoc IPermissionManager
     */
    function revokeGuestToken(uint256 tokenId) external whenNotPaused {
        GuestToken storage token = _guestTokens[tokenId];
        require(token.robotId != 0, "PermissionManager: token not found");
        require(
            robotRegistry.isOwner(msg.sender, token.robotId) ||
            _robotAdmins[token.robotId][msg.sender],
            "PermissionManager: not authorized"
        );

        delete _guestTokens[tokenId];
    }

    /**
     * @notice Use a guest token (marks it as used)
     * @param tokenId The token to use
     */
    function useGuestToken(uint256 tokenId) external {
        GuestToken storage token = _guestTokens[tokenId];
        require(token.robotId != 0, "PermissionManager: token not found");
        require(token.guest == msg.sender, "PermissionManager: not token holder");
        require(!token.used, "PermissionManager: token already used");
        require(block.timestamp >= token.validFrom, "PermissionManager: token not yet valid");
        require(block.timestamp <= token.validUntil, "PermissionManager: token expired");

        token.used = true;
        emit GuestTokenUsed(tokenId, msg.sender);
    }

    // ============ Emergency Functions ============

    /**
     * @inheritdoc IPermissionManager
     */
    function addEmergencyContact(
        uint256 robotId,
        address contact,
        bool canStop,
        bool canResume,
        uint256 priority
    ) external onlyRobotOwner(robotId) {
        require(!_isEmergencyContact[robotId][contact], "PermissionManager: already emergency contact");

        _emergencyContacts[robotId].push(EmergencyContact({
            contact: contact,
            canStop: canStop,
            canResume: canResume,
            priority: priority
        }));

        _isEmergencyContact[robotId][contact] = true;
    }

    /**
     * @inheritdoc IPermissionManager
     */
    function removeEmergencyContact(uint256 robotId, address contact) external onlyRobotOwner(robotId) {
        require(_isEmergencyContact[robotId][contact], "PermissionManager: not emergency contact");

        EmergencyContact[] storage contacts = _emergencyContacts[robotId];
        for (uint256 i = 0; i < contacts.length; i++) {
            if (contacts[i].contact == contact) {
                contacts[i] = contacts[contacts.length - 1];
                contacts.pop();
                break;
            }
        }

        _isEmergencyContact[robotId][contact] = false;
    }

    /**
     * @inheritdoc IPermissionManager
     */
    function triggerEmergencyStop(uint256 robotId) external {
        require(
            robotRegistry.isOwner(msg.sender, robotId) ||
            _canTriggerEmergencyStop(robotId, msg.sender),
            "PermissionManager: not authorized for emergency stop"
        );

        _emergencyStopped[robotId] = true;
        emit EmergencyStopTriggered(robotId, msg.sender);
    }

    /**
     * @inheritdoc IPermissionManager
     */
    function liftEmergencyStop(uint256 robotId) external {
        require(
            robotRegistry.isOwner(msg.sender, robotId) ||
            _canLiftEmergencyStop(robotId, msg.sender),
            "PermissionManager: not authorized to lift emergency stop"
        );

        _emergencyStopped[robotId] = false;
        emit EmergencyStopLifted(robotId, msg.sender);
    }

    // ============ View Functions ============

    /**
     * @inheritdoc IPermissionManager
     */
    function canPerformAction(
        uint256 robotId,
        address user,
        bytes32 actionHash
    ) external view returns (bool) {
        return _canPerformAction(robotId, user, actionHash, block.timestamp);
    }

    /**
     * @inheritdoc IPermissionManager
     */
    function canPerformActionAt(
        uint256 robotId,
        address user,
        bytes32 actionHash,
        uint256 timestamp
    ) external view returns (bool) {
        return _canPerformAction(robotId, user, actionHash, timestamp);
    }

    /**
     * @inheritdoc IPermissionManager
     */
    function getPermissionRoot(uint256 robotId) external view returns (bytes32) {
        return _permissionRoots[robotId];
    }

    /**
     * @inheritdoc IPermissionManager
     */
    function isEmergencyStopped(uint256 robotId) external view returns (bool) {
        return _emergencyStopped[robotId];
    }

    /**
     * @inheritdoc IPermissionManager
     */
    function getGuestToken(uint256 tokenId) external view returns (
        uint256 robotId,
        address guest,
        uint256 validFrom,
        uint256 validUntil,
        bytes32[] memory allowedActions,
        bool used
    ) {
        GuestToken storage token = _guestTokens[tokenId];
        return (
            token.robotId,
            token.guest,
            token.validFrom,
            token.validUntil,
            token.allowedActions,
            token.used
        );
    }

    /**
     * @inheritdoc IPermissionManager
     */
    function verifyPermissionProof(
        uint256 robotId,
        address user,
        bytes32 actionHash,
        bytes32[] calldata proof
    ) external view returns (bool) {
        bytes32 root = _permissionRoots[robotId];
        if (root == bytes32(0)) return false;

        bytes32 leaf = keccak256(abi.encodePacked(robotId, user, actionHash));
        return MerkleProof.verify(proof, root, leaf);
    }

    /**
     * @notice Get permission details
     * @param robotId The robot ID
     * @param user The user address
     * @param actionHash The action hash
     * @return The permission struct
     */
    function getPermission(
        uint256 robotId,
        address user,
        bytes32 actionHash
    ) external view returns (Permission memory) {
        return _permissions[robotId][user][actionHash];
    }

    /**
     * @notice Check if user is admin for robot
     * @param robotId The robot ID
     * @param user The user address
     * @return Whether user is admin
     */
    function isRobotAdmin(uint256 robotId, address user) external view returns (bool) {
        return _robotAdmins[robotId][user];
    }

    /**
     * @notice Get emergency contacts for a robot
     * @param robotId The robot ID
     * @return Array of emergency contacts
     */
    function getEmergencyContacts(uint256 robotId) external view returns (EmergencyContact[] memory) {
        return _emergencyContacts[robotId];
    }

    // ============ Admin Functions ============

    function pause() external onlyRole(ADMIN_ROLE) {
        _pause();
    }

    function unpause() external onlyRole(ADMIN_ROLE) {
        _unpause();
    }

    // ============ Internal Functions ============

    function _canPerformAction(
        uint256 robotId,
        address user,
        bytes32 actionHash,
        uint256 timestamp
    ) internal view returns (bool) {
        // Emergency stop overrides everything
        if (_emergencyStopped[robotId]) {
            return false;
        }

        // Owner can always perform actions
        if (robotRegistry.isOwner(user, robotId)) {
            return true;
        }

        // Check explicit permission
        Permission storage perm = _permissions[robotId][user][actionHash];

        // Check if permission is explicitly denied
        if (perm.permType == PermissionType.Denied) {
            return false;
        }

        // Check if permission is explicitly allowed
        if (perm.permType == PermissionType.Allowed) {
            // Check time validity
            if (perm.validFrom > 0 && timestamp < perm.validFrom) {
                return false;
            }
            if (perm.validUntil > 0 && timestamp > perm.validUntil) {
                return false;
            }

            // Check hourly restrictions if set
            if (perm.allowedHours.length > 0) {
                if (!_isAllowedAtTime(perm.allowedHours, timestamp)) {
                    return false;
                }
            }

            return true;
        }

        // Check guest tokens
        // Note: In production, would need to iterate through active tokens for this user
        // This is simplified for gas efficiency

        // Fall back to default permission
        return _defaultPermission[robotId] == PermissionType.Allowed;
    }

    function _isAllowedAtTime(uint256[] storage allowedHours, uint256 timestamp) internal view returns (bool) {
        // allowedHours is a bitmask for each day
        // Each uint256 has 24 bits for hours, 7 days = 168 bits needed

        uint256 dayOfWeek = (timestamp / 1 days + 4) % 7; // 0 = Monday
        uint256 hourOfDay = (timestamp / 1 hours) % 24;

        if (dayOfWeek >= allowedHours.length) {
            return true; // No restriction for this day
        }

        uint256 dayMask = allowedHours[dayOfWeek];
        return (dayMask & (1 << hourOfDay)) != 0;
    }

    function _canTriggerEmergencyStop(uint256 robotId, address caller) internal view returns (bool) {
        EmergencyContact[] storage contacts = _emergencyContacts[robotId];
        for (uint256 i = 0; i < contacts.length; i++) {
            if (contacts[i].contact == caller && contacts[i].canStop) {
                return true;
            }
        }
        return false;
    }

    function _canLiftEmergencyStop(uint256 robotId, address caller) internal view returns (bool) {
        EmergencyContact[] storage contacts = _emergencyContacts[robotId];
        for (uint256 i = 0; i < contacts.length; i++) {
            if (contacts[i].contact == caller && contacts[i].canResume) {
                return true;
            }
        }
        return false;
    }

    function _updatePermissionRoot(uint256 robotId) internal {
        // TODO: In production, compute actual merkle root of all permissions
        // For now, just update a hash to signal changes
        bytes32 oldRoot = _permissionRoots[robotId];
        bytes32 newRoot = keccak256(abi.encodePacked(robotId, block.timestamp, block.number));
        _permissionRoots[robotId] = newRoot;

        emit PermissionRootUpdated(robotId, oldRoot, newRoot);
    }
}
