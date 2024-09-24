use serde::{Deserialize, Serialize};

use crate::Hash256;

use super::transaction::Transaction;

#[derive(Debug, Deserialize, Serialize)]
pub struct Inputs {
    /// Public input: the Keyspace root to start from.
    pub old_root: Hash256,
    /// Public input: the expected Keyspace root after applying the list of transactions.
    pub new_root: Hash256,
    /// Public input: a commitment to the list of transactions processed.
    pub txs_commitment: Hash256,

    /// Private input: the list of transactions to process.
    pub txs: Vec<Transaction>,
}
