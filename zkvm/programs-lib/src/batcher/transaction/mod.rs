use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};

use super::proof::{
    sp1::{SP1Proof, SP1ProofVerify},
    sp1_forced::SP1ForcedProof,
};
use forced_transaction::ForcedTransaction;
use keyspace_imt::{proof::mutate::MutateProof, Hash256};
use sequenced_transaction::SequencedTransaction;

mod forced_transaction;
mod sequenced_transaction;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Transaction {
    Sequenced(SequencedTransaction),
    Forced(ForcedTransaction),
}

impl Transaction {
    /// Creates a new [SequencedTransaction].
    pub fn sequenced(
        imt_mutate_proof: MutateProof<Hash256, Hash256>,
        prev_tx_commitment: Hash256,
        proof: SP1Proof,
    ) -> Self {
        Self::Sequenced(SequencedTransaction::new(
            imt_mutate_proof,
            prev_tx_commitment,
            proof,
        ))
    }

    /// Creates a new [ForcedTransaction].
    pub fn forced(
        imt_mutate_proof: MutateProof<Hash256, Hash256>,
        prev_tx_commitment: Hash256,
        proof: SP1ForcedProof,
    ) -> Self {
        Self::Forced(ForcedTransaction::new(
            imt_mutate_proof,
            prev_tx_commitment,
            proof,
        ))
    }

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

    /// Processes the [Transaction] and returns the new imt root.
    ///
    /// Transactions are processed differently:
    /// - [SequencedTransaction] MUST have a valid [SequencedTransaction::proof] and [SequencedTransaction::imt_mutate_proof].
    ///   If either of those fail, an error is returned.
    /// - [ForcedTransaction] MIGHT have an invalid [ForcedTransaction::proof]. In such case, the transaction is ignored,
    ///   its [SequencedTransaction::imt_mutate_proof] is not applied (thus not verified either) and the `old_root` is returned
    ///   as is. If the [ForcedTransaction::proof] is valid, the [SequencedTransaction::imt_mutate_proof] MUST be valid, else
    ///   an error is returned.
    pub fn process(&self, sp1_verify: SP1ProofVerify, old_root: &Hash256) -> Result<Hash256> {
        match self {
            Transaction::Sequenced(sequenced_transaction) => {
                sequenced_transaction.verify_proof(sp1_verify)?;
                sequenced_transaction.verify_imt_mutate(old_root)
            }
            Transaction::Forced(forced_transaction) => {
                if forced_transaction.verify_proof().is_ok() {
                    forced_transaction.verify_imt_mutate(old_root)
                } else {
                    Ok(*old_root)
                }
            }
        }
    }
}
