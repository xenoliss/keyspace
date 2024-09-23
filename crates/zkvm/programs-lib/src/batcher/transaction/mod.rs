use anyhow::Result;
use serde::{Deserialize, Serialize};
use tiny_keccak::Keccak;

use crate::Hash256;

pub mod forced_transaction;
pub mod sequenced_transaction;

use forced_transaction::ForcedTransaction;
use sequenced_transaction::SequencedTransaction;

#[derive(Debug, Deserialize, Serialize)]
pub enum Tx {
    Forced(ForcedTransaction),
    Sequenced(SequencedTransaction),
}

impl Tx {
    pub fn commitment(&self) -> Hash256 {
        match self {
            Tx::Sequenced(sequenced) => sequenced.commitment(),
            Tx::Forced(forced) => forced.commitment(),
        }
    }

    pub fn verify_imt_mutate_proof(&self, old_root: &Hash256) -> Result<Hash256> {
        match self {
            Tx::Sequenced(sequenced) => sequenced.imt_mutate_proof.verify(Keccak::v256, *old_root),
            Tx::Forced(forced) => forced.imt_mutate_proof.verify(Keccak::v256, *old_root),
        }
    }
}
