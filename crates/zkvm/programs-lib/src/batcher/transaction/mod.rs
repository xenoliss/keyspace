use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};

use crate::Hash256;
use keyspace_imt::proof::mutate::MutateProof;

use super::proof::Proof;

#[derive(Debug, Deserialize, Serialize)]
pub struct Transaction {
    /// The IMT mutate proof associated with this transaction.
    pub imt_mutate_proof: MutateProof<Hash256, Hash256>,
    /// The previous transaction commitment.
    pub prev_tx_commitment: Hash256,
    // The proof to verify.
    pub proof: Proof,
}

impl Transaction {
    /// Computes the [Transaction] commitment.
    pub fn commitment(&self) -> Hash256 {
        let (keyspace_id, new_value) = match &self.imt_mutate_proof {
            MutateProof::Insert(insert) => (insert.node.key, insert.node.value),
            MutateProof::Update(update) => (update.node.key, update.new_value),
        };

        let mut k = Keccak::v256();
        k.update(&self.prev_tx_commitment);
        k.update(&keyspace_id);
        k.update(&new_value);
        k.update(match &self.proof {
            Proof::Custom { proof, .. } => proof,
            Proof::SP1 { proof, .. } => proof,
        });

        let mut hash = [0; 32];
        k.finalize(&mut hash);
        hash
    }
}
