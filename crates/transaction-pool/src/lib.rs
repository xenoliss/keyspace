use std::collections::VecDeque;

use anyhow::anyhow;
use tokio::{
    select,
    sync::{mpsc::Receiver, oneshot},
};
use tracing::{debug, info, warn};

use message::{GetPendingTransactions, GetPendingTransactionsResponse, PushPendingTransaction};
use transaction::{PendingTransaction, SequencedTransaction};
use transaction_verifier::TransactionVerifier;

pub mod message;
pub mod transaction;

mod transaction_verifier;

/// The [TransactionPool] manages the transactions and their lifecycle.
pub struct TransactionPool {
    node_stream: Receiver<PushPendingTransaction>,
    indexer_stream: Receiver<GetPendingTransactions>,

    pending_txs: VecDeque<PendingTransaction>,
    sequenced_txs: Vec<SequencedTransaction>,

    tx_verifier: TransactionVerifier,
}

impl TransactionPool {
    /// Creates a new [TransactionPool].
    pub fn new(
        node_stream: Receiver<PushPendingTransaction>,
        indexer_stream: Receiver<GetPendingTransactions>,
    ) -> Self {
        Self {
            node_stream,
            indexer_stream,

            pending_txs: VecDeque::new(),
            sequenced_txs: vec![],

            tx_verifier: TransactionVerifier::new(),
        }
    }

    /// Runs the [TransactionPool] to listen for incoming.
    pub async fn run(&mut self) {
        info!("Transaction pool started");

        loop {
            select! {
                Some(push_pending_transaction) = self.node_stream.recv() => {
                    self.handle_push_pending_transaction_message(push_pending_transaction)
                }

                Some(request_pending_transaction) = self.indexer_stream.recv() => {
                    self.handle_request_pending_transaction_message(request_pending_transaction).await
                }
            }
        }
    }

    async fn handle_request_pending_transaction_message(&mut self, msg: GetPendingTransactions) {
        debug!("Processing GetPendingTransactions message");

        let GetPendingTransactions {
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

        // TODO: Better error handling.
        debug!("Sending pending transactions to sequencer");
        let _ = res_sink.send(GetPendingTransactionsResponse {
            txs,
            res_sink: sequencer_sink,
        });

        // TODO: Better error handling.
        debug!("Waiting for sequencer ack");
        let sequencer_ack = sequencer_stream
            .await
            .expect("failed to await sequencer ack");

        match sequencer_ack {
            Ok(_) => {
                debug!("Marking transactions as sequenced");
                self.sequenced_txs
                    .extend(self.pending_txs.drain(..count).map(|tx| tx.sequenced()));
            }
            Err(_) => {
                warn!("Keeping transactions as pending");
            }
        }
    }

    fn handle_push_pending_transaction_message(&mut self, msg: PushPendingTransaction) {
        debug!("Processing PushPendingTransaction message");

        let PushPendingTransaction { tx, res_sink: res } = msg;

        // TODO: Verifying at instant T might success here but the transaction might
        //       fail when actually submitted in a batch proof (for instance the same user
        //       sending 2 incompatible transactions that modify the same KeySpace record).
        match self.tx_verifier.verify_tx(&tx) {
            Ok(_) => {
                self.pending_txs.push_back(tx);
                debug!("Transaction pushed to mempool");

                // TODO: Better error handling.
                res.send(Ok(())).expect("failed to respond");
            }
            Err(err) => {
                warn!("Transaction verification failed");

                // TODO: Better error handling.
                res.send(Err(anyhow!("transaction verification failed")))
                    .expect("failed to respond");
            }
        }
    }
}
