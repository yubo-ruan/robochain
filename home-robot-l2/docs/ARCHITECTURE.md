# Home Robot L2 Architecture

## Overview

A Layer 2 blockchain designed specifically for home robots, providing decentralized identity, permissions, payments, and data collection incentives.

## Stack Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    Home Robot L2 Stack                       │
├─────────────────────────────────────────────────────────────┤
│  Application Layer                                           │
│  ├── Robot Identity Registry                                │
│  ├── Permission Manager                                     │
│  ├── Data Collection Rewards                                │
│  ├── Payment Channels                                       │
│  └── Task Marketplace                                       │
├─────────────────────────────────────────────────────────────┤
│  Execution Layer (Custom Rollup)                            │
│  ├── OP Stack base                                          │
│  ├── Custom precompiles for:                               │
│  │   ├── Trajectory validation                             │
│  │   ├── Hardware attestation verification                 │
│  │   └── Privacy-preserving data checks                    │
│  └── Low block time (< 1s) for responsive UX               │
├─────────────────────────────────────────────────────────────┤
│  Data Availability                                          │
│  ├── Demo data → IPFS/Arweave (large files)                │
│  ├── Metadata/proofs → Celestia or EigenDA (cheap DA)      │
│  └── Critical state → Ethereum calldata (settlement)        │
├─────────────────────────────────────────────────────────────┤
│  Settlement Layer                                           │
│  └── Ethereum mainnet (security inheritance)                │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Robot Identity Registry

Manages robot identities with TEE-based attestation.

**Functions:**
- Register robot with hardware attestation proof
- Link robot to owner address
- Transfer ownership
- Manage robot lifecycle (active/inactive/retired)
- Store hardware fingerprint and capabilities

**Key Properties:**
- Each robot has a unique on-chain identity
- TEE attestation proves robot runs approved firmware
- Ownership is transferable (for resale)

### 2. Permission Manager

Controls what actions robots can perform and who can command them.

**Functions:**
- Define action allowlists/blocklists per user
- Time-based permissions (e.g., cleaning only 9am-5pm)
- Geofencing rules (on-chain boundary definitions)
- Emergency stop registry
- Guest access tokens (temporary permissions)

**Offline Support:**
- Permissions cached locally on robot
- Merkle proofs for verification without network
- Graceful degradation policy

### 3. Data Collection Rewards (UMI-style)

Incentivizes users to collect and share robot demonstration data.

**Data Types:**
- Teleoperation demos (trajectories, actions)
- Task success/failure labels
- Environment scans (3D, semantic)
- Edge case corrections
- Natural language commands + execution pairs

**Reward Mechanism:**
- Base reward per valid demo
- Multipliers for: novel tasks, rare objects, high quality, edge cases
- Reputation system affects rewards
- Retrospective bonuses when data improves models

**Validation:**
- Automated quality checks (trajectory smoothness, completion)
- Validator network (staked validators sample submissions)
- Retrospective validation (data that helps training = bonus)

### 4. Payment Channels

Handles micropayments and subscriptions.

**Use Cases:**
- Pay-per-task billing
- Subscription services
- Robot-as-a-service (rent to neighbors)
- Consumables auto-ordering
- Energy accounting

### 5. Task Marketplace

Marketplace for robot skills and task templates.

**Features:**
- Skill NFTs (downloadable capabilities)
- Task templates (shareable routines)
- Cross-robot skill transfer
- Task reputation scores

## Token Economics

### Single Token Model

One token serves both utility and governance functions.

**Utility:**
- Stake to become data validator
- Pay for model access / premium features
- Earn from data contributions
- Pay for task marketplace items

**Governance:**
- Vote on protocol upgrades
- Vote on data policies
- Vote on reward parameters

### Supply & Distribution

```
Total Supply: 1,000,000,000 tokens

Allocation:
├── Data Rewards Pool:     40% (400M) - Emitted over 10 years
├── Ecosystem Fund:        20% (200M) - Grants, partnerships
├── Team & Advisors:       15% (150M) - 4-year vesting, 1-year cliff
├── Early Contributors:    10% (100M) - 2-year vesting
├── Treasury:              10% (100M) - Protocol-owned liquidity
└── Initial Liquidity:      5% (50M)  - DEX liquidity
```

### Emission Schedule

Data rewards follow a halving schedule:
- Year 1-2: 100M tokens
- Year 3-4: 80M tokens
- Year 5-6: 60M tokens
- Year 7-8: 40M tokens
- Year 9-10: 20M tokens

## Validator Network

Three-tier validation for data quality:

### Tier 1: Automated Checks (On-chain/Off-chain hybrid)
- Trajectory smoothness score
- Task completion detection
- Duplicate/near-duplicate detection
- Sensor data consistency
- File format validation

### Tier 2: Validator Network (Staked validators)
- Random sampling of submissions
- Human review of edge cases
- Slash for approving bad data
- Reward for catching bad data

### Tier 3: AI Model Validation
- Run submitted demos through evaluation models
- Check for adversarial/poisoning attempts
- Semantic consistency checks

### Validator Requirements
- Minimum stake: 10,000 tokens
- Slashing: Up to 10% for approving fraudulent data
- Rewards: Share of data submission fees

## TEE Integration

Robots must have Trusted Execution Environment for:

### Attestation
- Prove robot runs approved firmware
- Generate hardware fingerprint
- Sign attestation proofs

### Key Management
- Private keys generated in TEE
- Never exposed to main OS
- Secure transaction signing

### Supported TEEs
- ARM TrustZone (most consumer robots)
- Intel SGX (high-end robots)
- AMD SEV (server-side validation)

## Offline Architecture

Robots must function when network is unavailable.

### Permission Caching
```
┌─────────────────────────────────────────────┐
│           On-Robot Permission Cache          │
├─────────────────────────────────────────────┤
│  Merkle Root (from L2)                      │
│  ├── User permissions (Merkle proofs)       │
│  ├── Action allowlists                      │
│  ├── Time-based rules                       │
│  └── Emergency contacts                     │
├─────────────────────────────────────────────┤
│  Sync Strategy:                             │
│  ├── Pull on boot                           │
│  ├── Periodic refresh (every 1h online)    │
│  └── Push notifications for critical updates│
└─────────────────────────────────────────────┘
```

### Transaction Queuing
- Queue transactions when offline
- Batch submit when online
- Conflict resolution via timestamps
- Maximum offline duration: 7 days

### Degradation Policy
- Core safety functions always work
- Earning rewards requires online verification
- High-risk actions may require online check

## Data Flow

### Demo Submission Flow
```
1. User records demo on robot
2. On-device preprocessing (privacy filters)
3. Upload to IPFS/Arweave → get content hash
4. Submit metadata to L2:
   - Content hash
   - Task type
   - Duration
   - Robot ID
   - TEE signature
5. Validators sample and score
6. Rewards distributed after validation period
7. Data available for model training
```

### Privacy Preservation
- Face detection + blurring on-device
- Background anonymization option
- Selective room recording
- ZK proofs for "robot was in kitchen" without video
- Data expiry after N training runs

## Smart Contract Architecture

```
contracts/
├── core/
│   ├── RobotRegistry.sol      # Robot identity management
│   ├── PermissionManager.sol  # Access control
│   ├── DataRewards.sol        # Data collection incentives
│   └── HomeRobotToken.sol     # Native token
├── interfaces/
│   ├── IRobotRegistry.sol
│   ├── IPermissionManager.sol
│   ├── IDataRewards.sol
│   └── IHomeRobotToken.sol
└── libraries/
    ├── AttestationLib.sol     # TEE attestation helpers
    ├── MerkleProofLib.sol     # Permission proof verification
    └── RewardCalculator.sol   # Reward math
```

## Roadmap

### Phase 1: Foundation
- [ ] Deploy OP Stack devnet
- [ ] Core contracts (Registry, Permissions, Token)
- [ ] Basic TEE integration

### Phase 2: Data Layer
- [ ] Data submission pipeline
- [ ] Validator network
- [ ] Reward distribution

### Phase 3: Robot Integration
- [ ] On-robot SDK
- [ ] Offline caching
- [ ] Demo recording pipeline

### Phase 4: Ecosystem
- [ ] Task marketplace
- [ ] Payment channels
- [ ] Cross-chain bridges

### Phase 5: Mainnet
- [ ] Security audits
- [ ] Mainnet deployment
- [ ] Token launch
