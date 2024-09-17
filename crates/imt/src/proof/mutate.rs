use super::{insert::ImtInsert, update::ImtUpdate};

/// A imt mutation that can either be an insert or an update.
pub enum ImtMutate<K, V> {
    /// An [ImtInsert].
    Insert(ImtInsert<K, V>),
    /// An [ImtUpdate].
    Update(ImtUpdate<K, V>),
}
