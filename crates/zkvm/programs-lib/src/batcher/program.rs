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

            // 2. Process the transaction.
            root = tx
                .process(sp1_verify, &root)
                .expect("failed to process the transaction");
        }

        // Make sure the final root obtained after applying the txs matches with the provided new_root.
        assert_eq!(root, inputs.new_root);

        // Make sure the final tx commitement obtained after applying the txs matches with the provided txs_commitment.
        assert_eq!(tx_commitment.expect("impossible"), inputs.txs_commitment);
    }
}
