#!/usr/bin/env python3
"""
RoboChain Architecture Visualizer
Generates visual diagrams of the Phase 1 & 2 implementation
"""

import os
import sys

# ANSI color codes
class Colors:
    HEADER = '\033[95m'
    BLUE = '\033[94m'
    CYAN = '\033[96m'
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    RED = '\033[91m'
    BOLD = '\033[1m'
    UNDERLINE = '\033[4m'
    END = '\033[0m'
    WHITE = '\033[97m'
    GRAY = '\033[90m'

def print_header():
    print(f"""
{Colors.CYAN}{Colors.BOLD}
╔══════════════════════════════════════════════════════════════════════════════╗
║                                                                              ║
║   ██████╗  ██████╗ ██████╗  ██████╗  ██████╗██╗  ██╗ █████╗ ██╗███╗   ██╗   ║
║   ██╔══██╗██╔═══██╗██╔══██╗██╔═══██╗██╔════╝██║  ██║██╔══██╗██║████╗  ██║   ║
║   ██████╔╝██║   ██║██████╔╝██║   ██║██║     ███████║███████║██║██╔██╗ ██║   ║
║   ██╔══██╗██║   ██║██╔══██╗██║   ██║██║     ██╔══██║██╔══██║██║██║╚██╗██║   ║
║   ██║  ██║╚██████╔╝██████╔╝╚██████╔╝╚██████╗██║  ██║██║  ██║██║██║ ╚████║   ║
║   ╚═╝  ╚═╝ ╚═════╝ ╚═════╝  ╚═════╝  ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝   ║
║                                                                              ║
║              A Specialized L1 Blockchain for Home Robot Coordination         ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
{Colors.END}
""")

def print_phase1_objects():
    print(f"""
{Colors.YELLOW}{Colors.BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
                         PHASE 1: CORE OBJECT MODEL
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{Colors.END}

{Colors.GREEN}  ┌─────────────────────────────────────────────────────────────────────────┐
  │                         DOMAIN OBJECTS                                   │
  └─────────────────────────────────────────────────────────────────────────┘{Colors.END}

    {Colors.CYAN}┌──────────────┐{Colors.END}    {Colors.CYAN}┌──────────────┐{Colors.END}    {Colors.CYAN}┌──────────────┐{Colors.END}    {Colors.CYAN}┌──────────────┐{Colors.END}
    {Colors.CYAN}│{Colors.END}    {Colors.BOLD}ROBOT{Colors.END}     {Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END}    {Colors.BOLD}SPACE{Colors.END}     {Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END}  {Colors.BOLD}PRINCIPAL{Colors.END}  {Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END}    {Colors.BOLD}TASK{Colors.END}      {Colors.CYAN}│{Colors.END}
    {Colors.CYAN}├──────────────┤{Colors.END}    {Colors.CYAN}├──────────────┤{Colors.END}    {Colors.CYAN}├──────────────┤{Colors.END}    {Colors.CYAN}├──────────────┤{Colors.END}
    {Colors.CYAN}│{Colors.END} • id         {Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END} • id         {Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END} • id         {Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END} • id         {Colors.CYAN}│{Colors.END}
    {Colors.CYAN}│{Colors.END} • manufacturer{Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END} • name       {Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END} • name       {Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END} • space_id   {Colors.CYAN}│{Colors.END}
    {Colors.CYAN}│{Colors.END} • model      {Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END} • owners[]   {Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END} • type       {Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END} • requester  {Colors.CYAN}│{Colors.END}
    {Colors.CYAN}│{Colors.END} • firmware   {Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END} • zones[]    {Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END} • kyc_level  {Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END} • reward     {Colors.CYAN}│{Colors.END}
    {Colors.CYAN}│{Colors.END} • capabilities{Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END} • geo_region {Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END} • reputation {Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END} • status     {Colors.CYAN}│{Colors.END}
    {Colors.CYAN}│{Colors.END} • status     {Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END} • policies   {Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END} • pubkey     {Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END} • constraints{Colors.CYAN}│{Colors.END}
    {Colors.CYAN}└──────────────┘{Colors.END}    {Colors.CYAN}└──────────────┘{Colors.END}    {Colors.CYAN}└──────────────┘{Colors.END}    {Colors.CYAN}└──────────────┘{Colors.END}

    {Colors.CYAN}┌──────────────┐{Colors.END}    {Colors.CYAN}┌──────────────┐{Colors.END}    {Colors.CYAN}┌────────────────────────┐{Colors.END}
    {Colors.CYAN}│{Colors.END}   {Colors.BOLD}POLICY{Colors.END}    {Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END}   {Colors.BOLD}DATASET{Colors.END}   {Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END}   {Colors.BOLD}PAYMENT CHANNEL{Colors.END}    {Colors.CYAN}│{Colors.END}
    {Colors.CYAN}├──────────────┤{Colors.END}    {Colors.CYAN}├──────────────┤{Colors.END}    {Colors.CYAN}├────────────────────────┤{Colors.END}
    {Colors.CYAN}│{Colors.END} • grantor    {Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END} • type       {Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END} • payer / payee        {Colors.CYAN}│{Colors.END}
    {Colors.CYAN}│{Colors.END} • grantee    {Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END} • robot_id   {Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END} • rate_per_second      {Colors.CYAN}│{Colors.END}
    {Colors.CYAN}│{Colors.END} • capabilities{Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END} • merkle_root{Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END} • escrowed / withdrawn {Colors.CYAN}│{Colors.END}
    {Colors.CYAN}│{Colors.END} • zones[]    {Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END} • proofs[]   {Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END} • conditions[]         {Colors.CYAN}│{Colors.END}
    {Colors.CYAN}│{Colors.END} • conditions {Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END} • off_chain  {Colors.CYAN}│{Colors.END}    {Colors.CYAN}│{Colors.END} • status               {Colors.CYAN}│{Colors.END}
    {Colors.CYAN}└──────────────┘{Colors.END}    {Colors.CYAN}└──────────────┘{Colors.END}    {Colors.CYAN}└────────────────────────┘{Colors.END}
""")

def print_phase1_transactions():
    print(f"""
{Colors.GREEN}  ┌─────────────────────────────────────────────────────────────────────────┐
  │                         TRANSACTION TYPES                                │
  └─────────────────────────────────────────────────────────────────────────┘{Colors.END}

    {Colors.BLUE}Robot Lifecycle{Colors.END}         {Colors.BLUE}Space Management{Colors.END}        {Colors.BLUE}Task Lifecycle{Colors.END}
    ─────────────────       ──────────────────       ────────────────
    • RegisterRobot         • RegisterSpace          • PostTask
    • UpdateFirmware        • SetPolicy              • AcceptTask
    • TransferRobot         • AddZone                • StartTask
    • UpdateRobotStatus                              • SubmitCompletion
                                                     • CancelTask

    {Colors.BLUE}Permissions{Colors.END}             {Colors.BLUE}Payments{Colors.END}                {Colors.BLUE}Data & Evidence{Colors.END}
    ─────────────           ──────────              ─────────────────
    • GrantCapability       • OpenPaymentStream      • SubmitDataset
    • RevokeCapability      • ExtendPaymentStream    • ReportIncident
    • EmergencyRevoke       • WithdrawPayment
                            • ClosePaymentStream
""")

def print_phase1_execution():
    print(f"""
{Colors.GREEN}  ┌─────────────────────────────────────────────────────────────────────────┐
  │                      PARALLEL EXECUTION ENGINE                           │
  └─────────────────────────────────────────────────────────────────────────┘{Colors.END}

    {Colors.YELLOW}Step 1: Transaction Scheduling{Colors.END}

    ┌─────────────┐         ┌─────────────────────┐         ┌─────────────────┐
    │   MEMPOOL   │         │   DAG DEPENDENCY    │         │    PARALLEL     │
    │             │   ───►  │      ANALYSIS       │   ───►  │     BATCHES     │
    │  Tx1, Tx2   │         │                     │         │                 │
    │  Tx3, Tx4   │         │  Tx1 ──┐            │         │  Batch 1: Tx1,3 │
    │  Tx5, Tx6   │         │        ├──► Tx4     │         │  Batch 2: Tx2   │
    │             │         │  Tx2 ──┘            │         │  Batch 3: Tx4,5 │
    └─────────────┘         │  Tx3 ───────► Tx5   │         └─────────────────┘
                            └─────────────────────┘

    {Colors.YELLOW}Step 2: Parallel Batch Execution{Colors.END}

    ┌───────────────────────────────────────────────────────────────────────┐
    │                                                                       │
    │   ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐           │
    │   │Thread 1 │    │Thread 2 │    │Thread 3 │    │Thread 4 │           │
    │   │         │    │         │    │         │    │         │           │
    │   │  {Colors.GREEN}Tx1{Colors.END}    │    │  {Colors.GREEN}Tx3{Colors.END}    │    │  {Colors.GREEN}Tx4{Colors.END}    │    │  {Colors.GREEN}Tx5{Colors.END}    │           │
    │   │         │    │         │    │         │    │         │           │
    │   └────┬────┘    └────┬────┘    └────┬────┘    └────┬────┘           │
    │        │              │              │              │                 │
    │        └──────────────┴──────────────┴──────────────┘                 │
    │                              │                                        │
    │                        ┌─────▼─────┐                                  │
    │                        │  OBJECT   │                                  │
    │                        │   STORE   │                                  │
    │                        └───────────┘                                  │
    └───────────────────────────────────────────────────────────────────────┘

    {Colors.GRAY}Key Insight: Transactions declare read/write sets, enabling automatic
    parallelization of non-conflicting operations.{Colors.END}
""")

def print_phase2_consensus():
    print(f"""
{Colors.YELLOW}{Colors.BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
                           PHASE 2: CONSENSUS LAYER
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{Colors.END}

{Colors.GREEN}  ┌─────────────────────────────────────────────────────────────────────────┐
  │                         BFT CONSENSUS ROUND                              │
  └─────────────────────────────────────────────────────────────────────────┘{Colors.END}

                              {Colors.BOLD}CONSENSUS FLOW{Colors.END}

    ┌──────────┐      ┌──────────┐      ┌──────────┐      ┌──────────┐
    │   {Colors.BLUE}IDLE{Colors.END}   │ ───► │ {Colors.YELLOW}PROPOSE{Colors.END}  │ ───► │   {Colors.CYAN}VOTE{Colors.END}   │ ───► │  {Colors.GREEN}COMMIT{Colors.END}  │
    │          │      │          │      │          │      │          │
    │ Wait for │      │ Leader   │      │Validators│      │ Quorum   │
    │ block    │      │ creates  │      │ vote on  │      │ reached  │
    │ time     │      │ block    │      │ proposal │      │ finalize │
    │ (500ms)  │      │          │      │          │      │          │
    └──────────┘      └──────────┘      └──────────┘      └────┬─────┘
                                                               │
                                                               ▼
                                                     ┌──────────────────┐
                                                     │     {Colors.GREEN}FINALITY{Colors.END}     │
                                                     │   Certificate    │
                                                     │   (2/3 + 1)      │
                                                     └──────────────────┘
""")

def print_phase2_block():
    print(f"""
{Colors.GREEN}  ┌─────────────────────────────────────────────────────────────────────────┐
  │                            BLOCK STRUCTURE                               │
  └─────────────────────────────────────────────────────────────────────────┘{Colors.END}

    ┌───────────────────────────────────────────────────────────────────────┐
    │ {Colors.BOLD}BLOCK #{Colors.CYAN}42{Colors.END}                                                          │
    ├───────────────────────────────────────────────────────────────────────┤
    │ {Colors.YELLOW}HEADER{Colors.END}                                                               │
    │ ┌───────────────────────────────────────────────────────────────────┐ │
    │ │  height: 42              │  timestamp: 1699900000               │ │
    │ │  prev_hash: 0x7a3f...    │  proposer: 0x8b2c...                 │ │
    │ │  tx_root: 0x4d1e...      │  state_root: 0x9f5a...               │ │
    │ │  offchain_root: 0x2b7... │  signature: 0xc4e9...                │ │
    │ └───────────────────────────────────────────────────────────────────┘ │
    ├───────────────────────────────────────────────────────────────────────┤
    │ {Colors.YELLOW}TRANSACTIONS{Colors.END}                                                         │
    │ ┌───────────────────────────────────────────────────────────────────┐ │
    │ │  [RegisterRobot, AcceptTask, SubmitCompletion, OpenPaymentStream] │ │
    │ └───────────────────────────────────────────────────────────────────┘ │
    ├───────────────────────────────────────────────────────────────────────┤
    │ {Colors.YELLOW}OFFCHAIN COMMITMENTS{Colors.END}                                                 │
    │ ┌───────────────────────────────────────────────────────────────────┐ │
    │ │  [TrajectoryHash: 0x..., EvidenceHash: 0x..., SensorHash: 0x...]  │ │
    │ └───────────────────────────────────────────────────────────────────┘ │
    └───────────────────────────────────────────────────────────────────────┘
""")

def print_phase2_validators():
    print(f"""
{Colors.GREEN}  ┌─────────────────────────────────────────────────────────────────────────┐
  │                            VALIDATOR SET                                 │
  └─────────────────────────────────────────────────────────────────────────┘{Colors.END}

    ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
    │ {Colors.CYAN}Validator 1{Colors.END}     │  │ {Colors.CYAN}Validator 2{Colors.END}     │  │ {Colors.CYAN}Validator 3{Colors.END}     │  │ {Colors.CYAN}Validator 4{Colors.END}     │
    ├─────────────────┤  ├─────────────────┤  ├─────────────────┤  ├─────────────────┤
    │ Stake: {Colors.GREEN}100K{Colors.END}    │  │ Stake: {Colors.GREEN}150K{Colors.END}    │  │ Stake: {Colors.GREEN}80K{Colors.END}     │  │ Stake: {Colors.GREEN}120K{Colors.END}    │
    │ Region: US      │  │ Region: EU      │  │ Region: APAC    │  │ Region: US      │
    │ Uptime: {Colors.GREEN}99.9%{Colors.END}  │  │ Uptime: {Colors.GREEN}99.8%{Colors.END}  │  │ Uptime: {Colors.GREEN}99.7%{Colors.END}  │  │ Uptime: {Colors.GREEN}99.9%{Colors.END}  │
    │ Blocks: 1,234   │  │ Blocks: 1,456   │  │ Blocks: 987     │  │ Blocks: 1,100   │
    └─────────────────┘  └─────────────────┘  └─────────────────┘  └─────────────────┘

    {Colors.YELLOW}Quorum Requirement:{Colors.END} 2/3 + 1 = {Colors.GREEN}3 validators{Colors.END} needed for consensus
""")

def print_phase2_offline():
    print(f"""
{Colors.GREEN}  ┌─────────────────────────────────────────────────────────────────────────┐
  │                     OFFLINE RECONCILIATION PROTOCOL                      │
  └─────────────────────────────────────────────────────────────────────────┘{Colors.END}

    {Colors.YELLOW}Robot Operating Offline:{Colors.END}

    ┌────────────────┐      ┌────────────────┐      ┌────────────────┐
    │  Local Task    │      │   Event Log    │      │    Signed      │
    │  Execution     │ ───► │   Recording    │ ───► │    Events      │
    │                │      │                │      │                │
    │ • Clean room   │      │ seq: 1, 2, 3.. │      │ [signature]    │
    │ • Navigate     │      │ timestamps     │      │ [signature]    │
    │ • Complete     │      │ event_types    │      │ [signature]    │
    └────────────────┘      └────────────────┘      └───────┬────────┘
                                                            │
                                                            ▼
    {Colors.YELLOW}On Reconnect:{Colors.END}                                     {Colors.CYAN}Offline Storage{Colors.END}
                                                            │
    ┌────────────────┐      ┌────────────────┐      ┌───────▼────────┐
    │    Create      │      │    Validate    │      │   Submit to    │
    │     Batch      │ ◄─── │   Signatures   │ ◄─── │     Chain      │
    │                │      │   & Sequence   │      │                │
    └───────┬────────┘      └────────────────┘      └────────────────┘
            │
            ▼
    ┌────────────────┐
    │  Reconcile &   │
    │  Settle Pmts   │
    │                │
    │ • Verify proofs│
    │ • Release funds│
    │ • Update state │
    └────────────────┘
""")

def print_dataflow():
    print(f"""
{Colors.YELLOW}{Colors.BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
                             END-TO-END DATA FLOW
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{Colors.END}

    {Colors.CYAN}┌─────────┐{Colors.END}      {Colors.CYAN}┌─────────────┐{Colors.END}      {Colors.CYAN}┌──────────────┐{Colors.END}      {Colors.CYAN}┌────────────┐{Colors.END}
    {Colors.CYAN}│{Colors.END}  User   {Colors.CYAN}│{Colors.END} ───► {Colors.CYAN}│{Colors.END} Transaction {Colors.CYAN}│{Colors.END} ───► {Colors.CYAN}│{Colors.END}   Mempool    {Colors.CYAN}│{Colors.END} ───► {Colors.CYAN}│{Colors.END}  Scheduler {Colors.CYAN}│{Colors.END}
    {Colors.CYAN}│{Colors.END} Client  {Colors.CYAN}│{Colors.END}      {Colors.CYAN}│{Colors.END}  Creation   {Colors.CYAN}│{Colors.END}      {Colors.CYAN}│{Colors.END}              {Colors.CYAN}│{Colors.END}      {Colors.CYAN}│{Colors.END}            {Colors.CYAN}│{Colors.END}
    {Colors.CYAN}└─────────┘{Colors.END}      {Colors.CYAN}└─────────────┘{Colors.END}      {Colors.CYAN}└──────────────┘{Colors.END}      {Colors.CYAN}└──────┬─────┘{Colors.END}
                                                                      │
                                                                      ▼
    {Colors.GREEN}┌─────────┐{Colors.END}      {Colors.GREEN}┌─────────────┐{Colors.END}      {Colors.GREEN}┌──────────────┐{Colors.END}      {Colors.GREEN}┌────────────┐{Colors.END}
    {Colors.GREEN}│{Colors.END}  State  {Colors.GREEN}│{Colors.END} ◄─── {Colors.GREEN}│{Colors.END}  Execution  {Colors.GREEN}│{Colors.END} ◄─── {Colors.GREEN}│{Colors.END}   Parallel   {Colors.GREEN}│{Colors.END} ◄─── {Colors.GREEN}│{Colors.END}   Batch    {Colors.GREEN}│{Colors.END}
    {Colors.GREEN}│{Colors.END}  Store  {Colors.GREEN}│{Colors.END}      {Colors.GREEN}│{Colors.END}   Results   {Colors.GREEN}│{Colors.END}      {Colors.GREEN}│{Colors.END}  Execution   {Colors.GREEN}│{Colors.END}      {Colors.GREEN}│{Colors.END}  Creation  {Colors.GREEN}│{Colors.END}
    {Colors.GREEN}└────┬────┘{Colors.END}      {Colors.GREEN}└─────────────┘{Colors.END}      {Colors.GREEN}└──────────────┘{Colors.END}      {Colors.GREEN}└────────────┘{Colors.END}
         │
         ▼
    {Colors.YELLOW}┌─────────┐{Colors.END}      {Colors.YELLOW}┌─────────────┐{Colors.END}      {Colors.YELLOW}┌──────────────┐{Colors.END}      {Colors.YELLOW}┌────────────┐{Colors.END}
    {Colors.YELLOW}│{Colors.END}  Block  {Colors.YELLOW}│{Colors.END} ───► {Colors.YELLOW}│{Colors.END}  Consensus  {Colors.YELLOW}│{Colors.END} ───► {Colors.YELLOW}│{Colors.END}   Finality   {Colors.YELLOW}│{Colors.END} ───► {Colors.YELLOW}│{Colors.END}  Receipts  {Colors.YELLOW}│{Colors.END}
    {Colors.YELLOW}│{Colors.END} Propose {Colors.YELLOW}│{Colors.END}      {Colors.YELLOW}│{Colors.END}   Voting    {Colors.YELLOW}│{Colors.END}      {Colors.YELLOW}│{Colors.END} Certificate  {Colors.YELLOW}│{Colors.END}      {Colors.YELLOW}│{Colors.END}  & Events  {Colors.YELLOW}│{Colors.END}
    {Colors.YELLOW}└─────────┘{Colors.END}      {Colors.YELLOW}└─────────────┘{Colors.END}      {Colors.YELLOW}└──────────────┘{Colors.END}      {Colors.YELLOW}└────────────┘{Colors.END}
""")

def print_demo_scenario():
    print(f"""
{Colors.YELLOW}{Colors.BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
                             DEMO: ROBOT TASK FLOW
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{Colors.END}

    {Colors.BOLD}Scenario: Alice's cleaning robot completes a task{Colors.END}

    {Colors.CYAN}Step 1: Registration{Colors.END}
    ┌──────────────────────────────────────────────────────────────────────┐
    │  {Colors.GREEN}►{Colors.END} Alice registers as Principal (home owner)                       │
    │  {Colors.GREEN}►{Colors.END} Alice registers her home as a Space                             │
    │  {Colors.GREEN}►{Colors.END} CleanBot-X1 registered by RobotCorp (with HW attestation)       │
    └──────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
    {Colors.CYAN}Step 2: Permission Grant{Colors.END}
    ┌──────────────────────────────────────────────────────────────────────┐
    │  {Colors.GREEN}►{Colors.END} Alice grants CLEANING + NAVIGATION capabilities to CleanBot     │
    │  {Colors.GREEN}►{Colors.END} Policy: Active Mon-Fri, 9am-5pm, Living Room + Kitchen only     │
    └──────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
    {Colors.CYAN}Step 3: Task Lifecycle{Colors.END}
    ┌──────────────────────────────────────────────────────────────────────┐
    │  {Colors.YELLOW}PostTask:{Colors.END} "Clean living room" - Reward: 1 ROUSD                   │
    │       │                                                              │
    │       ▼                                                              │
    │  {Colors.YELLOW}AcceptTask:{Colors.END} CleanBot accepts (has required capabilities)         │
    │       │                                                              │
    │       ▼                                                              │
    │  {Colors.YELLOW}StartTask:{Colors.END} Robot begins cleaning                                  │
    │       │                                                              │
    │       ▼                                                              │
    │  {Colors.GREEN}SubmitCompletion:{Colors.END} Evidence + Safety proofs uploaded               │
    └──────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
    {Colors.CYAN}Step 4: Payment Settlement{Colors.END}
    ┌──────────────────────────────────────────────────────────────────────┐
    │  {Colors.GREEN}►{Colors.END} Payment channel releases 1 ROUSD to CleanBot                    │
    │  {Colors.GREEN}►{Colors.END} Robot reputation score increased                                │
    │  {Colors.GREEN}►{Colors.END} Task marked as Completed on-chain                               │
    └──────────────────────────────────────────────────────────────────────┘
""")

def print_code_stats():
    """Count lines in each module"""
    base_path = os.path.dirname(os.path.abspath(__file__))
    src_path = os.path.join(base_path, 'src')

    stats = {}
    total_lines = 0

    modules = ['objects', 'transactions', 'execution', 'consensus', 'storage', 'crypto', 'network']

    for module in modules:
        module_path = os.path.join(src_path, module)
        if os.path.exists(module_path):
            lines = 0
            for file in os.listdir(module_path):
                if file.endswith('.rs'):
                    with open(os.path.join(module_path, file), 'r') as f:
                        lines += len(f.readlines())
            stats[module] = lines
            total_lines += lines

    # Count lib.rs and main.rs
    for file in ['lib.rs', 'main.rs']:
        filepath = os.path.join(src_path, file)
        if os.path.exists(filepath):
            with open(filepath, 'r') as f:
                lines = len(f.readlines())
                stats[file] = lines
                total_lines += lines

    print(f"""
{Colors.YELLOW}{Colors.BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
                              CODE STATISTICS
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{Colors.END}
""")

    # Create bar chart
    max_lines = max(stats.values()) if stats else 1
    bar_width = 40

    for module, lines in sorted(stats.items(), key=lambda x: -x[1]):
        bar_length = int((lines / max_lines) * bar_width)
        bar = '█' * bar_length + '░' * (bar_width - bar_length)
        print(f"    {module:15} {Colors.CYAN}{bar}{Colors.END} {lines:,} lines")

    print(f"\n    {Colors.BOLD}{'─' * 60}{Colors.END}")
    print(f"    {Colors.GREEN}TOTAL: {total_lines:,} lines of Rust code{Colors.END}")

def main():
    print_header()
    print_phase1_objects()
    print_phase1_transactions()
    print_phase1_execution()
    print_phase2_consensus()
    print_phase2_block()
    print_phase2_validators()
    print_phase2_offline()
    print_dataflow()
    print_demo_scenario()
    print_code_stats()

    print(f"""
{Colors.GREEN}{Colors.BOLD}
╔══════════════════════════════════════════════════════════════════════════════╗
║                          VISUALIZATION COMPLETE                              ║
║                                                                              ║
║  To build and run the project:                                               ║
║    $ cd /workspace/robochain                                                 ║
║    $ cargo build                                                             ║
║    $ cargo run                                                               ║
║                                                                              ║
║  Documentation:                                                              ║
║    • IMPLEMENTATION_PLAN.md - Full implementation roadmap                    ║
║    • ARCHITECTURE.md - ASCII architecture diagrams                           ║
╚══════════════════════════════════════════════════════════════════════════════╝
{Colors.END}
""")

if __name__ == "__main__":
    main()
