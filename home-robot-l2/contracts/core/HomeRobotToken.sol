// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Permit.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "../interfaces/IHomeRobotToken.sol";

/**
 * @title HomeRobotToken
 * @notice Native token for the Home Robot L2 network
 * @dev Single token with utility and governance functions
 */
contract HomeRobotToken is
    ERC20,
    ERC20Burnable,
    ERC20Permit,
    AccessControl,
    Pausable,
    ReentrancyGuard,
    IHomeRobotToken
{
    // ============ Constants ============

    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");
    bytes32 public constant VESTING_MANAGER_ROLE = keccak256("VESTING_MANAGER_ROLE");

    uint256 public constant MAX_SUPPLY = 1_000_000_000 * 1e18; // 1 billion tokens

    // ============ State Variables ============

    // Vesting
    mapping(address => VestingSchedule) private _vestingSchedules;
    uint256 private _totalVestingLocked;

    // Emission
    EmissionSchedule private _emissionSchedule;
    uint256 private _totalRewardsDistributed;

    // Minters
    mapping(address => bool) private _minters;

    // Treasury address
    address public treasury;

    // ============ Constructor ============

    constructor(
        address admin,
        address _treasury,
        uint256 initialEmissionYearly,
        uint256 emissionDuration
    ) ERC20("Home Robot Token", "HROBOT") ERC20Permit("Home Robot Token") {
        _grantRole(DEFAULT_ADMIN_ROLE, admin);
        _grantRole(ADMIN_ROLE, admin);
        _grantRole(MINTER_ROLE, admin);
        _grantRole(VESTING_MANAGER_ROLE, admin);

        treasury = _treasury;

        // Initialize emission schedule
        _emissionSchedule = EmissionSchedule({
            yearlyAmount: initialEmissionYearly,
            startTime: block.timestamp,
            endTime: block.timestamp + emissionDuration,
            distributed: 0
        });

        // Mint initial allocations
        _mintInitialAllocations(admin, _treasury);
    }

    // ============ Minting Functions ============

    /**
     * @inheritdoc IHomeRobotToken
     */
    function mint(
        address to,
        uint256 amount,
        string calldata reason
    ) external onlyRole(MINTER_ROLE) whenNotPaused {
        require(totalSupply() + amount <= MAX_SUPPLY, "HomeRobotToken: exceeds max supply");

        _mint(to, amount);
        emit TokensMinted(to, amount, reason);
    }

    /**
     * @inheritdoc IHomeRobotToken
     */
    function mintReward(address to, uint256 amount) external whenNotPaused {
        require(_minters[msg.sender], "HomeRobotToken: not authorized minter");
        require(totalSupply() + amount <= MAX_SUPPLY, "HomeRobotToken: exceeds max supply");

        // Check emission budget
        uint256 remaining = getRemainingEmissionBudget();
        require(amount <= remaining, "HomeRobotToken: exceeds emission budget");

        _emissionSchedule.distributed += amount;
        _totalRewardsDistributed += amount;

        _mint(to, amount);
        emit TokensMinted(to, amount, "data_reward");
    }

    /**
     * @inheritdoc IHomeRobotToken
     */
    function burn(uint256 amount) public override(ERC20Burnable, IHomeRobotToken) {
        super.burn(amount);
        emit TokensBurned(msg.sender, amount);
    }

    /**
     * @inheritdoc IHomeRobotToken
     */
    function burnFrom(
        address from,
        uint256 amount
    ) public override(ERC20Burnable, IHomeRobotToken) {
        super.burnFrom(from, amount);
        emit TokensBurned(from, amount);
    }

    // ============ Vesting Functions ============

    /**
     * @inheritdoc IHomeRobotToken
     */
    function createVestingSchedule(
        address beneficiary,
        uint256 amount,
        uint256 cliffDuration,
        uint256 vestingDuration,
        bool revocable
    ) external onlyRole(VESTING_MANAGER_ROLE) {
        require(beneficiary != address(0), "HomeRobotToken: invalid beneficiary");
        require(amount > 0, "HomeRobotToken: amount must be > 0");
        require(vestingDuration > 0, "HomeRobotToken: duration must be > 0");
        require(
            _vestingSchedules[beneficiary].totalAmount == 0,
            "HomeRobotToken: schedule exists"
        );

        // Transfer tokens to this contract for vesting
        require(
            transfer(address(this), amount),
            "HomeRobotToken: transfer failed"
        );

        _vestingSchedules[beneficiary] = VestingSchedule({
            totalAmount: amount,
            releasedAmount: 0,
            startTime: block.timestamp,
            cliffDuration: cliffDuration,
            vestingDuration: vestingDuration,
            revocable: revocable,
            revoked: false
        });

        _totalVestingLocked += amount;

        emit VestingScheduleCreated(beneficiary, amount, cliffDuration, vestingDuration);
    }

    /**
     * @inheritdoc IHomeRobotToken
     */
    function releaseVestedTokens(address beneficiary) external nonReentrant {
        VestingSchedule storage schedule = _vestingSchedules[beneficiary];
        require(schedule.totalAmount > 0, "HomeRobotToken: no schedule");
        require(!schedule.revoked, "HomeRobotToken: schedule revoked");

        uint256 releasable = _getReleasableAmount(schedule);
        require(releasable > 0, "HomeRobotToken: nothing to release");

        schedule.releasedAmount += releasable;
        _totalVestingLocked -= releasable;

        _transfer(address(this), beneficiary, releasable);

        emit VestingTokensReleased(beneficiary, releasable);
    }

    /**
     * @inheritdoc IHomeRobotToken
     */
    function revokeVestingSchedule(address beneficiary) external onlyRole(VESTING_MANAGER_ROLE) {
        VestingSchedule storage schedule = _vestingSchedules[beneficiary];
        require(schedule.totalAmount > 0, "HomeRobotToken: no schedule");
        require(schedule.revocable, "HomeRobotToken: not revocable");
        require(!schedule.revoked, "HomeRobotToken: already revoked");

        // Release any vested tokens first
        uint256 releasable = _getReleasableAmount(schedule);
        if (releasable > 0) {
            schedule.releasedAmount += releasable;
            _transfer(address(this), beneficiary, releasable);
        }

        // Return unvested tokens to treasury
        uint256 unvested = schedule.totalAmount - schedule.releasedAmount;
        if (unvested > 0) {
            _transfer(address(this), treasury, unvested);
            _totalVestingLocked -= unvested;
        }

        schedule.revoked = true;

        emit VestingRevoked(beneficiary, unvested);
    }

    /**
     * @inheritdoc IHomeRobotToken
     */
    function getVestingSchedule(
        address beneficiary
    ) external view returns (VestingSchedule memory) {
        return _vestingSchedules[beneficiary];
    }

    /**
     * @inheritdoc IHomeRobotToken
     */
    function getReleasableAmount(address beneficiary) external view returns (uint256) {
        VestingSchedule storage schedule = _vestingSchedules[beneficiary];
        if (schedule.totalAmount == 0 || schedule.revoked) return 0;
        return _getReleasableAmount(schedule);
    }

    // ============ Emission Functions ============

    /**
     * @inheritdoc IHomeRobotToken
     */
    function updateEmissionSchedule(
        uint256 yearlyAmount,
        uint256 startTime,
        uint256 endTime
    ) external onlyRole(ADMIN_ROLE) {
        require(startTime < endTime, "HomeRobotToken: invalid time range");

        _emissionSchedule = EmissionSchedule({
            yearlyAmount: yearlyAmount,
            startTime: startTime,
            endTime: endTime,
            distributed: 0
        });

        emit EmissionScheduleUpdated(yearlyAmount, startTime, endTime);
    }

    /**
     * @inheritdoc IHomeRobotToken
     */
    function getEmissionSchedule() external view returns (EmissionSchedule memory) {
        return _emissionSchedule;
    }

    /**
     * @inheritdoc IHomeRobotToken
     */
    function getRemainingEmissionBudget() public view returns (uint256) {
        EmissionSchedule storage schedule = _emissionSchedule;

        if (block.timestamp < schedule.startTime) {
            return 0;
        }

        if (block.timestamp >= schedule.endTime) {
            // Calculate total budget and subtract distributed
            uint256 totalDuration = schedule.endTime - schedule.startTime;
            uint256 totalBudget = (schedule.yearlyAmount * totalDuration) / 365 days;
            return totalBudget > schedule.distributed ? totalBudget - schedule.distributed : 0;
        }

        // Calculate budget up to current time
        uint256 elapsed = block.timestamp - schedule.startTime;
        uint256 budgetToDate = (schedule.yearlyAmount * elapsed) / 365 days;

        return budgetToDate > schedule.distributed ? budgetToDate - schedule.distributed : 0;
    }

    // ============ Authorization Functions ============

    /**
     * @inheritdoc IHomeRobotToken
     */
    function addMinter(address minter) external onlyRole(ADMIN_ROLE) {
        _minters[minter] = true;
        emit MinterAdded(minter);
    }

    /**
     * @inheritdoc IHomeRobotToken
     */
    function removeMinter(address minter) external onlyRole(ADMIN_ROLE) {
        _minters[minter] = false;
        emit MinterRemoved(minter);
    }

    /**
     * @inheritdoc IHomeRobotToken
     */
    function isMinter(address account) external view returns (bool) {
        return _minters[account];
    }

    // ============ View Functions ============

    /**
     * @inheritdoc IHomeRobotToken
     */
    function maxSupply() external pure returns (uint256) {
        return MAX_SUPPLY;
    }

    /**
     * @inheritdoc IHomeRobotToken
     */
    function circulatingSupply() external view returns (uint256) {
        return totalSupply() - balanceOf(treasury) - _totalVestingLocked;
    }

    /**
     * @inheritdoc IHomeRobotToken
     */
    function totalVestingLocked() external view returns (uint256) {
        return _totalVestingLocked;
    }

    /**
     * @inheritdoc IHomeRobotToken
     */
    function totalRewardsDistributed() external view returns (uint256) {
        return _totalRewardsDistributed;
    }

    // ============ Admin Functions ============

    /**
     * @notice Update treasury address
     */
    function setTreasury(address newTreasury) external onlyRole(ADMIN_ROLE) {
        require(newTreasury != address(0), "HomeRobotToken: invalid treasury");
        treasury = newTreasury;
    }

    function pause() external onlyRole(ADMIN_ROLE) {
        _pause();
    }

    function unpause() external onlyRole(ADMIN_ROLE) {
        _unpause();
    }

    // ============ Internal Functions ============

    function _mintInitialAllocations(address admin, address _treasury) internal {
        // Initial allocations (percentages of 1B):
        // - Treasury: 10% (100M) - for ecosystem, grants
        // - Initial liquidity: 5% (50M)
        // Note: Other allocations (team, early contributors) will be vested

        uint256 treasuryAllocation = 100_000_000 * 1e18;
        uint256 liquidityAllocation = 50_000_000 * 1e18;

        _mint(_treasury, treasuryAllocation);
        _mint(admin, liquidityAllocation); // Admin can provide liquidity

        emit TokensMinted(_treasury, treasuryAllocation, "treasury_allocation");
        emit TokensMinted(admin, liquidityAllocation, "initial_liquidity");
    }

    function _getReleasableAmount(
        VestingSchedule storage schedule
    ) internal view returns (uint256) {
        if (block.timestamp < schedule.startTime + schedule.cliffDuration) {
            return 0;
        }

        uint256 elapsed = block.timestamp - schedule.startTime;
        uint256 vested;

        if (elapsed >= schedule.vestingDuration) {
            vested = schedule.totalAmount;
        } else {
            vested = (schedule.totalAmount * elapsed) / schedule.vestingDuration;
        }

        return vested - schedule.releasedAmount;
    }

    // Override required by Solidity for multiple inheritance
    function _update(
        address from,
        address to,
        uint256 amount
    ) internal override whenNotPaused {
        super._update(from, to, amount);
    }
}
