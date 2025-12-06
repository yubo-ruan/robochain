# V1 Implementation Plan: Home Robot Chain

## Executive Summary

This is a comprehensive plan to build a specialized L1 blockchain for home robot coordination. The chain serves as a neutral control plane for identity, permissions, economic settlement, and verifiable behavior summaries—not real-time control.

---

## Phase 1: Foundation Layer

### 1.1 Core Object Model

Define the fundamental domain objects as native chain primitives:

```
Objects:
├── Robot (RobotID, manufacturer, hw_fingerprint, firmware_hash, capabilities)
├── Space (SpaceID, owners[], policy_graph_hash)
├── Principal (PrincipalID, type: owner|guest|service_provider, kyc_level)
├── Task (TaskID, space, requester, capabilities_required, reward, constraints)
├── Policy (PolicyID, grantor, grantee, capability_set, conditions, revocable)
├── Dataset (DatasetID, off_chain_ref, merkle_root, zk_proof_ref)
└── PaymentChannel (ChannelID, payer, payee, rate, conditions, stream_state)
```

**Implementation approach:**
- Each object type gets a dedicated storage module with CRUD operations
- Objects are content-addressed by their ID (hash of creation params)
- All objects carry version numbers for optimistic concurrency

### 1.2 Transaction Types (Core Events)

Implement these as native transaction variants, not smart contract calls:

| Transaction | Purpose | Objects Touched |
|-------------|---------|-----------------|
| `RegisterRobot` | Onboard new robot with HW attestation | Robot |
| `UpdateFirmware` | Record firmware change with vendor sig | Robot |
| `RegisterSpace` | Create home/space with owner | Space, Principal |
| `GrantCapability` | Permission from principal to robot | Policy, Space, Robot |
| `RevokeCapability` | Remove permission (immediate effect) | Policy |
| `PostTask` | Create task with requirements + reward | Task, Space |
| `AcceptTask` | Robot claims task | Task, Robot |
| `SubmitCompletion` | Complete with evidence commitment | Task, Dataset |
| `OpenPaymentStream` | Start conditional payment flow | PaymentChannel |
| `ClosePaymentStream` | Settle and finalize | PaymentChannel |
| `ReportIncident` | Flag safety/policy violation | Task, Robot, Dataset |
| `SetPolicy` | Update space policy graph | Space |

### 1.3 Parallel Execution Engine

**Key insight:** Transactions declare which objects they touch, enabling parallel execution.

```
Execution Model:
1. Transactions arrive in mempool
2. Scheduler builds dependency graph from declared object IDs
3. Non-conflicting transactions execute in parallel batches
4. Conflicting transactions serialize within their batch
```

**Implementation:**
- Use a DAG-based scheduler (similar to Sui's approach but domain-specific)
- Object locks are acquired atomically at batch start
- Failed transactions release locks and retry next block

---

## Phase 2: Consensus Layer

### 2.1 BFT Consensus Design

**Requirements derived from robot use cases:**
- Sub-second block times (target: 500ms)
- High throughput for small structured messages (target: 10,000 TPS)
- Strong finality for capability revocations
- Graceful handling of partitioned robots

**Recommended approach:** Modified HotStuff/Jolteon-style consensus

```
Block Structure:
├── Header (prev_hash, timestamp, validator_sigs)
├── Transactions[] (batched by object partition)
├── StateRoot (merkle root of object store)
└── OffchainCommitments[] (dataset hashes referenced this block)
```

### 2.2 Validator Requirements

Validators are data-center grade nodes, not consumer hardware:

| Requirement | Specification |
|-------------|---------------|
| Bandwidth | 1 Gbps sustained |
| Storage | NVMe, 10TB+ |
| CPU | 32+ cores |
| Stake | Minimum stake in native token |

**Validator selection:** Proof-of-Stake with safety-weighted reputation

### 2.3 Offline Reconciliation Protocol

Robots operating offline accumulate signed local logs:

```
Offline Flow:
1. Robot performs tasks locally, signs each event
2. Events stored on-device with monotonic sequence numbers
3. On reconnect: submit batch with timestamp proofs
4. Chain validates: signatures, sequence, timeout compliance
5. Disputes resolved via evidence commitments
```

---

## Phase 3: Economic Layer

### 3.1 Dual Token Model

| Token | Purpose | Properties |
|-------|---------|------------|
| `$ROBO` | Staking, governance, security | Volatile, used by validators/stakers |
| `$ROUSD` | Payments, fees, task rewards | Stable, bridged from USDC |

**Fee mechanism:**
- Users pay in `$ROUSD`
- Protocol converts portion to `$ROBO` for validator rewards
- Predictable costs for robot operators

### 3.2 Payment Streaming Primitive

Native support for time-based payments:

```
PaymentStream {
  payer: PrincipalID,
  payee: RobotID | PrincipalID,
  rate: u64,           // microUSD per second
  start_time: u64,
  conditions: Vec<Condition>,  // e.g., "valid safety proof each hour"
  escrowed: u64,
  withdrawn: u64,
}
```

**Operations:**
- `extend_stream(amount)` - add more escrow
- `withdraw_earned()` - payee claims accrued
- `pause_stream(reason)` - conditional halt
- `close_stream()` - settle remaining

### 3.3 Staking & Slashing

Three staking pools with distinct slashing conditions:

| Pool | Stakers | Slashing Triggers |
|------|---------|-------------------|
| Validator | Node operators | Consensus faults, downtime |
| Safety | Manufacturers, operators | Verified policy violations |
| Insurance | Anyone | Pays out claims, refilled by premiums |

```
SafetyStake {
  principal: PrincipalID,
  amount: u64,
  robot_ids: Vec<RobotID>,  // robots covered by this stake
  incident_count: u32,
  accumulated_yield: u64,
}
```

---

## Phase 4: Safety & Privacy Layer

### 4.1 On-Chain Evidence Model

**Principle:** Store commitments and proofs, not raw data.

```
EvidenceCommitment {
  dataset_id: DatasetID,
  merkle_root: Hash,
  off_chain_uri: String,  // encrypted, access-controlled
  attestation: SafetyAttestation,
}

SafetyAttestation {
  policy_id: PolicyID,
  constraints_satisfied: Vec<ConstraintProof>,
  // e.g., "path stayed outside forbidden regions"
  // e.g., "force never exceeded 10N"
  verifier_signature: Signature,
}
```

### 4.2 Zero-Knowledge Safety Proofs

Robots generate ZK proofs for safety-critical claims:

| Claim | Proof Type |
|-------|------------|
| "Stayed in allowed regions" | ZK range proof on trajectory |
| "Didn't exceed velocity limit" | ZK bound proof on IMU data |
| "Completed task without entering room X" | ZK exclusion proof |

**Implementation:** Use RISC Zero or SP1 for general ZK-VM proofs

### 4.3 Access Control Contract

```
DataAccessPolicy {
  dataset_id: DatasetID,
  default: Deny,
  rules: [
    { party: Insurer, condition: "incident + owner_sig + manufacturer_sig", access: RedactedLogs },
    { party: Owner, condition: "always", access: Full },
    { party: Manufacturer, condition: "warranty_period", access: DiagnosticOnly },
  ]
}
```

### 4.4 Emergency Primitives

```
EmergencyRevoke {
  trigger: PanicButton | SafetyOracle | OwnerOverride,
  effect: ImmediateCapabilityRevoke | PaymentStreamHalt | RobotLockout,
  scope: SingleRobot | AllInSpace | Global,
}
```

---

## Phase 5: Incentive Mechanisms

### 5.1 Data Bounty System

**Pull model:** Buyers define what data is valuable.

```
DataBounty {
  bounty_id: BountyID,
  poster: PrincipalID,
  criteria: DataCriteria,  // "small object, dark countertop, low light, grasp failure"
  reward_per_sample: u64,
  max_samples: u32,
  accepted_count: u32,
  quality_scorer: OracleID,  // off-chain ML service
}
```

**Flow:**
1. Lab/OEM posts bounty with criteria
2. Robots check if they can capture qualifying data (owner-approved)
3. Robot uploads commitment + proof of criteria match
4. Quality oracle scores sample
5. Payment released based on score

### 5.2 Safety Mining

Reward conservative, safe behavior over time:

```
SafetyMining {
  robot_id: RobotID,
  tasks_completed: u32,
  incidents_verified: u32,
  safety_score: f64,  // derived from task history
  accrued_yield: u64,
}

// Per block, distribute from safety_pool:
yield = base_rate * safety_score * stake_amount
```

**Slashing:** Verified incidents slash accumulated yield + portion of stake.

### 5.3 Task Mining (Growth Incentives)

Bootstrap network effects:

```
TaskMiningReward {
  category: NewGeo | NewTaskType | EarlyAdopter | CriticalService,
  multiplier: f64,
  pool_allocation: u64,
}
```

**Examples:**
- First 100 tasks in a new city: 2x reward multiplier
- Emergency response tasks: 3x multiplier
- Accessibility tasks: 1.5x multiplier

---

## Phase 6: Interoperability

### 6.1 Bridge Architecture

```
                    ┌─────────────────┐
                    │   Ethereum L1   │
                    │  (USDC, $ROBO)  │
                    └────────┬────────┘
                             │ Light client bridge
                    ┌────────▼────────┐
                    │  RoboChain L1   │
                    │ (Native objects)│
                    └────────┬────────┘
                             │ State proofs
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
         ┌────────┐    ┌────────┐    ┌────────┐
         │ Robot  │    │ Robot  │    │ Cloud  │
         │  Node  │    │  Node  │    │Services│
         └────────┘    └────────┘    └────────┘
```

### 6.2 Identity Bridging

Link on-chain robot IDs to:
- Manufacturer registries (X.509 certs, secure element attestations)
- Human identity (optional KYC for professionals)
- Existing DID systems

---

## V1 Milestone Breakdown

### Milestone 1: Object Store & Basic Transactions (Weeks 1-6)
- [ ] Implement core object types (Robot, Space, Principal, Task, Policy)
- [ ] Basic CRUD operations for each object
- [ ] Transaction parsing and validation
- [ ] Simple sequential execution
- [ ] Unit tests for all object operations

### Milestone 2: Parallel Execution Engine (Weeks 7-10)
- [ ] Dependency graph builder from transaction declarations
- [ ] Parallel batch scheduler
- [ ] Object locking and conflict resolution
- [ ] Benchmarks: target 5,000 TPS on test workloads

### Milestone 3: Consensus Integration (Weeks 11-16)
- [ ] Integrate HotStuff-based consensus library
- [ ] Block production and propagation
- [ ] Finality signaling
- [ ] 4-node testnet with sub-second blocks

### Milestone 4: Economic Primitives (Weeks 17-22)
- [ ] Dual token accounting ($ROBO, $ROUSD)
- [ ] Payment streaming implementation
- [ ] Basic staking and unstaking
- [ ] Fee market and conversion mechanism

### Milestone 5: Safety & Evidence Layer (Weeks 23-28)
- [ ] Evidence commitment storage
- [ ] Integration with ZK proof verifier (RISC Zero)
- [ ] Access control contract implementation
- [ ] Emergency revocation primitives

### Milestone 6: Incentive Mechanisms (Weeks 29-34)
- [ ] Data bounty posting and claiming
- [ ] Safety mining reward distribution
- [ ] Task mining multipliers
- [ ] Reputation/history ledger

### Milestone 7: Bridge & Testnet Launch (Weeks 35-40)
- [ ] Light client bridge to Ethereum testnet
- [ ] USDC bridging for $ROUSD
- [ ] Public testnet with 10+ validators
- [ ] Documentation and SDK

---

## Technical Stack Recommendations

| Component | Recommendation | Rationale |
|-----------|----------------|-----------|
| Language | Rust | Performance, safety, ecosystem |
| Consensus | Mysticeti/Bullshark variant | Parallel-friendly, low latency |
| Storage | RocksDB + custom merkle | Proven, fast object lookups |
| Serialization | Borsh | Efficient, deterministic |
| ZK Proofs | RISC Zero / SP1 | General-purpose ZK-VM |
| Networking | libp2p | Mature, widely used |
| Bridge | IBC-style light clients | Battle-tested pattern |

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| ZK proof generation too slow for robots | Use lightweight proof schemes; batch proofs; allow deferred proof submission |
| Privacy leakage through transaction patterns | Add transaction batching, decoy transactions, encrypted mempools |
| Bridge security | Use multi-sig + optimistic verification + fraud proofs |
| Validator centralization | Geographic distribution requirements; delegated staking |
| Regulatory uncertainty | Modular KYC integration; jurisdiction-specific policy modules |

---

## Summary

This V1 plan delivers a specialized blockchain that:

1. **Models the physical world** with native Robot, Space, Task, and Policy objects
2. **Enables parallel execution** through declared object dependencies
3. **Provides sub-second finality** via BFT consensus optimized for structured messages
4. **Handles robot economics** with streaming payments, safety staking, and stable fees
5. **Preserves privacy** by keeping raw data off-chain with on-chain commitments and ZK proofs
6. **Incentivizes good behavior** through data bounties, safety mining, and task mining

The chain is not a general-purpose L1—it's purpose-built infrastructure for robot coordination, accountability, and economic settlement.
