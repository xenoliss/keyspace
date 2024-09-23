use serde::{Deserialize, Serialize};

use crate::Hash256;

use super::k_signature::KSignature;

#[derive(Debug, Deserialize, Serialize)]
pub struct Inputs {
    /// Public input: the Keyspace id.
    pub keyspace_id: Hash256,
    /// Public input: the Keyspace current value.
    pub current_value: Hash256,
    /// Public input: the Keyspace new value.
    pub new_value: Hash256,

    /// Private input: the signature over keccak(keyspace_id, new_value).
    pub sig: KSignature,
    /// Private input: the verifier key hash.
    pub vk_hash: Hash256,
}
