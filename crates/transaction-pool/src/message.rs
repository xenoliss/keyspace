use anyhow::Result;
use tokio::sync::oneshot;

use crate::transaction::{PendingTransaction, SequencedTransaction};

/// Request the [crate::TransactionPool] to adds the provided [PendingTransaction].
pub struct PushPendingTransaction {
    pub tx: PendingTransaction,
    pub res_sink: oneshot::Sender<Result<()>>,
}

// TODO: Add some sort of filtering or ordering so that the Sequencer can more precisely query
//       for transactions.
/// Request the [crate::TransactionPool] for the current list of [PendingTransaction].
pub struct GetPendingTransactionsForSequencing {
    pub max_count: Option<usize>,
    pub res_sink: oneshot::Sender<GetPendingTransactionsForSequencingResponse>,
}

// TODO: The response should be more complete with the pisibility to tell exacly which transaction have been selected.
/// Message sent by the [crate::TransactionPool] in response for a [GetPendingTransactionsForSequencing] message.
pub struct GetPendingTransactionsForSequencingResponse {
    pub txs: Vec<PendingTransaction>,
    pub res_sink: oneshot::Sender<Result<()>>,
}

/// Request the [crate::TransactionPool] to mark the provided [SequencedTransaction] as finalized,
/// effectively removing them from the pool.
pub struct FinalizeTransactions {
    pub txs: Vec<SequencedTransaction>,
    pub res_sink: oneshot::Sender<Result<()>>,
}
