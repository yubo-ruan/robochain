//! Validator management for RoboChain

use crate::crypto::{Hash, PublicKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Validator information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    /// Validator's public key
    pub public_key: PublicKey,
    /// Display name
    pub name: String,
    /// Stake amount (in micro-ROBO)
    pub stake: u64,
    /// Whether validator is active
    pub is_active: bool,
    /// Commission rate (basis points, e.g., 1000 = 10%)
    pub commission: u16,
    /// Accumulated rewards
    pub rewards: u64,
    /// Number of blocks proposed
    pub blocks_proposed: u64,
    /// Number of blocks missed
    pub blocks_missed: u64,
    /// Last active timestamp
    pub last_active: u64,
    /// Slashing history count
    pub slashing_count: u32,
    /// Geographic region (for distribution)
    pub region: String,
}

impl Validator {
    pub fn new(public_key: PublicKey, name: String, stake: u64, region: String) -> Self {
        Validator {
            public_key,
            name,
            stake,
            is_active: true,
            commission: 1000, // 10% default
            rewards: 0,
            blocks_proposed: 0,
            blocks_missed: 0,
            last_active: 0,
            slashing_count: 0,
            region,
        }
    }

    /// Calculate voting power (proportional to stake)
    pub fn voting_power(&self, total_stake: u64) -> f64 {
        if total_stake == 0 {
            return 0.0;
        }
        self.stake as f64 / total_stake as f64
    }

    /// Record a proposed block
    pub fn record_proposed(&mut self, timestamp: u64) {
        self.blocks_proposed += 1;
        self.last_active = timestamp;
    }

    /// Record a missed block
    pub fn record_missed(&mut self) {
        self.blocks_missed += 1;
    }

    /// Calculate uptime percentage
    pub fn uptime(&self) -> f64 {
        let total = self.blocks_proposed + self.blocks_missed;
        if total == 0 {
            return 100.0;
        }
        (self.blocks_proposed as f64 / total as f64) * 100.0
    }

    /// Slash validator stake
    pub fn slash(&mut self, percentage: f64) -> u64 {
        let slash_amount = (self.stake as f64 * percentage) as u64;
        self.stake = self.stake.saturating_sub(slash_amount);
        self.slashing_count += 1;
        slash_amount
    }
}

/// Validator set management
#[derive(Debug, Clone)]
pub struct ValidatorSet {
    /// Active validators
    validators: HashMap<PublicKey, Validator>,
    /// Total stake
    total_stake: u64,
    /// Minimum stake required
    min_stake: u64,
    /// Maximum validators allowed
    max_validators: usize,
}

impl ValidatorSet {
    pub fn new(min_stake: u64, max_validators: usize) -> Self {
        ValidatorSet {
            validators: HashMap::new(),
            total_stake: 0,
            min_stake,
            max_validators,
        }
    }

    /// Add a validator
    pub fn add_validator(&mut self, validator: Validator) -> Result<(), ValidatorError> {
        if validator.stake < self.min_stake {
            return Err(ValidatorError::InsufficientStake);
        }
        if self.validators.len() >= self.max_validators {
            return Err(ValidatorError::MaxValidatorsReached);
        }
        if self.validators.contains_key(&validator.public_key) {
            return Err(ValidatorError::AlreadyExists);
        }

        self.total_stake += validator.stake;
        self.validators.insert(validator.public_key, validator);
        Ok(())
    }

    /// Remove a validator
    pub fn remove_validator(&mut self, public_key: &PublicKey) -> Result<Validator, ValidatorError> {
        let validator = self
            .validators
            .remove(public_key)
            .ok_or(ValidatorError::NotFound)?;
        self.total_stake -= validator.stake;
        Ok(validator)
    }

    /// Get a validator
    pub fn get(&self, public_key: &PublicKey) -> Option<&Validator> {
        self.validators.get(public_key)
    }

    /// Get mutable validator
    pub fn get_mut(&mut self, public_key: &PublicKey) -> Option<&mut Validator> {
        self.validators.get_mut(public_key)
    }

    /// Get all validators
    pub fn all(&self) -> Vec<&Validator> {
        self.validators.values().collect()
    }

    /// Get active validators
    pub fn active(&self) -> Vec<&Validator> {
        self.validators.values().filter(|v| v.is_active).collect()
    }

    /// Get total stake
    pub fn total_stake(&self) -> u64 {
        self.total_stake
    }

    /// Select leader for a round (round-robin weighted by stake)
    pub fn select_leader(&self, round: u64) -> Option<PublicKey> {
        let active: Vec<_> = self.active();
        if active.is_empty() {
            return None;
        }

        // Weighted selection based on stake
        let total = self.total_stake;
        if total == 0 {
            return None;
        }

        // Simple round-robin for now
        let index = (round as usize) % active.len();
        Some(active[index].public_key)
    }

    /// Calculate quorum threshold
    pub fn quorum_count(&self, threshold: f64) -> usize {
        let active_count = self.active().len();
        (active_count as f64 * threshold).ceil() as usize
    }

    /// Check if we have quorum
    pub fn has_quorum(&self, voters: &[PublicKey], threshold: f64) -> bool {
        let required = self.quorum_count(threshold);
        let valid_voters: Vec<_> = voters
            .iter()
            .filter(|v| self.validators.contains_key(*v))
            .collect();
        valid_voters.len() >= required
    }

    /// Get validators sorted by stake
    pub fn by_stake(&self) -> Vec<&Validator> {
        let mut validators: Vec<_> = self.validators.values().collect();
        validators.sort_by(|a, b| b.stake.cmp(&a.stake));
        validators
    }

    /// Slash a validator
    pub fn slash(&mut self, public_key: &PublicKey, percentage: f64) -> Result<u64, ValidatorError> {
        let validator = self
            .validators
            .get_mut(public_key)
            .ok_or(ValidatorError::NotFound)?;

        let slashed = validator.slash(percentage);
        self.total_stake -= slashed;

        // Deactivate if below minimum stake
        if validator.stake < self.min_stake {
            validator.is_active = false;
        }

        Ok(slashed)
    }

    /// Update validator stake
    pub fn update_stake(&mut self, public_key: &PublicKey, new_stake: u64) -> Result<(), ValidatorError> {
        let validator = self
            .validators
            .get_mut(public_key)
            .ok_or(ValidatorError::NotFound)?;

        self.total_stake = self.total_stake - validator.stake + new_stake;
        validator.stake = new_stake;

        if new_stake < self.min_stake {
            validator.is_active = false;
        } else {
            validator.is_active = true;
        }

        Ok(())
    }
}

/// Validator errors
#[derive(Debug, Clone)]
pub enum ValidatorError {
    InsufficientStake,
    MaxValidatorsReached,
    AlreadyExists,
    NotFound,
    Unauthorized,
}

/// Validator requirements
#[derive(Debug, Clone)]
pub struct ValidatorRequirements {
    /// Minimum bandwidth in Mbps
    pub min_bandwidth_mbps: u32,
    /// Minimum storage in GB
    pub min_storage_gb: u32,
    /// Minimum CPU cores
    pub min_cpu_cores: u32,
    /// Minimum stake
    pub min_stake: u64,
}

impl Default for ValidatorRequirements {
    fn default() -> Self {
        ValidatorRequirements {
            min_bandwidth_mbps: 1000,  // 1 Gbps
            min_storage_gb: 10_000,    // 10 TB
            min_cpu_cores: 32,
            min_stake: 100_000_000_000, // 100,000 ROBO
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::KeyPair;

    fn create_test_validator(name: &str, stake: u64) -> Validator {
        let keypair = KeyPair::generate();
        Validator::new(
            keypair.public_key(),
            name.to_string(),
            stake,
            "US".to_string(),
        )
    }

    #[test]
    fn test_validator_set() {
        let mut set = ValidatorSet::new(1000, 100);

        let v1 = create_test_validator("Validator 1", 10000);
        let v2 = create_test_validator("Validator 2", 20000);

        set.add_validator(v1).unwrap();
        set.add_validator(v2).unwrap();

        assert_eq!(set.active().len(), 2);
        assert_eq!(set.total_stake(), 30000);
    }

    #[test]
    fn test_quorum() {
        let mut set = ValidatorSet::new(1000, 100);

        for i in 0..4 {
            let v = create_test_validator(&format!("V{}", i), 10000);
            set.add_validator(v).unwrap();
        }

        // 2/3 quorum of 4 = 3
        assert_eq!(set.quorum_count(0.67), 3);
    }

    #[test]
    fn test_slashing() {
        let mut set = ValidatorSet::new(1000, 100);

        let keypair = KeyPair::generate();
        let validator = Validator::new(
            keypair.public_key(),
            "Test".to_string(),
            10000,
            "US".to_string(),
        );
        let pk = validator.public_key;

        set.add_validator(validator).unwrap();

        // Slash 10%
        let slashed = set.slash(&pk, 0.1).unwrap();
        assert_eq!(slashed, 1000);
        assert_eq!(set.get(&pk).unwrap().stake, 9000);
    }
}
