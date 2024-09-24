use anyhow::{ensure, Ok, Result};
use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};

use crate::{
    batcher::proof::sp1::{SP1Proof, SP1ProofVerify},
    Hash256,
};
use keyspace_imt::proof::mutate::MutateProof;

/// A sequenced transaction is submitted to the KeySpace node.
///
/// A [SequencedTransaction::proof] MUST be valid. If the proof is invalid,
/// the entire batch proof MUST fail.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SequencedTransaction {
    /// The imt mutate proof associated with this transaction.
    imt_mutate_proof: MutateProof<Hash256, Hash256>,
    /// The previous transaction commitment.
    prev_tx_commitment: Hash256,
    // The proof to verify.
    proof: SP1Proof,
}

impl SequencedTransaction {
    /// Creates a new [SequencedTransaction].
    pub fn new(
        imt_mutate_proof: MutateProof<Hash256, Hash256>,
        prev_tx_commitment: Hash256,
        proof: SP1Proof,
    ) -> Self {
        Self {
            imt_mutate_proof,
            prev_tx_commitment,
            proof,
        }
    }

    /// Computes the [SequencedTransaction] commitment.
    ///
    /// Before computing the transaction commitment, it checks if the transaction commitment chain is valid
    /// by comparing the [SequencedTransaction::prev_tx_commitment] to the `prev_tx_commitment` argument passed,
    /// and returns an error if they differ.
    pub fn commitment(&self, prev_tx_commitment: Option<Hash256>) -> Result<Hash256> {
        // Make sure the `prev_tx_commitment` matches the one stored in the transaction.
        if let Some(prev_tx_commitment) = prev_tx_commitment {
            ensure!(
                prev_tx_commitment == self.prev_tx_commitment,
                "tx commitment chain is invalid"
            );
        }

        // Extract the record update from the imt MutateProof.
        let (keyspace_id, current_value, new_value) = match &self.imt_mutate_proof {
            MutateProof::Insert(insert) => (insert.node.key, insert.node.key, insert.node.value),
            MutateProof::Update(update) => (update.node.key, update.node.value, update.new_value),
        };

        // Compute the transaction commitment.
        let mut k = Keccak::v256();
        k.update(&self.prev_tx_commitment);
        k.update(&keyspace_id);
        k.update(&current_value);
        k.update(&new_value);

        let mut hash = [0; 32];
        k.finalize(&mut hash);

        Ok(hash)
    }

    /// Verifies the transaction [SP1Proof].
    pub fn verify_proof(&self, sp1_verify: SP1ProofVerify) -> Result<()> {
        self.proof.verify(&self.imt_mutate_proof, sp1_verify)
    }

    /// Verifies the imt [MutateProof].
    pub fn verify_imt_mutate(&self, old_root: &Hash256) -> Result<Hash256> {
        self.imt_mutate_proof.verify(Keccak::v256, *old_root)
    }
}
