use keyspace_imt::proof::mutate::MutateProof;
use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};

use crate::Hash256;

#[derive(Debug, Deserialize, Serialize)]
pub struct SequencedTransaction {
    /// The IMT mutate proof associated with this transaction.
    pub imt_mutate_proof: MutateProof<Hash256, Hash256>,
    /// The previous transaction commitment.
    pub prev_tx_commitment: Hash256,
    // /// The record proof.
    // pub proof: Proof,
}

impl SequencedTransaction {
    pub fn commitment(&self) -> Hash256 {
        let (keyspace_id, new_value) = match &self.imt_mutate_proof {
            MutateProof::Insert(insert) => (insert.node.key, insert.node.value),
            MutateProof::Update(update) => (update.node.key, update.new_value),
        };

        let mut k = Keccak::v256();
        k.update(&self.prev_tx_commitment);
        k.update(&keyspace_id);
        k.update(&new_value);

        let mut hash = [0; 32];
        k.finalize(&mut hash);
        hash
    }

    // pub fn process_proof(&self, sp1_verify: Sp1ProofVerify) {
    //     match &self.proof {
    //         Proof::SP1(proof) => {
    //             proof.commit_to_proof(&self.imt_mutate, sp1_verify);
    //         }
    //         Proof::PLONK(proof) => {
    //             assert!(
    //                 proof.is_valid_record_proof(&self.imt_mutate),
    //                 "invalid PLONK proof"
    //             )
    //         }
    //     }
    // }
}

#[cfg(test)]
mod tests {

    // use super::*;

    // #[test]
    // fn test_hash_insert() {
    //     let mut imt = Imt::new(Keccak::v256);

    //     let node_key = [1; 32];
    //     let node_value = [42; 32];
    //     let insert = imt.insert_node(node_key, node_value);

    //     let sut = OffchainTx {
    //         imt_mutate: insert,
    //         prev_tx_commitment: [0xff; 32],
    //         proof: Proof::SP1(SP1Proof {
    //             record_vk_hash: [0; 32],
    //             storage_hash: [0; 32],
    //         }),
    //     };
    //     let hash = sut.hash();

    //     let mut expected_keccak = Keccak::v256();
    //     expected_keccak.update(&sut.prev_tx_commitment);
    //     expected_keccak.update(&node_key);
    //     expected_keccak.update(&node_value);
    //     let mut expected_commitment = [0u8; 32];
    //     expected_keccak.finalize(&mut expected_commitment);

    //     assert_eq!(hash, expected_commitment);
    // }

    // #[test]
    // fn test_hash_update() {
    //     let mut imt = Imt::new(Keccak::v256);

    //     let node_key = [1; 32];
    //     let node_value = [42; 32];
    //     imt.insert_node(node_key, node_value);
    //     let node_value = [43; 32];
    //     let update = imt.update_node(node_key, node_value);

    //     let offchain_tx = OffchainTx {
    //         imt_mutate: update,
    //         prev_tx_commitment: [0xff; 32],
    //         proof: Proof::SP1(SP1Proof {
    //             record_vk_hash: [0; 32],
    //             storage_hash: [0; 32],
    //         }),
    //     };

    //     let hash = offchain_tx.hash();

    //     let mut expected_keccak = Keccak::v256();
    //     expected_keccak.update(&offchain_tx.prev_tx_commitment);
    //     expected_keccak.update(&node_key);
    //     expected_keccak.update(&node_value);
    //     let mut expected_commitment = [0u8; 32];
    //     expected_keccak.finalize(&mut expected_commitment);

    //     assert_eq!(hash, expected_commitment);
    // }
}
