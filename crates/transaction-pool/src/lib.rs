use tokio::sync::mpsc::Receiver;

use keyspace_indexer::Transaction;

pub struct TransactionPool {
    indexer_stream: Receiver<Transaction>,
    transactions: Vec<Transaction>,
}

impl TransactionPool {
    pub fn new(indexer_stream: Receiver<Transaction>) -> Self {
        Self {
            indexer_stream,
            transactions: vec![],
        }
    }

    pub async fn run(&mut self) {
        while let Some(tx) = self.indexer_stream.recv().await {
            // TODO: Add sync primitive.
            self.transactions.push(tx);
            println!("Transaction added to pool");
        }
    }
}
