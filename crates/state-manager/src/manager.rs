use anyhow::Result;
use std::{collections::VecDeque, fmt::Debug};
use tiny_keccak::Keccak;
use tokio::sync::mpsc::Receiver;
use tracing::{debug, info};

use crate::storage::{Transaction, TransactionalStorage};
use keyspace_imt::{storage::ImtStorageWriter, tree::Imt};
use keyspace_keystore_bindings::bindings::KeyStore::{BatchProved, ForcedTransactionSubmitted};

/// This enum defines the different messages that the [StateManager] listen for.
pub enum StateManagerMsg {
    /// Wrapper around the [BatchProved] emitted by the L1 KeyStore contract.
    BatchProved(BatchProved),
    /// Wrapper around the [ForcedTransactionSubmitted] emitted by the L1 KeyStore contract.
    ForcedTransactionSubmitted(ForcedTransactionSubmitted),
}

/// The state manager responsible for persiting the roolup state.
#[derive(Debug)]
pub struct StateManager<Storage> {
    /// The underlying storage layer that stores the rollup state.
    storage: Storage,

    /// The stream of [StateManagerMsg], feeded by the indexer, to process.
    indexer_stream: Receiver<StateManagerMsg>,

    /// The pending list of forced transactions waiting to be proved.
    /// NOTE: Those transactions are not managed via some mempool mechanism as
    ///       they are always sent to the L1 KeyStore contract directly and are
    ///       only temporarly needed by the [StateManager] to rebuild the imt.
    pending_forced_transactions: VecDeque<ForcedTransactionSubmitted>,
}

impl<S> StateManager<S> {
    /// Creates a new [StateManager].
    pub fn new(storage: S, indexer_stream: Receiver<StateManagerMsg>) -> Self {
        Self {
            storage,
            indexer_stream,
            pending_forced_transactions: VecDeque::new(),
        }
    }
}

impl<S> StateManager<S>
where
    S: TransactionalStorage,
    for<'a> S::T<'a>: ImtStorageWriter<NodeK = [u8; 32], NodeV = [u8; 32]>,
{
    /// Runs the [StateManager] to listen for [StateManagerMsg] from the indexer and rebuild the imt state.
    pub async fn run(mut self) -> Result<()> {
        info!("StateManager started");

        while let Some(msg) = self.indexer_stream.recv().await {
            match msg {
                StateManagerMsg::BatchProved(batch_proved) => {
                    self.handle_batch_proved(batch_proved).await?;
                }
                StateManagerMsg::ForcedTransactionSubmitted(forced_tx_submitted) => {
                    self.handle_forced_tx_submitted(forced_tx_submitted);
                }
            }
        }

        Ok(())
    }

    /// Update the imt state based on the forced transactions (if any) and the sequenced transactions that are
    /// included in the provided [BatchProved].
    ///
    /// For each forced transaction, its proof MUST be re-verified before updating the imt as they are allowed
    /// to be invalid. The imt is only updated when the proof verification passes.
    ///
    /// For sequenced transactions, there is no need to re-verify the proofs as the Batcher program already enforces
    /// the proof validity.
    async fn handle_batch_proved(&mut self, batch_proved: BatchProved) -> Result<()> {
        debug!(event = "BatchProved", "Processing event");

        let mut tx = self.storage.transaction();
        let mut imt = Imt::writer(Keccak::v256, &mut tx);

        let forced_tx_count: usize = batch_proved.forcedTxCount.to();
        for _ in 0..forced_tx_count {
            let forced_tx = self
                .pending_forced_transactions
                .pop_front()
                .expect("inconsitent forced txs state");

            debug!(
                keyspace_id = forced_tx.keySpaceId.to_string(),
                new_value = forced_tx.newValue.to_string(),
                "Applying forced transaction"
            );

            // TODO: Verify the proof.
            let is_valid = true;
            if is_valid {
                imt.set_node(forced_tx.keySpaceId.into(), forced_tx.newValue.into())?;
            }
        }

        // Process the sequenced transactions that were sent to the node mempool already.
        for tx in batch_proved.sequencedTxs {
            debug!(
                keyspace_id = tx.keySpaceId.to_string(),
                new_value = tx.newValue.to_string(),
                "Applying sequenced transaction"
            );

            // NOTE: For sequenced transactions there is no need to verify them again here before
            //       updating the imt state as sequenced transactions MUST be valid for the Batcher
            //       proof to verify correctly in the L1 KeyStore contract.
            imt.set_node(tx.keySpaceId.into(), tx.newValue.into())?;
        }

        debug!(
            size = imt.size(),
            depth = imt.depth(),
            root = format!("{:?}", imt.root()),
            "Imt updated dimensions"
        );

        tx.commit();

        Ok(())
    }

    /// Push the received [ForcedTransactionSubmitted] to the [Self::pending_forced_transactions] queue.
    fn handle_forced_tx_submitted(&mut self, forced_tx_submitted: ForcedTransactionSubmitted) {
        debug!(event = "ForcedTransactionSubmitted", "Processing event");

        self.pending_forced_transactions
            .push_back(forced_tx_submitted);
    }
}
