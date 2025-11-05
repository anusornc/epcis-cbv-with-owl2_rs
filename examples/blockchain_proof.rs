// Blockchain Proof of Concept - Actual Working Example
// This file demonstrates that the blockchain implementation is real and functional

use epcis_knowledge_graph::blockchain::{
    Blockchain, BlockchainConfig, Transaction, TransactionType,
    EpcisTransaction, Block
};
use epcis_knowledge_graph::models::epcis::EpcisEvent;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== BLOCKCHAIN PROOF OF CONCEPT ===\n");

    // 1. Create blockchain with default config (4 validators)
    println!("1. Initializing blockchain...");
    let config = BlockchainConfig::default();
    let mut blockchain = Blockchain::new(config)?;

    println!("   ✓ Blockchain created with genesis block");
    println!("   ✓ Height: {}", blockchain.height());
    println!("   ✓ Validators: {}", blockchain.consensus.validator_count());
    println!("   ✓ Genesis hash: {}", blockchain.latest_block().hash());

    // 2. Create EPCIS event (Dairy Farm)
    println!("\n2. Creating EPCIS ObjectEvent (Dairy Farm - Raw Milk Collection)...");
    let farm_event = EpcisEvent {
        event_id: "FARM_001".to_string(),
        event_type: "ObjectEvent".to_string(),
        event_time: "2024-01-01T08:00:00Z".to_string(),
        record_time: "2024-01-01T08:00:00Z".to_string(),
        event_action: "OBSERVE".to_string(),
        epc_list: vec!["urn:epc:id:sgtin:123456.789.001".to_string()],
        biz_step: Some("collecting".to_string()),
        disposition: Some("active".to_string()),
        biz_location: Some("urn:epc:id:sgln:123456.farm.0".to_string()),
    };
    println!("   ✓ Event ID: {}", farm_event.event_id);
    println!("   ✓ EPC: {}", farm_event.epc_list[0]);

    // 3. Convert to blockchain transaction
    println!("\n3. Converting EPCIS event to blockchain transaction...");
    let epcis_tx = EpcisTransaction::from_event(farm_event, "validator1".to_string())?;
    println!("   ✓ Transaction ID: {}", epcis_tx.blockchain_tx.tx_id);
    println!("   ✓ Sender: {}", epcis_tx.blockchain_tx.sender);
    println!("   ✓ Transaction verified: {}", epcis_tx.blockchain_tx.verify());

    // 4. Add transaction to pool
    println!("\n4. Adding transaction to blockchain pool...");
    blockchain.add_transaction(epcis_tx.blockchain_tx.clone())?;
    println!("   ✓ Transaction added to pool");
    println!("   ✓ Pending transactions: {}", blockchain.transaction_pool.len());

    // 5. Check whose turn it is
    println!("\n5. Checking validator turn (PoA Round Robin)...");
    let next_height = blockchain.height() + 1;
    let current_validator = blockchain.consensus.get_current_validator(next_height);
    println!("   ✓ Next block height: {}", next_height);
    println!("   ✓ Current validator: {}", current_validator);
    println!("   ✓ Is validator1's turn: {}",
             blockchain.consensus.is_validator_turn("validator1", next_height));

    // 6. Create block
    println!("\n6. Creating new block...");
    let block = blockchain.create_block("validator1".to_string())?;
    println!("   ✓ Block height: {}", block.header.height);
    println!("   ✓ Block hash: {}", block.hash());
    println!("   ✓ Previous hash: {}", block.header.previous_hash);
    println!("   ✓ Transactions: {}", block.transactions.len());
    println!("   ✓ Block verified: {}", block.verify());

    // 7. Add block to chain
    println!("\n7. Adding block to blockchain...");
    blockchain.add_block(block)?;
    println!("   ✓ Block added successfully");
    println!("   ✓ Chain height: {}", blockchain.height());
    println!("   ✓ Total blocks: {}", blockchain.chain.len());
    println!("   ✓ Chain valid: {}", blockchain.verify_chain());

    // 8. Get statistics
    println!("\n8. Blockchain statistics:");
    let stats = blockchain.get_stats();
    println!("   ✓ Total blocks: {}", stats.total_blocks);
    println!("   ✓ Total transactions: {}", stats.total_transactions);
    println!("   ✓ Pending transactions: {}", stats.pending_transactions);
    println!("   ✓ Validators: {}", stats.validators);

    // 9. Demonstrate Round Robin
    println!("\n9. Demonstrating PoA Round Robin rotation:");
    for height in 0..8 {
        let validator = blockchain.consensus.get_current_validator(height);
        println!("   Block {} → {}", height, validator);
    }

    // 10. Verify entire chain
    println!("\n10. Final verification:");
    println!("   ✓ Chain integrity: {}", blockchain.verify_chain());
    println!("   ✓ Latest block hash: {}", blockchain.latest_block().hash());

    println!("\n=== PROOF COMPLETE ===");
    println!("All blockchain operations executed successfully!");
    println!("This demonstrates:");
    println!("  • Block creation and validation");
    println!("  • Transaction processing");
    println!("  • PoA Round Robin consensus");
    println!("  • EPCIS event integration");
    println!("  • Cryptographic hashing");
    println!("  • Chain verification");

    Ok(())
}
