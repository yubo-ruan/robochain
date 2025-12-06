// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

/**
 * @title IHomeRobotToken
 * @notice Interface for the Home Robot L2 native token
 * @dev Single token with utility and governance functions
 */
interface IHomeRobotToken is IERC20 {
    // ============ Structs ============

    struct VestingSchedule {
        uint256 totalAmount;
        uint256 releasedAmount;
        uint256 startTime;
        uint256 cliffDuration;
        uint256 vestingDuration;
        bool revocable;
        bool revoked;
    }

    struct EmissionSchedule {
        uint256 yearlyAmount;
        uint256 startTime;
        uint256 endTime;
        uint256 distributed;
    }

    // ============ Events ============

    event TokensMinted(
        address indexed to,
        uint256 amount,
        string reason
    );

    event TokensBurned(
        address indexed from,
        uint256 amount
    );

    event VestingScheduleCreated(
        address indexed beneficiary,
        uint256 amount,
        uint256 cliffDuration,
        uint256 vestingDuration
    );

    event VestingTokensReleased(
        address indexed beneficiary,
        uint256 amount
    );

    event VestingRevoked(
        address indexed beneficiary,
        uint256 amountRevoked
    );

    event EmissionScheduleUpdated(
        uint256 yearlyAmount,
        uint256 startTime,
        uint256 endTime
    );

    event RewardsPoolFunded(
        uint256 amount
    );

    event MinterAdded(address indexed minter);
    event MinterRemoved(address indexed minter);

    // ============ Minting Functions ============

    /**
     * @notice Mint tokens to an address (only authorized minters)
     * @param to Recipient address
     * @param amount Amount to mint
     * @param reason Reason for minting (for tracking)
     */
    function mint(address to, uint256 amount, string calldata reason) external;

    /**
     * @notice Mint tokens for data rewards (from emission schedule)
     * @param to Recipient address
     * @param amount Amount to mint
     */
    function mintReward(address to, uint256 amount) external;

    /**
     * @notice Burn tokens from caller's balance
     * @param amount Amount to burn
     */
    function burn(uint256 amount) external;

    /**
     * @notice Burn tokens from an address (requires approval)
     * @param from Address to burn from
     * @param amount Amount to burn
     */
    function burnFrom(address from, uint256 amount) external;

    // ============ Vesting Functions ============

    /**
     * @notice Create a vesting schedule for a beneficiary
     * @param beneficiary Address to receive vested tokens
     * @param amount Total tokens to vest
     * @param cliffDuration Cliff period in seconds
     * @param vestingDuration Total vesting duration in seconds
     * @param revocable Whether schedule can be revoked
     */
    function createVestingSchedule(
        address beneficiary,
        uint256 amount,
        uint256 cliffDuration,
        uint256 vestingDuration,
        bool revocable
    ) external;

    /**
     * @notice Release vested tokens to beneficiary
     * @param beneficiary Address to release tokens to
     */
    function releaseVestedTokens(address beneficiary) external;

    /**
     * @notice Revoke a vesting schedule (only if revocable)
     * @param beneficiary Address whose schedule to revoke
     */
    function revokeVestingSchedule(address beneficiary) external;

    /**
     * @notice Get vesting schedule for an address
     * @param beneficiary The beneficiary address
     * @return The vesting schedule
     */
    function getVestingSchedule(address beneficiary) external view returns (VestingSchedule memory);

    /**
     * @notice Get releasable amount for a beneficiary
     * @param beneficiary The beneficiary address
     * @return The amount that can be released
     */
    function getReleasableAmount(address beneficiary) external view returns (uint256);

    // ============ Emission Functions ============

    /**
     * @notice Update emission schedule (governance only)
     * @param yearlyAmount New yearly emission amount
     * @param startTime Start time of new schedule
     * @param endTime End time of new schedule
     */
    function updateEmissionSchedule(
        uint256 yearlyAmount,
        uint256 startTime,
        uint256 endTime
    ) external;

    /**
     * @notice Get current emission schedule
     * @return The emission schedule
     */
    function getEmissionSchedule() external view returns (EmissionSchedule memory);

    /**
     * @notice Get remaining emission budget for current period
     * @return Remaining amount
     */
    function getRemainingEmissionBudget() external view returns (uint256);

    // ============ Authorization Functions ============

    /**
     * @notice Add an authorized minter (governance only)
     * @param minter Address to authorize
     */
    function addMinter(address minter) external;

    /**
     * @notice Remove an authorized minter (governance only)
     * @param minter Address to remove
     */
    function removeMinter(address minter) external;

    /**
     * @notice Check if an address is an authorized minter
     * @param account Address to check
     * @return Whether address is a minter
     */
    function isMinter(address account) external view returns (bool);

    // ============ View Functions ============

    /**
     * @notice Get total supply cap
     * @return Maximum supply
     */
    function maxSupply() external view returns (uint256);

    /**
     * @notice Get circulating supply (excluding vesting, treasury)
     * @return Circulating amount
     */
    function circulatingSupply() external view returns (uint256);

    /**
     * @notice Get total tokens locked in vesting
     * @return Locked amount
     */
    function totalVestingLocked() external view returns (uint256);

    /**
     * @notice Get total rewards distributed
     * @return Distributed amount
     */
    function totalRewardsDistributed() external view returns (uint256);
}
