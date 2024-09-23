use tiny_keccak::{Hasher, Keccak};

use crate::keyspace_value_from_storage;

use super::inputs::Inputs;

pub struct Program;

impl Program {
    pub fn run(inputs: &Inputs) {
        // Compute the `msg_hash`: keccack(keyspace_id, new_value).
        let mut k = Keccak::v256();
        let mut msg_hash = [0; 32];
        k.update(&inputs.keyspace_id);
        k.update(&inputs.new_value);
        k.finalize(&mut msg_hash);

        // Recover the public key from the signature and `msg_hash`.
        let recovered_pub_key = inputs.sig.ecrecover(&msg_hash);

        // Recover the `current_value`: keccack(storage_hash, vk_hash).
        let current_value = keyspace_value_from_storage(&inputs.vk_hash, &recovered_pub_key);

        // Ensure the recovered `current_value` matches with the one passed as public input.
        assert_eq!(inputs.current_value, current_value);
    }
}
