use anyhow::{anyhow, ensure, Ok, Result};
use k256::sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};

use keyspace_imt::proof::mutate::MutateProof;

use crate::{authorization_key, keyspace_value, Hash256};

pub type SP1ProofVerify = fn(&[u32; 8], &Hash256);

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct SP1Proof {
    /// The SP1 record program verifier key hash.
    record_vk_hash: Hash256,
    /// The SP1 record program verifier key hash to use for forced inclusion.
    forced_vk_hash: Hash256,
    /// The SP1 record storage hash.
    storage_hash: Hash256,
}

impl SP1Proof {
    /// Creates a new [SP1Proof].
    pub fn new(record_vk_hash: Hash256, forced_vk_hash: Hash256, storage_hash: Hash256) -> Self {
        Self {
            record_vk_hash,
            forced_vk_hash,
            storage_hash,
        }
    }

    /// Verifies the [SP1Proof].
    pub fn verify(
        &self,
        imt_mutate_proof: &MutateProof<Hash256, Hash256>,
        sp1_verify: SP1ProofVerify,
    ) -> Result<()> {
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

        let public_values_digest = Sha256::digest(pub_inputs);

        let vk_hash = bytes_to_words_be(&self.record_vk_hash)
            .try_into()
            .map_err(|_| anyhow!("failed to parse record_vk_hash"))?;

        sp1_verify(&vk_hash, &public_values_digest.into());

        Ok(())
    }
}

/// Converts a byte array in big endian to a slice of words.
pub fn bytes_to_words_be(bytes: &[u8]) -> Vec<u32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| u32::from_be_bytes(chunk.try_into().expect("bytes_to_words_be failed")))
        .collect::<Vec<_>>()
}
