use anyhow::{ensure, Ok, Result};
use gnark_bn254_verifier::{verify, Fr, ProvingSystem};
use k256::sha2::{Digest, Sha256};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

use keyspace_imt::proof::mutate::MutateProof;

use crate::{authorization_key, keyspace_value, Hash256};

const SP1_PLONK_VK_BYTES: &[u8] = include_bytes!("/home/vscode/.sp1/circuits/v2.0.0/plonk_vk.bin");

/// An SP1 proof to use for forced transactions.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct SP1ForcedProof {
    /// The SP1 record proof wrapped in a PLONK.
    pub wrapped_proof: Vec<u8>,
    /// The SP1 record program verifier key hash.
    record_vk_hash: Hash256,
    /// The SP1 record program verifier key hash to use for forced inclusion.
    forced_vk_hash: Hash256,
    /// The SP1 record storage hash.
    storage_hash: Hash256,
}

impl SP1ForcedProof {
    /// Creates a new [SP1ForcedProof].
    pub fn new(
        wrapped_proof: Vec<u8>,
        record_vk_hash: Hash256,
        forced_vk_hash: Hash256,
        storage_hash: Hash256,
    ) -> Self {
        Self {
            wrapped_proof,
            record_vk_hash,
            forced_vk_hash,
            storage_hash,
        }
    }

    /// Verifies the [SP1ForcedProof].
    pub fn verify(&self, imt_mutate_proof: &MutateProof<Hash256, Hash256>) -> Result<()> {
        let (keyspace_id, current_value, new_value) = match imt_mutate_proof {
            MutateProof::Insert(insert) => (insert.node.key, insert.node.key, insert.node.value),
            MutateProof::Update(update) => (update.node.key, update.node.value, update.new_value),
        };

        // Ensure the provided inputs match with the `current_value`.
        let authorization_key = authorization_key(&self.record_vk_hash, Some(&self.forced_vk_hash));
        ensure!(
            current_value == keyspace_value(&authorization_key, &self.storage_hash),
            "authorization_key does not match with current_value"
        );

        let mut pub_inputs = [0; 96];
        pub_inputs[..32].copy_from_slice(&keyspace_id);
        pub_inputs[32..64].copy_from_slice(&current_value);
        pub_inputs[64..].copy_from_slice(&new_value);

        // Verify the PLONK proof.
        let public_values_digest = public_values_digest(keyspace_id, current_value, new_value);

        ensure!(
            verify(
                &self.wrapped_proof,
                SP1_PLONK_VK_BYTES,
                &[
                    Fr::from(BigUint::from_bytes_be(&self.record_vk_hash)),
                    Fr::from(public_values_digest),
                ],
                ProvingSystem::Plonk,
            ),
            "wrapped proof verification failed"
        );

        Ok(())
    }
}

// Temporary copy/paste of SP1PublicValues::hash_bn254() which does not compile when using it from sp1-sdk = { version = "2.0.0" }
fn public_values_digest(
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
