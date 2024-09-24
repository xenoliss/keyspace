use gnark_bn254_verifier::{verify, Fr, ProvingSystem};
use k256::sha2::{Digest, Sha256};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

use crate::{authorization_key, keyspace_value, Hash256};
use keyspace_imt::proof::mutate::MutateProof;

const SP1_PLONK_VK_BYTES: &[u8] = include_bytes!("/home/vscode/.sp1/circuits/v2.0.0/plonk_vk.bin");

#[derive(Debug, Deserialize, Serialize)]
pub enum Proof {
    Custom {
        proof: Vec<u8>,
        vk: Vec<u8>,
        storage_hash: Hash256,
    },
    SP1 {
        proof: Vec<u8>,
        inner_vk_hash: Hash256,
        storage_hash: Hash256,
    },
}

impl Proof {
    pub fn is_valid(&self, imt_mutate_proof: &MutateProof<Hash256, Hash256>) -> bool {
        let (keyspace_id, current_value, new_value) = match imt_mutate_proof {
            MutateProof::Insert(insert) => (insert.node.key, insert.node.key, insert.node.value),
            MutateProof::Update(update) => (update.node.key, update.node.value, update.new_value),
        };

        let (proof, vk, inner_vk_hash) = match self {
            Proof::Custom {
                proof,
                vk,
                storage_hash,
            } => {
                // Ensure the provided inputs match with the `current_value`.
                let inner_vk_hash = Sha256::digest(vk).into();
                let authorization_key = authorization_key(&inner_vk_hash, None);
                let keyspace_value = keyspace_value(&authorization_key, storage_hash);
                assert_eq!(current_value, keyspace_value);

                (proof, &vk[..], inner_vk_hash)
            }
            Proof::SP1 {
                proof,
                inner_vk_hash,
                storage_hash,
            } => {
                // Ensure the provided inputs match with the `current_value`.
                let outer_vk_hash = Sha256::digest(SP1_PLONK_VK_BYTES).into();
                let authorization_key = authorization_key(inner_vk_hash, Some(&outer_vk_hash));
                let keyspace_value = keyspace_value(&authorization_key, storage_hash);
                assert_eq!(current_value, keyspace_value);

                (proof, SP1_PLONK_VK_BYTES, *inner_vk_hash)
            }
        };

        // Verify the PLONK proof.
        let public_values_digest = self.public_values_digest(keyspace_id, current_value, new_value);

        verify(
            proof,
            vk,
            &[
                Fr::from(BigUint::from_bytes_be(&inner_vk_hash)),
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
