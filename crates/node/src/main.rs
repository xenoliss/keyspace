use alloy::primitives::address;
use tokio::sync::mpsc;

use keyspace_indexer::Indexer;
use keyspace_state_manager::{manager::StateManager, storage::btree::BTreeStorage};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // Configure logging.
    let env_filter = EnvFilter::new("info,keyspace=trace");
    tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_env_filter(env_filter)
        .init();

    // Create the indexer channel.
    let (indexer_sync, indexer_stream) = mpsc::channel(1000);

    // Instanciate the indexer.
    let indexer = Indexer::new(
        "http://127.0.0.1:8545",
        0,
        10,
        address!("5FbDB2315678afecb367f032d93F642f64180aa3"),
        indexer_sync,
    )
    .expect("failed to create the indexer");

    // Instanciate the state manager.
    let storage = BTreeStorage::default();
    let state_manager = StateManager::new(storage, indexer_stream);

    // Start the services.
    tokio::spawn(state_manager.run());
    indexer.run().await.expect("indexer service failed");
}
