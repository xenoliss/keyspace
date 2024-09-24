use anyhow::Result;
use keyspace_imt::Hash256;
use serde::{Deserialize, Serialize};

use forced_transaction::ForcedTransaction;
use sequenced_transaction::SequencedTransaction;

use super::proof::sp1::SP1ProofVerify;

mod forced_transaction;
mod sequenced_transaction;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Transaction {
    Sequenced(SequencedTransaction),
    Forced(ForcedTransaction),
}

impl Transaction {
    /// Computes the [Transaction] commitment.
    ///
    /// Before computing the transaction commitment, it checks if the transaction commitment chain is valid
    /// by comparing the transaction's `prev_tx_commitment` to the `prev_tx_commitment` argument passed,
    /// and returns an error if they differ.
    pub fn commitment(&self, prev_tx_commitment: Option<Hash256>) -> Result<Hash256> {
        match self {
            Transaction::Sequenced(sequenced_transaction) => {
                sequenced_transaction.commitment(prev_tx_commitment)
            }
            Transaction::Forced(forced_transaction) => {
                forced_transaction.commitment(prev_tx_commitment)
            }
        }
    }

    /// Verifies the transaction proof.
    pub fn verify_proof(&self, sp1_verify: SP1ProofVerify) -> Result<()> {
        match self {
            Transaction::Sequenced(sequenced_transaction) => {
                sequenced_transaction.verify_proof(sp1_verify)
            }
            Transaction::Forced(forced_transaction) => forced_transaction.verify_proof(),
        }
    }

    /// Verifies the imt mutate proof.
    pub fn verify_imt_mutate(&self, old_root: &Hash256) -> Result<Hash256> {
        match self {
            Transaction::Sequenced(sequenced_transaction) => {
                sequenced_transaction.verify_imt_mutate(old_root)
            }
            Transaction::Forced(forced_transaction) => {
                forced_transaction.verify_imt_mutate(old_root)
            }
        }
    }
}
