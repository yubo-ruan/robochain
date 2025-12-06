// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title IPermissionManager
 * @notice Interface for robot permission management
 * @dev Controls what actions robots can perform and who can command them
 */
interface IPermissionManager {
    // ============ Enums ============

    enum PermissionType {
        Denied,         // Explicitly denied
        Allowed,        // Explicitly allowed
        Inherited       // Inherit from parent/default
    }

    enum ActionCategory {
        Movement,       // Navigation, locomotion
        Manipulation,   // Picking, placing, manipulating objects
        Communication,  // Speaking, messaging
        Recording,      // Camera, microphone access
        Purchasing,     // Making purchases, ordering
        Emergency,      // Emergency actions
        Maintenance     // Self-maintenance, updates
    }

    // ============ Structs ============

    struct Permission {
        PermissionType permType;
        uint256 validFrom;          // 0 = immediate
        uint256 validUntil;         // 0 = forever
        uint256[] allowedHours;     // Bitmask for each day (24 bits per day, 7 days)
        bytes32 geofenceRoot;       // Merkle root of allowed zones (0 = no restriction)
    }

    struct UserPermissions {
        address user;
        uint256 robotId;
        bool isAdmin;               // Can modify other users' permissions
        mapping(bytes32 => Permission) actionPermissions;  // action hash => permission
    }

    struct GuestToken {
        uint256 robotId;
        address guest;
        uint256 validFrom;
        uint256 validUntil;
        bytes32[] allowedActions;   // Specific actions allowed
        bool used;
    }

    struct EmergencyContact {
        address contact;
        bool canStop;               // Can trigger emergency stop
        bool canResume;             // Can resume after stop
        uint256 priority;           // Lower = higher priority
    }

    // ============ Events ============

    event PermissionGranted(
        uint256 indexed robotId,
        address indexed user,
        bytes32 indexed actionHash,
        PermissionType permType
    );

    event PermissionRevoked(
        uint256 indexed robotId,
        address indexed user,
        bytes32 indexed actionHash
    );

    event AdminSet(
        uint256 indexed robotId,
        address indexed user,
        bool isAdmin
    );

    event GuestTokenCreated(
        uint256 indexed tokenId,
        uint256 indexed robotId,
        address indexed guest,
        uint256 validUntil
    );

    event GuestTokenUsed(
        uint256 indexed tokenId,
        address indexed guest
    );

    event EmergencyStopTriggered(
        uint256 indexed robotId,
        address indexed triggeredBy
    );

    event EmergencyStopLifted(
        uint256 indexed robotId,
        address indexed liftedBy
    );

    event PermissionRootUpdated(
        uint256 indexed robotId,
        bytes32 oldRoot,
        bytes32 newRoot
    );

    // ============ Core Permission Functions ============

    /**
     * @notice Set permission for a user to perform an action on a robot
     * @param robotId The robot ID
     * @param user The user address
     * @param actionHash Hash of the action identifier
     * @param permission The permission details
     */
    function setPermission(
        uint256 robotId,
        address user,
        bytes32 actionHash,
        Permission calldata permission
    ) external;

    /**
     * @notice Batch set multiple permissions
     * @param robotId The robot ID
     * @param users Array of user addresses
     * @param actionHashes Array of action hashes
     * @param permissions Array of permissions
     */
    function batchSetPermissions(
        uint256 robotId,
        address[] calldata users,
        bytes32[] calldata actionHashes,
        Permission[] calldata permissions
    ) external;

    /**
     * @notice Revoke a specific permission
     * @param robotId The robot ID
     * @param user The user address
     * @param actionHash The action hash
     */
    function revokePermission(
        uint256 robotId,
        address user,
        bytes32 actionHash
    ) external;

    /**
     * @notice Set admin status for a user
     * @param robotId The robot ID
     * @param user The user address
     * @param isAdmin Whether user is admin
     */
    function setAdmin(uint256 robotId, address user, bool isAdmin) external;

    // ============ Guest Access Functions ============

    /**
     * @notice Create a temporary guest access token
     * @param robotId The robot ID
     * @param guest The guest address
     * @param validFrom Start time
     * @param validUntil End time
     * @param allowedActions Actions the guest can perform
     * @return tokenId The guest token ID
     */
    function createGuestToken(
        uint256 robotId,
        address guest,
        uint256 validFrom,
        uint256 validUntil,
        bytes32[] calldata allowedActions
    ) external returns (uint256 tokenId);

    /**
     * @notice Revoke a guest token
     * @param tokenId The token to revoke
     */
    function revokeGuestToken(uint256 tokenId) external;

    // ============ Emergency Functions ============

    /**
     * @notice Add an emergency contact
     * @param robotId The robot ID
     * @param contact The contact address
     * @param canStop Whether contact can trigger stop
     * @param canResume Whether contact can resume
     * @param priority Contact priority
     */
    function addEmergencyContact(
        uint256 robotId,
        address contact,
        bool canStop,
        bool canResume,
        uint256 priority
    ) external;

    /**
     * @notice Remove an emergency contact
     * @param robotId The robot ID
     * @param contact The contact address
     */
    function removeEmergencyContact(uint256 robotId, address contact) external;

    /**
     * @notice Trigger emergency stop
     * @param robotId The robot ID
     */
    function triggerEmergencyStop(uint256 robotId) external;

    /**
     * @notice Lift emergency stop
     * @param robotId The robot ID
     */
    function liftEmergencyStop(uint256 robotId) external;

    // ============ View Functions ============

    /**
     * @notice Check if a user can perform an action
     * @param robotId The robot ID
     * @param user The user address
     * @param actionHash The action hash
     * @return Whether the action is allowed
     */
    function canPerformAction(
        uint256 robotId,
        address user,
        bytes32 actionHash
    ) external view returns (bool);

    /**
     * @notice Check if action is allowed at a specific time
     * @param robotId The robot ID
     * @param user The user address
     * @param actionHash The action hash
     * @param timestamp The time to check
     * @return Whether allowed at that time
     */
    function canPerformActionAt(
        uint256 robotId,
        address user,
        bytes32 actionHash,
        uint256 timestamp
    ) external view returns (bool);

    /**
     * @notice Get the permission merkle root for a robot (for offline verification)
     * @param robotId The robot ID
     * @return The merkle root
     */
    function getPermissionRoot(uint256 robotId) external view returns (bytes32);

    /**
     * @notice Check if robot is in emergency stop
     * @param robotId The robot ID
     * @return Whether robot is stopped
     */
    function isEmergencyStopped(uint256 robotId) external view returns (bool);

    /**
     * @notice Get guest token details
     * @param tokenId The token ID
     * @return The guest token struct
     */
    function getGuestToken(uint256 tokenId) external view returns (
        uint256 robotId,
        address guest,
        uint256 validFrom,
        uint256 validUntil,
        bytes32[] memory allowedActions,
        bool used
    );

    /**
     * @notice Verify a permission proof (for offline verification)
     * @param robotId The robot ID
     * @param user The user address
     * @param actionHash The action hash
     * @param proof The merkle proof
     * @return Whether the proof is valid
     */
    function verifyPermissionProof(
        uint256 robotId,
        address user,
        bytes32 actionHash,
        bytes32[] calldata proof
    ) external view returns (bool);
}
