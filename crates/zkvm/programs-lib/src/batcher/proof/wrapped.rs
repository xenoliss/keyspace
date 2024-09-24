use gnark_bn254_verifier::{verify, Fr, ProvingSystem};
use k256::sha2::{Digest, Sha256};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

use crate::Hash256;
use keyspace_imt::proof::mutate::MutateProof;

#[derive(Debug, Deserialize, Serialize)]
pub struct WrappedProof {
    /// The plonk's record verifier key.
    pub vk: Vec<u8>,
    /// The record proof.
    pub proof: Vec<u8>,
}

impl WrappedProof {
    pub fn is_valid_record_proof(&self, imt_mutate_proof: &MutateProof<Hash256, Hash256>) -> bool {
        let (keyspace_id, current_value, new_value) = match imt_mutate_proof {
            MutateProof::Insert(insert) => (insert.node.key, insert.node.key, insert.node.value),
            MutateProof::Update(update) => (update.node.key, update.node.value, update.new_value),
        };

        // TODO: Ensure the provided inputs match with the `current_value`.
        // let vk_hash_num = BigUint::from_str_radix(&self.plonk_vk_hash, 10).unwrap();
        // let vk_hash = &vk_hash_num.to_bytes_be().as_slice().try_into().unwrap();
        // let keyspace_key = keyspace_key_from_storage_hash(vk_hash, &self.storage_hash);
        // assert_eq!(current_key, keyspace_key);

        // Verify the PLONK proof.
        let public_values_digest = self.public_values_digest(keyspace_id, current_value, new_value);
        verify(
            &self.proof,
            &self.vk,
            &[
                // Fr::from(BigUint::from_str_radix(&self.zkvm_vk_hash, 10).unwrap()),
                Fr::from(public_values_digest),
            ],
            ProvingSystem::Plonk,
        )
    }

    // Temporary copy/paste of SP1PublicValues::from().hash_bn254() which does not compile
    // when using it from sp1-sdk = { version = "2.0.0" }
    fn public_values_digest(
        &self,
        keyspace_id: Hash256,
        current_value: Hash256,
        new_value: Hash256,
    ) -> BigUint {
        // Collect the original zkVm record program public inputs.
        let mut original_pub_inputs = [0; 96];
        original_pub_inputs[..32].copy_from_slice(&keyspace_id);
        original_pub_inputs[32..64].copy_from_slice(&current_value);
        original_pub_inputs[64..].copy_from_slice(&new_value);

        // Hash the public values.
        let mut hasher = Sha256::new();
        hasher.update(original_pub_inputs);
        let hash_result = hasher.finalize();
        let mut hash = hash_result.to_vec();

        // Mask the top 3 bits.
        hash[0] &= 0b00011111;

        // Return the masked hash as a BigUint.
        BigUint::from_bytes_be(&hash)
    }
}
