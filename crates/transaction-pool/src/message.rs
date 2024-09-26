use anyhow::Result;
use tokio::sync::oneshot;

use crate::transaction::{PendingTransaction, SequencedTransaction};

/// Request the [crate::TransactionPool] to adds the provided [PendingTransaction].
pub struct PushPendingTransaction {
    pub tx: PendingTransaction,
    pub res_sink: oneshot::Sender<Result<()>>,
}

/// Request the [crate::TransactionPool] for the current list of [PendingTransaction].
pub struct GetPendingTransactions {
    pub max_count: Option<usize>,
    pub res_sink: oneshot::Sender<GetPendingTransactionsResponse>,
}

/// Message sent by the [crate::TransactionPool] in response for a [GetPendingTransactions] message.
pub struct GetPendingTransactionsResponse {
    pub txs: Vec<PendingTransaction>,
    pub res_sink: oneshot::Sender<Result<()>>,
}

/// Request the [crate::TransactionPool] to mark the provided [SequencedTransaction] as finalized,
/// effectively removing them from the pool.
pub struct FinalizeTransactions {
    pub txs: Vec<SequencedTransaction>,
    pub res_sink: oneshot::Sender<Result<()>>,
}
