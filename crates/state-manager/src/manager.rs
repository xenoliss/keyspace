use keyspace_imt::tree::{Imt, ImtWriter};
use keyspace_indexer::StateUpdate;
use tiny_keccak::Keccak;
use tokio::sync::mpsc::Receiver;

use crate::storage::{
    btree::BTreeStorage, keys::vk_storage_key, StorageWriter, Transaction, TransactionalStorage,
};

#[derive(Debug)]
pub struct StateManager {
    indexer_stream: Receiver<StateUpdate>,
    storage: BTreeStorage,
}

impl StateManager {
    pub fn new(indexer_stream: Receiver<StateUpdate>) -> Self {
        Self {
            indexer_stream,
            storage: BTreeStorage::default(),
        }
    }
}

impl StateManager {
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
                    let mut tx = self.storage.transaction();

                    let mut imt = Imt::writer(Keccak::v256, &mut tx);

                    for forced_tx in offchain_txs {
                        imt.set_node(
                            forced_tx.originalKey.to_le_bytes(),
                            forced_tx.newKey.to_le_bytes_vec(),
                        );
                    }

                    tx.commit();
                }
            }
        }
    }
}
