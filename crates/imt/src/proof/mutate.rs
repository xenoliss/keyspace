use super::{insert::ImtInsert, update::ImtUpdate};

pub enum ImtMutate<K, V> {
    Insert(ImtInsert<K, V>),
    Update(ImtUpdate<K, V>),
}
