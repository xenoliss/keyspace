use anyhow::{anyhow, Result};
use std::time::Duration;
use tokio::{
    sync::{mpsc::Sender, oneshot},
    time::interval,
};
use tracing::info;

use keyspace_transaction_pool::{
    message::{GetPendingTransactionsForSequencing, GetPendingTransactionsForSequencingResponse},
    transaction::PendingTransaction,
};

/// The [Sequencer] periodically queries pending transactions form the [keyspace_transaction_pool::TransactionPool]
/// and forward them to the [keyspace_batcher::Batcher] to be submitted onchain.
pub struct Sequencer {
    sequencer_to_tx_pool_sink: Sender<GetPendingTransactionsForSequencing>,
    batcher_sink: Sender<Vec<PendingTransaction>>,
}

impl Sequencer {
    /// Creates a new [Sequencer].
    pub fn new(
        sequencer_to_tx_pool_sink: Sender<GetPendingTransactionsForSequencing>,
        batcher_sink: Sender<Vec<PendingTransaction>>,
    ) -> Self {
        Self {
            sequencer_to_tx_pool_sink,
            batcher_sink,
        }
    }

    /// Runs the [Sequencer].
    pub async fn run(self) -> Result<()> {
        info!("Sequencer started");

        let mut interval = interval(Duration::from_secs(15));

        loop {
            let (res_sink, res_stream) = oneshot::channel();

            self.sequencer_to_tx_pool_sink
                .send(GetPendingTransactionsForSequencing {
                    max_count: Some(10),
                    res_sink,
                })
                .await
                .map_err(|why| anyhow!("failed to query for the pending transactions: {why:?}"))?;

            let res = res_stream
                .await
                .map_err(|why| anyhow!("failed to get the pending transactions: {why:?}"))?;

            let GetPendingTransactionsForSequencingResponse { txs, res_sink } = res;

            // TODO: Maybe update the local state and or perform additional checks on the transactions.

            // Send ack back to the TransactionPool.
            res_sink
                .send(Ok(()))
                .map_err(|why| anyhow!("failed to get ack from TransactionPool: {why:?}"))?;

            // Forward the pending transactions to the Batcher fot L1 submission.
            if !txs.is_empty() {
                self.batcher_sink.send(txs).await.map_err(|why| {
                    anyhow!("failed to send the pending transactions to the Batcher: {why:?}")
                })?;
            }

            interval.tick().await;
        }
    }
}
