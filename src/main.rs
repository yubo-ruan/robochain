//! RoboChain Node
//!
//! A specialized L1 blockchain node for home robot coordination.

use robochain::crypto::KeyPair;
use robochain::execution::{ExecutionConfig, ExecutionEngine};
use robochain::explorer::{ExplorerService, create_router};
use robochain::objects::*;
use robochain::transactions::{Transaction, TransactionPayload, RegisterRobotTx, RegisterSpaceTx, PostTaskTx};
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    RoboChain Node v0.1.0                     ║");
    println!("║     A Specialized L1 Blockchain for Home Robot Coordination  ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // Initialize execution engine
    let exec_config = ExecutionConfig {
        max_batch_size: 1000,
        num_threads: 4,
        max_block_gas: 100_000_000,
    };

    let execution = Arc::new(RwLock::new(ExecutionEngine::new(exec_config)));
    println!("✓ Execution engine initialized");

    // Run demo scenario
    run_demo(&execution);

    // Start explorer server
    let explorer = Arc::new(ExplorerService::new(execution.clone()));
    let app = create_router(explorer);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("\n✓ Block Explorer running at http://localhost:3000");
    println!("  Press Ctrl+C to stop\n");

    axum::serve(listener, app).await.unwrap();
}

fn run_demo(execution: &Arc<RwLock<ExecutionEngine>>) {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("                        Demo Scenario");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut exec = execution.write();

    // 1. Register a principal (home owner)
    let owner_keypair = KeyPair::generate();
    let register_owner = create_register_principal_tx("Alice (Home Owner)", &owner_keypair);
    let receipt = exec.execute_transaction(register_owner);
    println!("1. Registered Principal: Alice (Home Owner)");
    println!("   Status: {}", if receipt.success { "✓ Success" } else { "✗ Failed" });
    println!("   Gas used: {}", receipt.gas_used);

    // 2. Register a space (home)
    let register_space = create_register_space_tx("Alice's Smart Home", &owner_keypair);
    let receipt = exec.execute_transaction(register_space);
    println!("\n2. Registered Space: Alice's Smart Home");
    println!("   Status: {}", if receipt.success { "✓ Success" } else { "✗ Failed" });
    println!("   Gas used: {}", receipt.gas_used);

    // 3. Register a robot
    let robot_keypair = KeyPair::generate();
    let register_robot = create_register_robot_tx("CleanBot-X1", "RobotCorp", &robot_keypair);
    let receipt = exec.execute_transaction(register_robot);
    println!("\n3. Registered Robot: CleanBot-X1 by RobotCorp");
    println!("   Status: {}", if receipt.success { "✓ Success" } else { "✗ Failed" });
    println!("   Gas used: {}", receipt.gas_used);

    // 4. Post a task
    let space_id = SpaceId::from_bytes(b"Alice's Smart HomePrincipalId"); // Simplified
    let post_task = create_post_task_tx(
        space_id,
        "Clean the living room and vacuum carpets",
        vec![Capability::Cleaning, Capability::Navigation],
        1_000_000, // 1 ROUSD reward
        &owner_keypair,
    );
    let receipt = exec.execute_transaction(post_task);
    println!("\n4. Posted Task: Clean the living room");
    println!("   Reward: 1 ROUSD");
    println!("   Status: {}", if receipt.success { "✓ Success" } else { "✗ Failed" });
    println!("   Gas used: {}", receipt.gas_used);

    // Print summary
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("                        Execution Summary");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("Blocks executed: {}", exec.current_block());
    println!("Objects in store: {}", exec.object_count());

    let stats = exec.stats();
    println!("Batches processed: {}", stats.batches);
    println!("TPS: {:.2}", stats.tps);

    println!("\n✓ Demo completed successfully!");
}

fn create_register_principal_tx(name: &str, keypair: &KeyPair) -> Transaction {
    let payload = TransactionPayload::RegisterPrincipal(robochain::transactions::RegisterPrincipalTx {
        display_name: name.to_string(),
        principal_type: PrincipalType::Owner,
    });

    let mut tx = Transaction::new(payload, keypair.public_key(), timestamp(), 1);
    tx.sign(keypair);
    tx
}

fn create_register_space_tx(name: &str, keypair: &KeyPair) -> Transaction {
    let payload = TransactionPayload::RegisterSpace(RegisterSpaceTx {
        name: name.to_string(),
        space_type: SpaceType::Residential,
        geo_region: GeoRegion {
            country: "US".to_string(),
            region: Some("CA".to_string()),
            city: Some("San Francisco".to_string()),
            approx_coords: Some((37.7749, -122.4194)),
        },
    });

    let mut tx = Transaction::new(payload, keypair.public_key(), timestamp(), 2);
    tx.sign(keypair);
    tx
}

fn create_register_robot_tx(model: &str, manufacturer: &str, keypair: &KeyPair) -> Transaction {
    let attestation = HardwareAttestation {
        secure_element_pubkey: keypair.public_key(),
        manufacturer_signature: keypair.sign(b"attestation"),
        cert_chain_hash: robochain::crypto::Hash::from_bytes(b"cert_chain"),
        attested_at: timestamp(),
    };

    let payload = TransactionPayload::RegisterRobot(RegisterRobotTx {
        manufacturer: manufacturer.to_string(),
        model: model.to_string(),
        hw_attestation: attestation,
        capabilities: vec![
            Capability::Cleaning,
            Capability::Navigation,
            Capability::Manipulation,
        ],
    });

    let mut tx = Transaction::new(payload, keypair.public_key(), timestamp(), 1);
    tx.sign(keypair);
    tx
}

fn create_post_task_tx(
    space_id: SpaceId,
    description: &str,
    capabilities: Vec<Capability>,
    reward: u64,
    keypair: &KeyPair,
) -> Transaction {
    let payload = TransactionPayload::PostTask(PostTaskTx {
        space_id,
        description: description.to_string(),
        required_capabilities: capabilities,
        reward,
        deadline: timestamp() + 3600, // 1 hour from now
        safety_constraints: vec![],
    });

    let mut tx = Transaction::new(payload, keypair.public_key(), timestamp(), 3);
    tx.sign(keypair);
    tx
}

fn timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
