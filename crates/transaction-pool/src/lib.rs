use anyhow::{anyhow, Result};
use std::collections::VecDeque;
use tokio::{
    select,
    sync::{mpsc::Receiver, oneshot},
};
use tracing::{debug, info, warn};

use message::{
    GetPendingTransactionsForSequencing, GetPendingTransactionsForSequencingResponse,
    PushPendingTransaction,
};
use transaction::{PendingTransaction, SequencedTransaction};
use transaction_verifier::TransactionVerifier;

pub mod message;
pub mod transaction;

mod transaction_verifier;

/// The [TransactionPool] manages the transactions and their lifecycle.
pub struct TransactionPool {
    rpc_to_tx_pool_stream: Receiver<PushPendingTransaction>,
    sequencer_to_tx_pool_stream: Receiver<GetPendingTransactionsForSequencing>,

    pending_txs: VecDeque<PendingTransaction>,
    sequenced_txs: Vec<SequencedTransaction>,

    tx_verifier: TransactionVerifier,
}

impl TransactionPool {
    /// Creates a new [TransactionPool].
    pub fn new(
        rpc_to_tx_pool_stream: Receiver<PushPendingTransaction>,
        sequencer_to_tx_pool_stream: Receiver<GetPendingTransactionsForSequencing>,
    ) -> Self {
        Self {
            rpc_to_tx_pool_stream,
            sequencer_to_tx_pool_stream,

            pending_txs: VecDeque::new(),
            sequenced_txs: vec![],

            tx_verifier: TransactionVerifier::new(),
        }
    }

    /// Runs the [TransactionPool] to listen for incoming.
    pub async fn run(mut self) -> Result<()> {
        info!("Transaction pool started");

        loop {
            select! {
                Some(push_pending_transaction) = self.rpc_to_tx_pool_stream.recv() => {
                    self.handle_push_pending_transaction_message(push_pending_transaction)?
                }

                Some(request_pending_transaction) = self.sequencer_to_tx_pool_stream.recv() => {
                    self.handle_request_pending_transaction_message(request_pending_transaction).await?
                }
            }
        }
    }

    /// Handle the [GetPendingTransactionsForSequencing] messages and submits back a corresponding [GetPendingTransactionsForSequencingResponse].
    async fn handle_request_pending_transaction_message(
        &mut self,
        msg: GetPendingTransactionsForSequencing,
    ) -> Result<()> {
        debug!("Processing GetPendingTransactionsForSequencing message");

        let GetPendingTransactionsForSequencing {
            max_count,
            res_sink,
        } = msg;

        let count = if let Some(max_count) = max_count {
            std::cmp::min(max_count, self.pending_txs.len())
        } else {
            self.pending_txs.len()
        };

        let txs = self.pending_txs.iter().take(count).cloned().collect();
        let (sequencer_sink, sequencer_stream) = oneshot::channel();

        debug!("Sending pending transactions to sequencer");
        res_sink
            .send(GetPendingTransactionsForSequencingResponse {
                txs,
                res_sink: sequencer_sink,
            })
            .map_err(|_| anyhow!("failed to send response to Sequencer"))?;

        debug!("Waiting for sequencer ack");
        let sequencer_ack = sequencer_stream
            .await
            .map_err(|why| anyhow!("failed to await Sequencer ack: {why:?}"))?;

        // If we got an ACK from the Sequencer, we can confidently move the transactions
        // from the pending list to the sequenced list.

        match sequencer_ack {
            Ok(_) => {
                debug!("Marking transactions as sequenced");
                self.sequenced_txs
                    .extend(self.pending_txs.drain(..count).map(|tx| tx.sequenced()));
            }
            Err(_) => {
                warn!("Keeping transactions as pending");
            }
        };

        Ok(())
    }

    fn handle_push_pending_transaction_message(
        &mut self,
        msg: PushPendingTransaction,
    ) -> Result<()> {
        debug!("Processing PushPendingTransaction message");

        let PushPendingTransaction { tx, res_sink } = msg;

        // TODO: Verifying at instant T might success here but the transaction might
        //       fail when actually submitted in a batch proof (for instance the same user
        //       sending 2 incompatible transactions that modify the same KeySpace record).
        match self.tx_verifier.verify_tx(&tx) {
            Ok(_) => {
                self.pending_txs.push_back(tx);
                debug!("Transaction pushed to mempool");

                res_sink
                    .send(Ok(()))
                    .map_err(|why| anyhow!("failed to ack success to Sequencer: {why:?}"))
            }
            Err(_) => {
                warn!("Transaction verification failed");

                res_sink
                    .send(Err(anyhow!("transaction verification failed")))
                    .map_err(|why| anyhow!("failed to acrk error to the Sequencer: {why:?}"))
            }
        }
    }
}
