use keyspace_indexer::StateUpdate;
use tokio::sync::mpsc::Receiver;

use crate::{keys::vk_storage_key, storage::StorageWriter};

#[derive(Debug)]
pub struct StateManager<S> {
    indexer_stream: Receiver<StateUpdate>,
    storage: S,
}

impl<S> StateManager<S> {
    pub fn new(indexer_stream: Receiver<StateUpdate>, storage: S) -> Self {
        Self {
            indexer_stream,
            storage,
        }
    }
}

impl<S> StateManager<S>
where
    S: StorageWriter,
{
    pub async fn run(&mut self) {
        while let Some(state_update) = self.indexer_stream.recv().await {
            match state_update {
                StateUpdate::VerifyingKey { hash, vk } => {
                    self.storage.set(vk_storage_key(hash.as_le_bytes()), vk);
                }
                StateUpdate::RecordUpdate {
                    tx_hash,
                    root,
                    onchain_tx_count,
                    offchain_txs,
                } => {
                    todo!("Apply the update the imt")
                }
            }
        }
    }
}
