use serde::{Deserialize, Serialize};

use crate::Hash256;

use super::signature::Signature;

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Inputs {
    /// Public input: the Keyspace id.
    pub keyspace_id: Hash256,
    /// Public input: the Keyspace current value.
    pub current_value: Hash256,
    /// Public input: the Keyspace new value.
    pub new_value: Hash256,

    /// Private input: the signature over keccak(keyspace_id, new_value).
    pub sig: Signature,
    /// Private input: the authorization key.
    pub authorization_key: Hash256,
    /// Private input: the sidecar hash.
    pub sidecar_hash: Hash256,
}
