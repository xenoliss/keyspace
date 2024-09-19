use tokio::sync::mpsc::Receiver;

#[derive(Debug)]
pub struct Transaction;

#[derive(Debug)]
pub struct TransactionPool {
    node_stream: Receiver<Transaction>,
    transactions: Vec<Transaction>,
}

impl TransactionPool {
    pub fn new(node_stream: Receiver<Transaction>) -> Self {
        Self {
            node_stream,
            transactions: vec![],
        }
    }

    pub async fn run(&mut self) {
        // Register the transaction received from the node.
        while let Some(tx) = self.node_stream.recv().await {
            self.transactions.push(tx);
        }
    }
}
