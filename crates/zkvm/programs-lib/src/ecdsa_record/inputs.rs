use serde::{Deserialize, Serialize};

use crate::Hash256;

use super::k_signature::KSignature;

#[derive(Debug, Deserialize, Serialize)]
pub struct Inputs {
    /// Public input: the Keyspace id.
    pub keyspace_id: Hash256,
    /// Public input: the Keyspace current key.
    pub current_key: Hash256,
    /// Public input: the Keyspace new key.
    pub new_key: Hash256,

    /// Private input: the signature over keccak(keyspace_id, new_key).
    pub sig: KSignature,
    /// Private input: the verifier key hash.
    pub vk_hash: Hash256,
}
