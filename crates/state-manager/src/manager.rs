use keyspace_imt::tree::{Imt, ImtWriter};
use keyspace_indexer::StateUpdate;
use tiny_keccak::Keccak;
use tokio::sync::mpsc::Receiver;

use crate::storage::{
    btree::BTreeStorage, keys::vk_storage_key, StorageWriter, Transaction, TransactionalStorage,
};

/// The state manager responsible for persiting the roolup state.
#[derive(Debug)]
pub struct StateManager {
    indexer_stream: Receiver<StateUpdate>,
    storage: BTreeStorage,
}

impl StateManager {
    /// Creates a new [StateManager].
    pub fn new(indexer_stream: Receiver<StateUpdate>) -> Self {
        Self {
            indexer_stream,
            // TODO: The storage should be injected in the constructor instead.
            storage: BTreeStorage::default(),
        }
    }

    /// Runs the [StateManager] to accept and apply state updates.
    pub async fn run(&mut self) {
        while let Some(state_update) = self.indexer_stream.recv().await {
            match state_update {
                StateUpdate::VerifyingKey(vk_registered) => {
                    self.storage.set(
                        vk_storage_key(vk_registered.vkHash.as_le_bytes()),
                        vk_registered.vk,
                    );
                }
                StateUpdate::Records(batch_proved) => {
                    let mut tx = self.storage.transaction();

                    let mut imt = Imt::writer(Keccak::v256, &mut tx);

                    // Process the "normal" transactions that were sent to the node mempool already.
                    for tx in batch_proved.txs {
                        // TODO: Verify proof?
                        imt.set_node(tx.keyspaceId.to_le_bytes(), tx.newKey.to_le_bytes_vec());
                    }

                    // TODO: Process the forced transactions that were sent to the L1 KeyStore contract.

                    tx.commit();
                }
            }
        }
    }
}
