use super::{inputs::Inputs, proof::sp1::SP1ProofVerify};

pub struct Program;

impl Program {
    pub fn run(inputs: &Inputs, sp1_verify: SP1ProofVerify) {
        let mut root = inputs.old_root;
        let mut tx_commitment = None;

        for tx in &inputs.txs {
            // 1. Chain the tx commitments.
            tx_commitment = Some(
                tx.commitment(tx_commitment)
                    .expect("failed to chain tx commitments"),
            );

            // TODO: Allow onchain proof to fail.

            // 2. Verify the record proof.
            tx.verify_proof(sp1_verify)
                .expect("failed to verify the proof");

            // 3. Verify the imt MutateProof and compute the new root.
            root = tx
                .verify_imt_mutate(&root)
                .expect("failed to verify the imt MutateProof");
        }

        // Make sure the final root obtained after applying the txs matches with the provided new_root.
        assert_eq!(root, inputs.new_root);

        // Make sure the final tx commitement obtained after applying the txs matches with the provided txs_commitment.
        assert_eq!(tx_commitment.expect("impossible"), inputs.txs_commitment);
    }
}
