use super::{insert::InsertProof, update::UpdateProof};

/// A imt mutation that can either be an insert or an update.
pub enum MutateProof<K, V> {
    /// An [InsertProof].
    Insert(InsertProof<K, V>),
    /// An [UpdateProof].
    Update(UpdateProof<K, V>),
}
