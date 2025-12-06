// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "../interfaces/IDataRewards.sol";
import "../interfaces/IRobotRegistry.sol";
import "../interfaces/IHomeRobotToken.sol";

/**
 * @title DataRewards
 * @notice UMI-style data collection incentives for home robots
 * @dev Manages demo submissions, validation, and reward distribution
 */
contract DataRewards is IDataRewards, AccessControl, Pausable, ReentrancyGuard {
    // ============ Constants ============

    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    bytes32 public constant VALIDATOR_MANAGER_ROLE = keccak256("VALIDATOR_MANAGER_ROLE");
    bytes32 public constant BONUS_DISTRIBUTOR_ROLE = keccak256("BONUS_DISTRIBUTOR_ROLE");

    uint256 public constant MIN_VALIDATOR_STAKE = 10_000 * 1e18; // 10,000 tokens
    uint256 public constant VALIDATION_PERIOD = 3 days;
    uint256 public constant MIN_VALIDATORS_FOR_CONSENSUS = 3;
    uint256 public constant QUALITY_SCORE_PRECISION = 100;
    uint256 public constant SLASH_PERCENTAGE = 10; // 10% slash for bad validations

    uint256 public constant BASE_REWARD = 10 * 1e18; // 10 tokens base reward

    // ============ State Variables ============

    IRobotRegistry public immutable robotRegistry;
    IHomeRobotToken public immutable token;

    // Demo counter
    uint256 private _nextDemoId = 1;

    // Demo ID => Demo data
    mapping(uint256 => Demo) private _demos;

    // Content hash => Demo ID (to prevent duplicates)
    mapping(bytes32 => uint256) private _contentHashToDemo;

    // Contributor address => Contributor data
    mapping(address => Contributor) private _contributors;

    // Validator address => Validator data
    mapping(address => Validator) private _validators;

    // Demo ID => Validation votes
    mapping(uint256 => ValidationVote[]) private _validationVotes;

    // Demo ID => Validator => Has voted
    mapping(uint256 => mapping(address => bool)) private _hasVoted;

    // Task type => Reward multiplier
    mapping(TaskType => RewardMultiplier) private _rewardMultipliers;

    // Total stats
    uint256 private _totalRewardsDistributed;
    uint256 private _totalDemos;
    uint256 private _totalValidatedDemos;

    // Pending demos list (simplified - in production use more efficient data structure)
    uint256[] private _pendingDemoIds;
    mapping(uint256 => uint256) private _pendingDemoIndex;

    // ============ Constructor ============

    constructor(
        address admin,
        address _robotRegistry,
        address _token
    ) {
        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _grantRole(ADMIN_ROLE, admin);
        _grantRole(VALIDATOR_MANAGER_ROLE, admin);
        _grantRole(BONUS_DISTRIBUTOR_ROLE, admin);

        robotRegistry = IRobotRegistry(_robotRegistry);
        token = IHomeRobotToken(_token);

        // Initialize default reward multipliers
        _initializeMultipliers();
    }

    // ============ Demo Submission Functions ============

    /**
     * @inheritdoc IDataRewards
     */
    function submitDemo(
        uint256 robotId,
        bytes32 contentHash,
        bytes32 metadataHash,
        TaskType taskType,
        uint256 duration,
        bytes calldata teeSignature
    ) external whenNotPaused nonReentrant returns (uint256 demoId) {
        // Verify robot exists and caller is owner
        require(
            robotRegistry.isOwner(msg.sender, robotId),
            "DataRewards: not robot owner"
        );

        // Check for duplicate content
        require(
            _contentHashToDemo[contentHash] == 0,
            "DataRewards: duplicate content"
        );

        // Verify TEE signature
        require(
            _verifyTeeSignature(robotId, contentHash, teeSignature),
            "DataRewards: invalid TEE signature"
        );

        // Verify robot attestation is valid
        require(
            robotRegistry.isAttestationValid(robotId),
            "DataRewards: robot attestation expired"
        );

        // Create demo
        demoId = _nextDemoId++;

        _demos[demoId] = Demo({
            id: demoId,
            robotId: robotId,
            contributor: msg.sender,
            contentHash: contentHash,
            metadataHash: metadataHash,
            taskType: taskType,
            duration: duration,
            submittedAt: block.timestamp,
            status: DemoStatus.Pending,
            qualityScore: 0,
            rewardAmount: 0,
            rewardClaimed: false
        });

        // Update mappings
        _contentHashToDemo[contentHash] = demoId;
        _pendingDemoIds.push(demoId);
        _pendingDemoIndex[demoId] = _pendingDemoIds.length - 1;

        // Update contributor stats
        _contributors[msg.sender].addr = msg.sender;
        _contributors[msg.sender].totalDemos++;

        _totalDemos++;

        emit DemoSubmitted(demoId, robotId, msg.sender, contentHash, taskType);
    }

    /**
     * @inheritdoc IDataRewards
     */
    function batchSubmitDemos(
        uint256[] calldata robotIds,
        bytes32[] calldata contentHashes,
        bytes32[] calldata metadataHashes,
        TaskType[] calldata taskTypes,
        uint256[] calldata durations,
        bytes[] calldata teeSignatures
    ) external whenNotPaused nonReentrant returns (uint256[] memory demoIds) {
        require(
            robotIds.length == contentHashes.length &&
            contentHashes.length == metadataHashes.length &&
            metadataHashes.length == taskTypes.length &&
            taskTypes.length == durations.length &&
            durations.length == teeSignatures.length,
            "DataRewards: array length mismatch"
        );

        demoIds = new uint256[](robotIds.length);

        for (uint256 i = 0; i < robotIds.length; i++) {
            // Simplified checks for batch - full validation in single submit
            require(robotRegistry.isOwner(msg.sender, robotIds[i]), "DataRewards: not robot owner");
            require(_contentHashToDemo[contentHashes[i]] == 0, "DataRewards: duplicate content");

            uint256 demoId = _nextDemoId++;
            demoIds[i] = demoId;

            _demos[demoId] = Demo({
                id: demoId,
                robotId: robotIds[i],
                contributor: msg.sender,
                contentHash: contentHashes[i],
                metadataHash: metadataHashes[i],
                taskType: taskTypes[i],
                duration: durations[i],
                submittedAt: block.timestamp,
                status: DemoStatus.Pending,
                qualityScore: 0,
                rewardAmount: 0,
                rewardClaimed: false
            });

            _contentHashToDemo[contentHashes[i]] = demoId;
            _pendingDemoIds.push(demoId);
            _pendingDemoIndex[demoId] = _pendingDemoIds.length - 1;

            emit DemoSubmitted(demoId, robotIds[i], msg.sender, contentHashes[i], taskTypes[i]);
        }

        _contributors[msg.sender].totalDemos += robotIds.length;
        _totalDemos += robotIds.length;
    }

    // ============ Validation Functions ============

    /**
     * @inheritdoc IDataRewards
     */
    function registerValidator(uint256 stakeAmount) external whenNotPaused nonReentrant {
        require(stakeAmount >= MIN_VALIDATOR_STAKE, "DataRewards: insufficient stake");
        require(!_validators[msg.sender].isActive, "DataRewards: already validator");

        // Transfer stake
        require(
            token.transferFrom(msg.sender, address(this), stakeAmount),
            "DataRewards: stake transfer failed"
        );

        _validators[msg.sender] = Validator({
            addr: msg.sender,
            stakeAmount: stakeAmount,
            validationsCount: 0,
            accuracyScore: 100, // Start with perfect score
            isActive: true,
            slashedAmount: 0
        });

        emit ValidatorRegistered(msg.sender, stakeAmount);
    }

    /**
     * @inheritdoc IDataRewards
     */
    function submitValidation(
        uint256 demoId,
        bool approved,
        uint256 qualityScore,
        bytes32 proofHash
    ) external whenNotPaused {
        require(_validators[msg.sender].isActive, "DataRewards: not active validator");
        require(!_hasVoted[demoId][msg.sender], "DataRewards: already voted");

        Demo storage demo = _demos[demoId];
        require(demo.id != 0, "DataRewards: demo not found");
        require(demo.status == DemoStatus.Pending, "DataRewards: demo not pending");
        require(qualityScore <= QUALITY_SCORE_PRECISION, "DataRewards: invalid quality score");

        // Record vote
        _validationVotes[demoId].push(ValidationVote({
            validator: msg.sender,
            approved: approved,
            qualityScore: qualityScore,
            timestamp: block.timestamp,
            proofHash: proofHash
        }));

        _hasVoted[demoId][msg.sender] = true;
        _validators[msg.sender].validationsCount++;
    }

    /**
     * @inheritdoc IDataRewards
     */
    function finalizeValidation(uint256 demoId) external whenNotPaused {
        Demo storage demo = _demos[demoId];
        require(demo.id != 0, "DataRewards: demo not found");
        require(demo.status == DemoStatus.Pending, "DataRewards: demo not pending");

        ValidationVote[] storage votes = _validationVotes[demoId];
        require(
            votes.length >= MIN_VALIDATORS_FOR_CONSENSUS ||
            block.timestamp >= demo.submittedAt + VALIDATION_PERIOD,
            "DataRewards: validation not ready"
        );

        // Count votes and calculate average quality
        uint256 approvalCount = 0;
        uint256 totalQuality = 0;

        for (uint256 i = 0; i < votes.length; i++) {
            if (votes[i].approved) {
                approvalCount++;
                totalQuality += votes[i].qualityScore;
            }
        }

        // Determine outcome
        bool approved = votes.length > 0 && approvalCount * 2 > votes.length; // >50% approval

        if (approved) {
            demo.status = DemoStatus.Validated;
            demo.qualityScore = approvalCount > 0 ? totalQuality / approvalCount : 0;
            demo.rewardAmount = _calculateReward(demo);
            _contributors[demo.contributor].validatedDemos++;
            _totalValidatedDemos++;
        } else {
            demo.status = DemoStatus.Rejected;
            _contributors[demo.contributor].rejectedDemos++;
        }

        // Remove from pending list
        _removeFromPendingList(demoId);

        // Update contributor reputation
        _updateContributorReputation(demo.contributor, approved);

        emit DemoValidated(demoId, approved, demo.qualityScore, votes.length);
    }

    /**
     * @inheritdoc IDataRewards
     */
    function disputeValidation(uint256 demoId, bytes32 evidence) external whenNotPaused {
        Demo storage demo = _demos[demoId];
        require(demo.id != 0, "DataRewards: demo not found");
        require(
            demo.status == DemoStatus.Validated || demo.status == DemoStatus.Rejected,
            "DataRewards: cannot dispute"
        );
        require(demo.contributor == msg.sender, "DataRewards: not contributor");

        demo.status = DemoStatus.Disputed;

        // TODO: Implement dispute resolution mechanism
        // Could involve governance vote, higher-tier validators, or oracle
    }

    /**
     * @inheritdoc IDataRewards
     */
    function slashValidator(
        address validator,
        uint256 amount,
        string calldata reason
    ) external onlyRole(VALIDATOR_MANAGER_ROLE) {
        Validator storage v = _validators[validator];
        require(v.isActive, "DataRewards: not active validator");

        uint256 slashAmount = amount > v.stakeAmount ? v.stakeAmount : amount;
        v.stakeAmount -= slashAmount;
        v.slashedAmount += slashAmount;

        // Deactivate if stake falls below minimum
        if (v.stakeAmount < MIN_VALIDATOR_STAKE) {
            v.isActive = false;
        }

        emit ValidatorSlashed(validator, slashAmount, reason);
    }

    // ============ Reward Functions ============

    /**
     * @inheritdoc IDataRewards
     */
    function claimReward(uint256 demoId) external whenNotPaused nonReentrant {
        Demo storage demo = _demos[demoId];
        require(demo.id != 0, "DataRewards: demo not found");
        require(demo.contributor == msg.sender, "DataRewards: not contributor");
        require(demo.status == DemoStatus.Validated, "DataRewards: demo not validated");
        require(!demo.rewardClaimed, "DataRewards: reward already claimed");

        demo.rewardClaimed = true;
        demo.status = DemoStatus.Rewarded;

        // Mint reward tokens
        token.mintReward(msg.sender, demo.rewardAmount);

        _contributors[msg.sender].totalRewards += demo.rewardAmount;
        _totalRewardsDistributed += demo.rewardAmount;

        emit RewardClaimed(demoId, msg.sender, demo.rewardAmount);
    }

    /**
     * @inheritdoc IDataRewards
     */
    function batchClaimRewards(uint256[] calldata demoIds) external whenNotPaused nonReentrant {
        uint256 totalReward = 0;

        for (uint256 i = 0; i < demoIds.length; i++) {
            Demo storage demo = _demos[demoIds[i]];

            if (
                demo.id != 0 &&
                demo.contributor == msg.sender &&
                demo.status == DemoStatus.Validated &&
                !demo.rewardClaimed
            ) {
                demo.rewardClaimed = true;
                demo.status = DemoStatus.Rewarded;
                totalReward += demo.rewardAmount;

                emit RewardClaimed(demoIds[i], msg.sender, demo.rewardAmount);
            }
        }

        if (totalReward > 0) {
            token.mintReward(msg.sender, totalReward);
            _contributors[msg.sender].totalRewards += totalReward;
            _totalRewardsDistributed += totalReward;
        }
    }

    /**
     * @inheritdoc IDataRewards
     */
    function calculateReward(uint256 demoId) external view returns (uint256) {
        Demo storage demo = _demos[demoId];
        if (demo.id == 0) return 0;

        return _calculateReward(demo);
    }

    /**
     * @inheritdoc IDataRewards
     */
    function distributeRetrospectiveBonus(
        uint256[] calldata demoIds,
        uint256[] calldata bonusAmounts,
        string calldata reason
    ) external onlyRole(BONUS_DISTRIBUTOR_ROLE) {
        require(demoIds.length == bonusAmounts.length, "DataRewards: array mismatch");

        for (uint256 i = 0; i < demoIds.length; i++) {
            Demo storage demo = _demos[demoIds[i]];
            if (demo.id != 0 && demo.status == DemoStatus.Rewarded) {
                token.mintReward(demo.contributor, bonusAmounts[i]);
                _contributors[demo.contributor].totalRewards += bonusAmounts[i];
                _totalRewardsDistributed += bonusAmounts[i];

                emit BonusDistributed(demoIds[i], demo.contributor, bonusAmounts[i], reason);
            }
        }
    }

    // ============ Staking Functions ============

    /**
     * @inheritdoc IDataRewards
     */
    function stakeAsContributor(uint256 amount) external whenNotPaused nonReentrant {
        require(
            token.transferFrom(msg.sender, address(this), amount),
            "DataRewards: transfer failed"
        );

        _contributors[msg.sender].stakeLocked += amount;
    }

    /**
     * @inheritdoc IDataRewards
     */
    function unstake(uint256 amount) external whenNotPaused nonReentrant {
        Contributor storage contributor = _contributors[msg.sender];
        require(contributor.stakeLocked >= amount, "DataRewards: insufficient stake");

        contributor.stakeLocked -= amount;
        require(token.transfer(msg.sender, amount), "DataRewards: transfer failed");
    }

    /**
     * @inheritdoc IDataRewards
     */
    function addValidatorStake(uint256 amount) external whenNotPaused nonReentrant {
        require(_validators[msg.sender].isActive, "DataRewards: not validator");
        require(
            token.transferFrom(msg.sender, address(this), amount),
            "DataRewards: transfer failed"
        );

        _validators[msg.sender].stakeAmount += amount;
    }

    // ============ View Functions ============

    /**
     * @inheritdoc IDataRewards
     */
    function getDemo(uint256 demoId) external view returns (Demo memory) {
        return _demos[demoId];
    }

    /**
     * @inheritdoc IDataRewards
     */
    function getContributor(address contributor) external view returns (Contributor memory) {
        return _contributors[contributor];
    }

    /**
     * @inheritdoc IDataRewards
     */
    function getValidator(address validator) external view returns (Validator memory) {
        return _validators[validator];
    }

    /**
     * @inheritdoc IDataRewards
     */
    function getRewardMultiplier(TaskType taskType) external view returns (RewardMultiplier memory) {
        return _rewardMultipliers[taskType];
    }

    /**
     * @inheritdoc IDataRewards
     */
    function getPendingDemos(uint256 limit) external view returns (uint256[] memory) {
        uint256 count = limit < _pendingDemoIds.length ? limit : _pendingDemoIds.length;
        uint256[] memory result = new uint256[](count);

        for (uint256 i = 0; i < count; i++) {
            result[i] = _pendingDemoIds[i];
        }

        return result;
    }

    /**
     * @inheritdoc IDataRewards
     */
    function getValidationVotes(uint256 demoId) external view returns (ValidationVote[] memory) {
        return _validationVotes[demoId];
    }

    /**
     * @inheritdoc IDataRewards
     */
    function isDuplicateContent(bytes32 contentHash) external view returns (bool) {
        return _contentHashToDemo[contentHash] != 0;
    }

    /**
     * @inheritdoc IDataRewards
     */
    function getTotalRewardsDistributed() external view returns (uint256) {
        return _totalRewardsDistributed;
    }

    /**
     * @inheritdoc IDataRewards
     */
    function getTotalDemos() external view returns (uint256) {
        return _totalDemos;
    }

    // ============ Admin Functions ============

    /**
     * @notice Update reward multiplier for a task type
     */
    function setRewardMultiplier(
        TaskType taskType,
        uint256 baseMultiplier,
        uint256 noveltyBonus,
        uint256 qualityBonus,
        uint256 edgeCaseBonus
    ) external onlyRole(ADMIN_ROLE) {
        _rewardMultipliers[taskType] = RewardMultiplier({
            taskType: taskType,
            baseMultiplier: baseMultiplier,
            noveltyBonus: noveltyBonus,
            qualityBonus: qualityBonus,
            edgeCaseBonus: edgeCaseBonus
        });

        emit MultiplierUpdated(taskType, baseMultiplier);
    }

    function pause() external onlyRole(ADMIN_ROLE) {
        _pause();
    }

    function unpause() external onlyRole(ADMIN_ROLE) {
        _unpause();
    }

    // ============ Internal Functions ============

    function _initializeMultipliers() internal {
        // Default multipliers (100 = 1x)
        _rewardMultipliers[TaskType.Manipulation] = RewardMultiplier(TaskType.Manipulation, 150, 50, 30, 100);
        _rewardMultipliers[TaskType.Navigation] = RewardMultiplier(TaskType.Navigation, 100, 30, 20, 50);
        _rewardMultipliers[TaskType.HumanInteraction] = RewardMultiplier(TaskType.HumanInteraction, 200, 80, 40, 150);
        _rewardMultipliers[TaskType.Cleaning] = RewardMultiplier(TaskType.Cleaning, 120, 40, 25, 80);
        _rewardMultipliers[TaskType.Cooking] = RewardMultiplier(TaskType.Cooking, 180, 70, 35, 120);
        _rewardMultipliers[TaskType.Organization] = RewardMultiplier(TaskType.Organization, 130, 45, 25, 90);
        _rewardMultipliers[TaskType.Maintenance] = RewardMultiplier(TaskType.Maintenance, 110, 35, 20, 60);
        _rewardMultipliers[TaskType.EdgeCase] = RewardMultiplier(TaskType.EdgeCase, 300, 100, 50, 200);
        _rewardMultipliers[TaskType.Other] = RewardMultiplier(TaskType.Other, 100, 30, 20, 50);
    }

    function _calculateReward(Demo storage demo) internal view returns (uint256) {
        RewardMultiplier storage mult = _rewardMultipliers[demo.taskType];

        // Base reward * task multiplier
        uint256 reward = (BASE_REWARD * mult.baseMultiplier) / 100;

        // Quality bonus (based on quality score 0-100)
        uint256 qualityBonus = (reward * mult.qualityBonus * demo.qualityScore) / (100 * QUALITY_SCORE_PRECISION);
        reward += qualityBonus;

        // Contributor reputation multiplier (reputation is 0-10000, scale to 0.5x - 2x)
        uint256 reputation = _contributors[demo.contributor].reputationScore;
        uint256 repMultiplier = 50 + (reputation * 150) / 10000; // 50-200%
        reward = (reward * repMultiplier) / 100;

        // Contributor stake bonus (extra 20% if staked)
        if (_contributors[demo.contributor].stakeLocked > 0) {
            reward = (reward * 120) / 100;
        }

        return reward;
    }

    function _verifyTeeSignature(
        uint256 robotId,
        bytes32 contentHash,
        bytes calldata signature
    ) internal view returns (bool) {
        // TODO: Implement actual TEE signature verification
        // This would verify the signature against the robot's registered TEE public key
        return signature.length > 0;
    }

    function _removeFromPendingList(uint256 demoId) internal {
        uint256 index = _pendingDemoIndex[demoId];
        uint256 lastIndex = _pendingDemoIds.length - 1;

        if (index != lastIndex) {
            uint256 lastDemoId = _pendingDemoIds[lastIndex];
            _pendingDemoIds[index] = lastDemoId;
            _pendingDemoIndex[lastDemoId] = index;
        }

        _pendingDemoIds.pop();
        delete _pendingDemoIndex[demoId];
    }

    function _updateContributorReputation(address contributor, bool positive) internal {
        Contributor storage c = _contributors[contributor];

        uint256 oldScore = c.reputationScore;

        if (positive) {
            // Increase reputation (diminishing returns)
            uint256 increase = 100 - (c.reputationScore / 200); // Max 100, min ~50
            c.reputationScore = _min(c.reputationScore + increase, 10000);
        } else {
            // Decrease reputation
            uint256 decrease = 200;
            c.reputationScore = decrease > c.reputationScore ? 0 : c.reputationScore - decrease;
        }

        emit ReputationUpdated(contributor, oldScore, c.reputationScore);
    }

    function _min(uint256 a, uint256 b) internal pure returns (uint256) {
        return a < b ? a : b;
    }
}
