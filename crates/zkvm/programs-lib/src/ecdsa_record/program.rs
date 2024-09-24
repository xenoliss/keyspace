use tiny_keccak::{Hasher, Keccak};

use crate::recover_keyspace_value;

use super::inputs::Inputs;

pub struct Program;

impl Program {
    pub fn run(inputs: &Inputs) {
        // Compute the `msg_hash`: keccack(keyspace_id, current_value, new_value).
        let mut k = Keccak::v256();
        let mut msg_hash = [0; 32];
        k.update(&inputs.keyspace_id);
        k.update(&inputs.current_value);
        k.update(&inputs.new_value);
        k.finalize(&mut msg_hash);

        // Recover the public key from the signature and `msg_hash`.
        let recovered_pub_key = inputs.sig.ecrecover(&msg_hash);

        // Recover the current `keyspace_value`.
        let keyspace_value = recover_keyspace_value(
            &inputs.authorization_key,
            &recovered_pub_key,
            &inputs.sidecar_hash,
        );

        // Ensure the recovered `current_value` matches with the one passed as public input.
        assert_eq!(inputs.current_value, keyspace_value);
    }
}
