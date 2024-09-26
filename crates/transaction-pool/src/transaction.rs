use serde::{Deserialize, Serialize};
use sp1_sdk::{SP1ProofWithPublicValues, SP1VerifyingKey};

/// A [PendingTransaction], is a transaction waiting to be picked by the Sequencer.
#[derive(Clone, Serialize, Deserialize)]
pub struct PendingTransaction {
    pub proof: SP1ProofWithPublicValues,
    pub vk: SP1VerifyingKey,
}

impl PendingTransaction {
    pub fn sequenced(self) -> SequencedTransaction {
        SequencedTransaction {
            proof: self.proof,
            vk: self.vk,
        }
    }
}

/// A [SequencedTransaction], is a transaction that has been processed by the Sequencer
/// and applied to the local state, but not submitted onchain by the Batcher.
#[derive(Clone, Serialize, Deserialize)]
pub struct SequencedTransaction {
    pub proof: SP1ProofWithPublicValues,
    pub vk: SP1VerifyingKey,
}
