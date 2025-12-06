# RoboChain Architecture - Phase 1 & 2

## Visual Overview

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              ROBOCHAIN ARCHITECTURE                              │
│                    A Specialized L1 for Home Robot Coordination                  │
└─────────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────────┐
│                                  PHASE 1: FOUNDATION                             │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │                         CORE OBJECT MODEL                                │    │
│  │                                                                          │    │
│  │   ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐               │    │
│  │   │  Robot   │  │  Space   │  │Principal │  │   Task   │               │    │
│  │   ├──────────┤  ├──────────┤  ├──────────┤  ├──────────┤               │    │
│  │   │• id      │  │• id      │  │• id      │  │• id      │               │    │
│  │   │• mfr     │  │• name    │  │• name    │  │• space_id│               │    │
│  │   │• model   │  │• owners[]│  │• type    │  │• requester│              │    │
│  │   │• firmware│  │• zones[] │  │• kyc_lvl │  │• reward  │               │    │
│  │   │• caps[]  │  │• policies│  │• pubkey  │  │• status  │               │    │
│  │   │• status  │  │• geo     │  │• rep     │  │• robot   │               │    │
│  │   └──────────┘  └──────────┘  └──────────┘  └──────────┘               │    │
│  │                                                                          │    │
│  │   ┌──────────┐  ┌──────────┐  ┌──────────────────┐                      │    │
│  │   │  Policy  │  │ Dataset  │  │ PaymentChannel   │                      │    │
│  │   ├──────────┤  ├──────────┤  ├──────────────────┤                      │    │
│  │   │• grantor │  │• type    │  │• payer           │                      │    │
│  │   │• grantee │  │• robot   │  │• payee           │                      │    │
│  │   │• caps[]  │  │• uri     │  │• rate            │                      │    │
│  │   │• zones[] │  │• merkle  │  │• escrowed        │                      │    │
│  │   │• conditions│ │• proofs[]│ │• withdrawn       │                      │    │
│  │   └──────────┘  └──────────┘  └──────────────────┘                      │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │                         TRANSACTION TYPES                                │    │
│  │                                                                          │    │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐          │    │
│  │  │ Robot Lifecycle │  │ Space Mgmt      │  │ Task Lifecycle  │          │    │
│  │  ├─────────────────┤  ├─────────────────┤  ├─────────────────┤          │    │
│  │  │• RegisterRobot  │  │• RegisterSpace  │  │• PostTask       │          │    │
│  │  │• UpdateFirmware │  │• SetPolicy      │  │• AcceptTask     │          │    │
│  │  │• TransferRobot  │  │• AddZone        │  │• StartTask      │          │    │
│  │  │• UpdateStatus   │  │                 │  │• SubmitComplete │          │    │
│  │  └─────────────────┘  └─────────────────┘  │• CancelTask     │          │    │
│  │                                            └─────────────────┘          │    │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐          │    │
│  │  │ Permissions     │  │ Payments        │  │ Data/Evidence   │          │    │
│  │  ├─────────────────┤  ├─────────────────┤  ├─────────────────┤          │    │
│  │  │• GrantCapability│  │• OpenStream     │  │• SubmitDataset  │          │    │
│  │  │• RevokeCapability│ │• ExtendStream   │  │• ReportIncident │          │    │
│  │  │• EmergencyRevoke│  │• WithdrawPmt    │  │                 │          │    │
│  │  │                 │  │• CloseStream    │  │                 │          │    │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────┘          │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │                      PARALLEL EXECUTION ENGINE                           │    │
│  │                                                                          │    │
│  │  ┌─────────────────────────────────────────────────────────────────┐    │    │
│  │  │                     TRANSACTION SCHEDULER                        │    │    │
│  │  │                                                                  │    │    │
│  │  │   Mempool          Dependency          Parallel                  │    │    │
│  │  │   ┌─────┐          Analysis           Batches                    │    │    │
│  │  │   │Tx1  │    ─►    ┌────────┐    ─►   ┌─────────────────┐       │    │    │
│  │  │   │Tx2  │          │ DAG    │         │ Batch 1: Tx1,Tx3│       │    │    │
│  │  │   │Tx3  │          │ Build  │         │ Batch 2: Tx2    │       │    │    │
│  │  │   │Tx4  │          │        │         │ Batch 3: Tx4,Tx5│       │    │    │
│  │  │   │Tx5  │          └────────┘         └─────────────────┘       │    │    │
│  │  │   └─────┘                                                        │    │    │
│  │  └─────────────────────────────────────────────────────────────────┘    │    │
│  │                                                                          │    │
│  │  ┌─────────────────────────────────────────────────────────────────┐    │    │
│  │  │                      BATCH EXECUTOR                              │    │    │
│  │  │                                                                  │    │    │
│  │  │   ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐      │    │    │
│  │  │   │Thread 1 │    │Thread 2 │    │Thread 3 │    │Thread 4 │      │    │    │
│  │  │   │ ┌─────┐ │    │ ┌─────┐ │    │ ┌─────┐ │    │ ┌─────┐ │      │    │    │
│  │  │   │ │ Tx1 │ │    │ │ Tx3 │ │    │ │ Tx4 │ │    │ │ Tx5 │ │      │    │    │
│  │  │   │ └─────┘ │    │ └─────┘ │    │ └─────┘ │    │ └─────┘ │      │    │    │
│  │  │   └────┬────┘    └────┬────┘    └────┬────┘    └────┬────┘      │    │    │
│  │  │        │              │              │              │            │    │    │
│  │  │        └──────────────┴──────────────┴──────────────┘            │    │    │
│  │  │                            │                                      │    │    │
│  │  │                      ┌─────▼─────┐                               │    │    │
│  │  │                      │  Object   │                               │    │    │
│  │  │                      │   Store   │                               │    │    │
│  │  │                      └───────────┘                               │    │    │
│  │  └─────────────────────────────────────────────────────────────────┘    │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────────┐
│                                  PHASE 2: CONSENSUS                              │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │                          BFT CONSENSUS ENGINE                            │    │
│  │                                                                          │    │
│  │   ┌────────────────────────────────────────────────────────────────┐    │    │
│  │   │                    CONSENSUS ROUND                              │    │    │
│  │   │                                                                 │    │    │
│  │   │  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐ │    │    │
│  │   │  │  IDLE    │───►│ PROPOSE  │───►│  VOTE    │───►│ COMMIT   │ │    │    │
│  │   │  │          │    │          │    │          │    │          │ │    │    │
│  │   │  │Wait for  │    │Leader    │    │Validators│    │Quorum    │ │    │    │
│  │   │  │block time│    │creates   │    │vote on   │    │reached,  │ │    │    │
│  │   │  │(500ms)   │    │block     │    │proposal  │    │finalize  │ │    │    │
│  │   │  └──────────┘    └──────────┘    └──────────┘    └────┬─────┘ │    │    │
│  │   │                                                       │       │    │    │
│  │   │                                                       ▼       │    │    │
│  │   │                                              ┌──────────────┐ │    │    │
│  │   │                                              │   FINALITY   │ │    │    │
│  │   │                                              │  Certificate │ │    │    │
│  │   │                                              └──────────────┘ │    │    │
│  │   └────────────────────────────────────────────────────────────────┘    │    │
│  │                                                                          │    │
│  │   ┌────────────────────────────────────────────────────────────────┐    │    │
│  │   │                      BLOCK STRUCTURE                            │    │    │
│  │   │                                                                 │    │    │
│  │   │   ┌─────────────────────────────────────────────────────────┐  │    │    │
│  │   │   │                    BLOCK HEADER                          │  │    │    │
│  │   │   ├─────────────────────────────────────────────────────────┤  │    │    │
│  │   │   │ height: 42          │ timestamp: 1699900000            │  │    │    │
│  │   │   │ prev_hash: 0x7a3f...│ proposer: 0x8b2c...              │  │    │    │
│  │   │   │ tx_root: 0x4d1e...  │ state_root: 0x9f5a...            │  │    │    │
│  │   │   │ offchain_root: 0x...│ signature: 0x...                 │  │    │    │
│  │   │   └─────────────────────────────────────────────────────────┘  │    │    │
│  │   │   ┌─────────────────────────────────────────────────────────┐  │    │    │
│  │   │   │                   TRANSACTIONS                           │  │    │    │
│  │   │   │   [RegisterRobot, AcceptTask, SubmitCompletion, ...]    │  │    │    │
│  │   │   └─────────────────────────────────────────────────────────┘  │    │    │
│  │   │   ┌─────────────────────────────────────────────────────────┐  │    │    │
│  │   │   │               OFFCHAIN COMMITMENTS                       │  │    │    │
│  │   │   │   [DatasetHash, EvidenceHash, TrajectoryHash, ...]      │  │    │    │
│  │   │   └─────────────────────────────────────────────────────────┘  │    │    │
│  │   └────────────────────────────────────────────────────────────────┘    │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │                           VALIDATOR SET                                  │    │
│  │                                                                          │    │
│  │   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │    │
│  │   │ Validator 1 │  │ Validator 2 │  │ Validator 3 │  │ Validator 4 │    │    │
│  │   ├─────────────┤  ├─────────────┤  ├─────────────┤  ├─────────────┤    │    │
│  │   │Stake: 100K  │  │Stake: 150K  │  │Stake: 80K   │  │Stake: 120K  │    │    │
│  │   │Region: US   │  │Region: EU   │  │Region: APAC │  │Region: US   │    │    │
│  │   │Uptime: 99.9%│  │Uptime: 99.8%│  │Uptime: 99.7%│  │Uptime: 99.9%│    │    │
│  │   └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘    │    │
│  │                                                                          │    │
│  │   Quorum: 2/3 + 1 = 3 validators required for consensus                 │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │                     OFFLINE RECONCILIATION PROTOCOL                      │    │
│  │                                                                          │    │
│  │   ┌──────────────────────────────────────────────────────────────────┐  │    │
│  │   │                    ROBOT OFFLINE OPERATION                        │  │    │
│  │   │                                                                   │  │    │
│  │   │  ┌────────────┐      ┌────────────┐      ┌────────────┐          │  │    │
│  │   │  │ Local Task │  ─►  │ Event Log  │  ─►  │  Signed    │          │  │    │
│  │   │  │ Execution  │      │ Recording  │      │  Events    │          │  │    │
│  │   │  └────────────┘      └────────────┘      └────────────┘          │  │    │
│  │   │                                                │                  │  │    │
│  │   │                                                ▼                  │  │    │
│  │   │  ┌──────────────────────────────────────────────────────────┐    │  │    │
│  │   │  │                 ON RECONNECT                              │    │  │    │
│  │   │  │                                                           │    │  │    │
│  │   │  │  ┌─────────┐  ┌─────────────┐  ┌─────────────┐           │    │  │    │
│  │   │  │  │ Create  │─►│  Validate   │─►│  Submit to  │           │    │  │    │
│  │   │  │  │  Batch  │  │  Signatures │  │    Chain    │           │    │  │    │
│  │   │  │  └─────────┘  └─────────────┘  └─────────────┘           │    │  │    │
│  │   │  │                                       │                   │    │  │    │
│  │   │  │                                       ▼                   │    │  │    │
│  │   │  │                              ┌─────────────┐              │    │  │    │
│  │   │  │                              │ Reconcile & │              │    │  │    │
│  │   │  │                              │ Settle Pmts │              │    │  │    │
│  │   │  │                              └─────────────┘              │    │  │    │
│  │   │  └──────────────────────────────────────────────────────────┘    │  │    │
│  │   └──────────────────────────────────────────────────────────────────┘  │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────────┐
│                              DATA FLOW OVERVIEW                                  │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   ┌─────────┐      ┌─────────────┐      ┌──────────────┐      ┌────────────┐   │
│   │  User   │ ───► │ Transaction │ ───► │   Mempool    │ ───► │  Scheduler │   │
│   │ Client  │      │  Creation   │      │              │      │            │   │
│   └─────────┘      └─────────────┘      └──────────────┘      └─────┬──────┘   │
│                                                                      │          │
│                                                                      ▼          │
│   ┌─────────┐      ┌─────────────┐      ┌──────────────┐      ┌────────────┐   │
│   │  State  │ ◄─── │  Execution  │ ◄─── │   Parallel   │ ◄─── │   Batch    │   │
│   │  Store  │      │   Results   │      │   Execution  │      │  Creation  │   │
│   └─────────┘      └─────────────┘      └──────────────┘      └────────────┘   │
│       │                                                                         │
│       ▼                                                                         │
│   ┌─────────┐      ┌─────────────┐      ┌──────────────┐      ┌────────────┐   │
│   │  Block  │ ───► │  Consensus  │ ───► │   Finality   │ ───► │  Receipts  │   │
│   │ Propose │      │   Voting    │      │ Certificate  │      │  & Events  │   │
│   └─────────┘      └─────────────┘      └──────────────┘      └────────────┘   │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

## Module Structure

```
robochain/
├── src/
│   ├── lib.rs              # Library entry point
│   ├── main.rs             # Node binary
│   │
│   ├── objects/            # Phase 1: Core Domain Objects
│   │   ├── mod.rs          # Object traits and AnyObject enum
│   │   ├── object_id.rs    # ID types (RobotId, SpaceId, etc.)
│   │   ├── robot.rs        # Robot object
│   │   ├── space.rs        # Space/home object
│   │   ├── principal.rs    # Human/entity object
│   │   ├── task.rs         # Task/job object
│   │   ├── policy.rs       # Permission grant object
│   │   ├── dataset.rs      # Off-chain data reference
│   │   └── payment_channel.rs  # Streaming payment
│   │
│   ├── transactions/       # Phase 1: Transaction Types
│   │   ├── mod.rs          # Transaction wrapper
│   │   ├── types.rs        # All transaction payloads
│   │   ├── validation.rs   # Validation logic
│   │   └── effects.rs      # Execution & state effects
│   │
│   ├── execution/          # Phase 1: Parallel Execution
│   │   ├── mod.rs          # Engine configuration
│   │   ├── scheduler.rs    # DAG-based tx scheduler
│   │   ├── batch.rs        # Batch execution
│   │   └── executor.rs     # Main execution engine
│   │
│   ├── consensus/          # Phase 2: Consensus Layer
│   │   ├── mod.rs          # Consensus types & messages
│   │   ├── block.rs        # Block structure
│   │   ├── validator.rs    # Validator management
│   │   ├── engine.rs       # BFT consensus engine
│   │   └── offline.rs      # Offline reconciliation
│   │
│   ├── storage/            # Storage Layer
│   │   ├── mod.rs          # Storage types
│   │   ├── traits.rs       # ObjectStore trait
│   │   └── memory.rs       # In-memory implementation
│   │
│   ├── crypto/             # Cryptographic Primitives
│   │   └── mod.rs          # Hash, KeyPair, Signature
│   │
│   └── network/            # Network Layer (stub)
│       └── mod.rs          # P2P networking placeholder
│
├── Cargo.toml              # Project configuration
├── IMPLEMENTATION_PLAN.md  # Full implementation plan
└── ARCHITECTURE.md         # This file
```

## Key Design Decisions

### 1. Object-Centric Model
- Domain objects (Robot, Space, Task, etc.) are first-class citizens
- Each object has a unique ID derived from its creation parameters
- Objects carry version numbers for optimistic concurrency

### 2. Parallel Execution via Declared Dependencies
- Transactions declare which objects they read/write
- Scheduler builds dependency graph and groups non-conflicting txs
- Batches execute in parallel with proper isolation

### 3. BFT Consensus with Sub-Second Finality
- HotStuff-inspired consensus with round-robin leadership
- 500ms target block time
- 2/3 + 1 quorum for finality

### 4. Offline-First Robot Design
- Robots record signed events locally when offline
- Batch reconciliation on reconnect
- Sequence numbers prevent replay attacks

## Performance Targets

| Metric | Target | Current |
|--------|--------|---------|
| Block Time | 500ms | Configurable |
| TPS | 10,000 | Testing needed |
| Finality | < 1s | 1 block |
| Parallel Batches | Auto-scaled | Based on conflicts |
