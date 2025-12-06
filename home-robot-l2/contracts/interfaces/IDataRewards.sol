// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title IDataRewards
 * @notice Interface for UMI-style data collection incentives
 * @dev Manages demo submissions, validation, and reward distribution
 */
interface IDataRewards {
    // ============ Enums ============

    enum DemoStatus {
        Pending,        // Awaiting validation
        Validated,      // Passed validation
        Rejected,       // Failed validation
        Disputed,       // Under dispute
        Rewarded        // Rewards claimed
    }

    enum TaskType {
        Manipulation,       // Pick, place, pour, etc.
        Navigation,         // Movement, path planning
        HumanInteraction,   // Following commands, gestures
        Cleaning,           // Cleaning tasks
        Cooking,            // Food preparation
        Organization,       // Tidying, sorting
        Maintenance,        // Self-maintenance demos
        EdgeCase,           // Failure recovery, edge cases
        Other               // Miscellaneous
    }

    // ============ Structs ============

    struct Demo {
        uint256 id;
        uint256 robotId;
        address contributor;
        bytes32 contentHash;        // IPFS/Arweave hash
        bytes32 metadataHash;       // Hash of metadata JSON
        TaskType taskType;
        uint256 duration;           // Demo length in seconds
        uint256 submittedAt;
        DemoStatus status;
        uint256 qualityScore;       // 0-100, set by validators
        uint256 rewardAmount;       // Calculated reward
        bool rewardClaimed;
    }

    struct Contributor {
        address addr;
        uint256 totalDemos;
        uint256 validatedDemos;
        uint256 rejectedDemos;
        uint256 totalRewards;
        uint256 reputationScore;    // Affects reward multiplier
        uint256 stakeLocked;        // Tokens staked for quality guarantee
    }

    struct Validator {
        address addr;
        uint256 stakeAmount;
        uint256 validationsCount;
        uint256 accuracyScore;      // Track record of correct validations
        bool isActive;
        uint256 slashedAmount;      // Total slashed
    }

    struct ValidationVote {
        address validator;
        bool approved;
        uint256 qualityScore;
        uint256 timestamp;
        bytes32 proofHash;          // Hash of detailed validation report
    }

    struct RewardMultiplier {
        TaskType taskType;
        uint256 baseMultiplier;     // 100 = 1x, 200 = 2x
        uint256 noveltyBonus;       // For novel/rare tasks
        uint256 qualityBonus;       // For high-quality demos
        uint256 edgeCaseBonus;      // For failure recovery demos
    }

    // ============ Events ============

    event DemoSubmitted(
        uint256 indexed demoId,
        uint256 indexed robotId,
        address indexed contributor,
        bytes32 contentHash,
        TaskType taskType
    );

    event DemoValidated(
        uint256 indexed demoId,
        bool approved,
        uint256 qualityScore,
        uint256 validatorCount
    );

    event RewardClaimed(
        uint256 indexed demoId,
        address indexed contributor,
        uint256 amount
    );

    event ValidatorRegistered(
        address indexed validator,
        uint256 stakeAmount
    );

    event ValidatorSlashed(
        address indexed validator,
        uint256 amount,
        string reason
    );

    event ReputationUpdated(
        address indexed contributor,
        uint256 oldScore,
        uint256 newScore
    );

    event MultiplierUpdated(
        TaskType indexed taskType,
        uint256 baseMultiplier
    );

    event BonusDistributed(
        uint256 indexed demoId,
        address indexed contributor,
        uint256 bonusAmount,
        string reason
    );

    // ============ Demo Submission Functions ============

    /**
     * @notice Submit a new demonstration
     * @param robotId The robot that recorded the demo
     * @param contentHash IPFS/Arweave hash of demo data
     * @param metadataHash Hash of metadata JSON
     * @param taskType Type of task demonstrated
     * @param duration Demo length in seconds
     * @param teeSignature TEE signature proving authenticity
     * @return demoId The ID of the submitted demo
     */
    function submitDemo(
        uint256 robotId,
        bytes32 contentHash,
        bytes32 metadataHash,
        TaskType taskType,
        uint256 duration,
        bytes calldata teeSignature
    ) external returns (uint256 demoId);

    /**
     * @notice Batch submit multiple demos
     * @param robotIds Robot IDs
     * @param contentHashes Content hashes
     * @param metadataHashes Metadata hashes
     * @param taskTypes Task types
     * @param durations Durations
     * @param teeSignatures TEE signatures
     * @return demoIds Array of demo IDs
     */
    function batchSubmitDemos(
        uint256[] calldata robotIds,
        bytes32[] calldata contentHashes,
        bytes32[] calldata metadataHashes,
        TaskType[] calldata taskTypes,
        uint256[] calldata durations,
        bytes[] calldata teeSignatures
    ) external returns (uint256[] memory demoIds);

    // ============ Validation Functions ============

    /**
     * @notice Register as a validator
     * @param stakeAmount Amount to stake
     */
    function registerValidator(uint256 stakeAmount) external;

    /**
     * @notice Submit a validation vote
     * @param demoId The demo to validate
     * @param approved Whether demo is approved
     * @param qualityScore Quality score (0-100)
     * @param proofHash Hash of detailed validation report
     */
    function submitValidation(
        uint256 demoId,
        bool approved,
        uint256 qualityScore,
        bytes32 proofHash
    ) external;

    /**
     * @notice Finalize validation after voting period
     * @param demoId The demo to finalize
     */
    function finalizeValidation(uint256 demoId) external;

    /**
     * @notice Dispute a validation result
     * @param demoId The demo to dispute
     * @param evidence Evidence hash
     */
    function disputeValidation(uint256 demoId, bytes32 evidence) external;

    /**
     * @notice Slash a validator for misconduct
     * @param validator The validator address
     * @param amount Amount to slash
     * @param reason Reason for slashing
     */
    function slashValidator(
        address validator,
        uint256 amount,
        string calldata reason
    ) external;

    // ============ Reward Functions ============

    /**
     * @notice Claim reward for a validated demo
     * @param demoId The demo ID
     */
    function claimReward(uint256 demoId) external;

    /**
     * @notice Batch claim rewards for multiple demos
     * @param demoIds Array of demo IDs
     */
    function batchClaimRewards(uint256[] calldata demoIds) external;

    /**
     * @notice Calculate reward for a demo (before claiming)
     * @param demoId The demo ID
     * @return The reward amount
     */
    function calculateReward(uint256 demoId) external view returns (uint256);

    /**
     * @notice Distribute retrospective bonus (when data improves models)
     * @param demoIds Demos that contributed to improvement
     * @param bonusAmounts Bonus amounts for each
     * @param reason Reason for bonus
     */
    function distributeRetrospectiveBonus(
        uint256[] calldata demoIds,
        uint256[] calldata bonusAmounts,
        string calldata reason
    ) external;

    // ============ Staking Functions ============

    /**
     * @notice Stake tokens as a contributor (increases rewards)
     * @param amount Amount to stake
     */
    function stakeAsContributor(uint256 amount) external;

    /**
     * @notice Unstake tokens (with delay)
     * @param amount Amount to unstake
     */
    function unstake(uint256 amount) external;

    /**
     * @notice Add stake to validator
     * @param amount Amount to add
     */
    function addValidatorStake(uint256 amount) external;

    // ============ View Functions ============

    /**
     * @notice Get demo details
     * @param demoId The demo ID
     * @return The demo struct
     */
    function getDemo(uint256 demoId) external view returns (Demo memory);

    /**
     * @notice Get contributor details
     * @param contributor The contributor address
     * @return The contributor struct
     */
    function getContributor(address contributor) external view returns (Contributor memory);

    /**
     * @notice Get validator details
     * @param validator The validator address
     * @return The validator struct
     */
    function getValidator(address validator) external view returns (Validator memory);

    /**
     * @notice Get current reward multipliers
     * @param taskType The task type
     * @return The multiplier struct
     */
    function getRewardMultiplier(TaskType taskType) external view returns (RewardMultiplier memory);

    /**
     * @notice Get pending demos for validation
     * @param limit Maximum number to return
     * @return Array of demo IDs
     */
    function getPendingDemos(uint256 limit) external view returns (uint256[] memory);

    /**
     * @notice Get validation votes for a demo
     * @param demoId The demo ID
     * @return Array of validation votes
     */
    function getValidationVotes(uint256 demoId) external view returns (ValidationVote[] memory);

    /**
     * @notice Check if a content hash has already been submitted
     * @param contentHash The hash to check
     * @return Whether it exists
     */
    function isDuplicateContent(bytes32 contentHash) external view returns (bool);

    /**
     * @notice Get total rewards distributed
     * @return Total amount
     */
    function getTotalRewardsDistributed() external view returns (uint256);

    /**
     * @notice Get total demos submitted
     * @return Total count
     */
    function getTotalDemos() external view returns (uint256);
}
