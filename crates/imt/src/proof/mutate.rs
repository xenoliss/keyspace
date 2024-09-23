use anyhow::Result;
use serde::{Deserialize, Serialize};
use tiny_keccak::Hasher;

use crate::{Hash256, NodeKey, NodeValue};

use super::{insert::InsertProof, update::UpdateProof};

/// A imt mutation that can either be an insert or an update.
#[derive(Debug, Deserialize, Serialize)]
pub enum MutateProof<K, V> {
    /// An [InsertProof].
    Insert(InsertProof<K, V>),
    /// An [UpdateProof].
    Update(UpdateProof<K, V>),
}

impl<K, V> MutateProof<K, V>
where
    K: NodeKey,
    V: NodeValue,
{
    /// Verifies the IMT mutate proof and return the new updated root.
    ///
    /// Before performing the mutation, the state is checked to make sure it is coherent.
    /// In case of any inconsistency, an error is returned.
    pub fn verify<H: Hasher>(
        &self,
        hasher_factory: fn() -> H,
        old_root: Hash256,
    ) -> Result<Hash256> {
        match &self {
            MutateProof::Insert(insert) => insert.verify(hasher_factory, old_root),
            MutateProof::Update(update) => update.verify(hasher_factory, old_root),
        }
    }
}
