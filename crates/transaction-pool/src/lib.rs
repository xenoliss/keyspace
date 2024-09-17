use keyspace_indexer::ForcedTx;
use tokio::{select, sync::mpsc::Receiver};

#[derive(Debug)]
pub struct TransactionPool {
    indexer_stream: Receiver<ForcedTx>,
    forced_transactions: Vec<ForcedTx>,

    node_stream: Receiver<Transaction>,
    transactions: Vec<Transaction>,
}

#[derive(Debug)]
pub struct Transaction;

impl TransactionPool {
    pub fn new(indexer_stream: Receiver<ForcedTx>, node_stream: Receiver<Transaction>) -> Self {
        Self {
            indexer_stream,
            forced_transactions: vec![],
            node_stream,
            transactions: vec![],
        }
    }

    pub async fn run(&mut self) {
        loop {
            select! {
                // Register the forced transactions received from the indexer.
                Some(forced_tx) = self.indexer_stream.recv() => {
                    self.forced_transactions.push(forced_tx);
                }

                // Register the transactions received from the node.
                Some(tx) = self.node_stream.recv() => {
                    self.transactions.push(tx);
                }
            }
        }
    }
}
