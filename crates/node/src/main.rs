use alloy::primitives::address;
use anyhow::{anyhow, Result};
use futures::TryFutureExt;
use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;

use keyspace_indexer::Indexer;
use keyspace_sequencer::Sequencer;
use keyspace_state_manager::{manager::StateManager, storage::btree::BTreeStorage};
use keyspace_transaction_pool::TransactionPool;

#[tokio::main]
async fn main() -> Result<()> {
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
    let (indexer_to_state_manager_sink, indexer_to_state_manager_stream) = mpsc::channel(1000);
    let (sequencer_to_tx_pool_sink, sequencer_to_tx_pool_stream) = mpsc::channel(1000);
    let (rpc_to_tx_pool_sink, rpc_to_tx_pool_stream) = mpsc::channel(1000);
    let (sequencer_to_batcher_sink, sequencer_to_batcher_stream) = mpsc::channel(1000);

    // Instanciate the indexer.
    let indexer = Indexer::new(
        "http://127.0.0.1:8545",
        0,
        10,
        address!("5FbDB2315678afecb367f032d93F642f64180aa3"),
        indexer_to_state_manager_sink,
    )?;

    // Instanciate the StateManager.
    let storage = BTreeStorage::default();
    let state_manager = StateManager::new(storage, indexer_to_state_manager_stream);

    // Instanciate the Sequencer.
    let sequencer = Sequencer::new(sequencer_to_tx_pool_sink, sequencer_to_batcher_sink);

    // Instanciate the TransactionPool.
    let tx_pool = TransactionPool::new(rpc_to_tx_pool_stream, sequencer_to_tx_pool_stream);

    // Start the services.
    tokio::spawn(
        state_manager
            .run()
            .map_err(|why| anyhow!("StateManager errored: {}", why)),
    );

    tokio::spawn(
        sequencer
            .run()
            .map_err(|why| anyhow!("Sequencer errored: {}", why)),
    );

    tokio::spawn(
        tx_pool
            .run()
            .map_err(|why| anyhow!("TransactionPool errored: {}", why)),
    );

    indexer
        .run()
        .await
        .map_err(|why| anyhow!("Indexer errored: {}", why))
}
