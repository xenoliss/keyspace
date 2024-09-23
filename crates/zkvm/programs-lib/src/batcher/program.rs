use super::inputs::Inputs;

pub struct Program;

impl Program {
    pub fn run(inputs: &Inputs) {
        let root = inputs.old_root;
        let mut tx_commitment = [0; 32];

        for tx in &inputs.txs {
            // 1. Chain the tx commitments.
            tx_commitment = tx.commitment();

            // // 2. Verify the record proof.
            // //
            // // The record proof MUST be valid for offchain txs and MAY be invalid for onchain txs.
            // // If an onchain tx has an invalid record proof, it is skipped (its IMTMutate is not applied).
            // match tx {
            //     crate::batcher::tx::Tx::Offchain(offchain) => offchain.process_proof(sp1_verify),
            //     crate::batcher::tx::Tx::Onchain(onchain) => {
            //         if !onchain.is_valid_record_proof() {
            //             continue;
            //         }
            //     }
            // };

            // // 3. Verify the IMTMutate and compute the new root.
            // root = tx
            //     .verify_imt_mutate(&root)
            //     .expect("failed to verify the IMTMutate");
        }

        // Make sure the final root obtained after applying the txs matches with the provided new_root.
        assert_eq!(root, inputs.new_root);

        // Make sure the final tx commitement obtained after applying the txs matches with the provided txs_commitment.
        assert_eq!(tx_commitment, inputs.txs_commitment);
    }
}
