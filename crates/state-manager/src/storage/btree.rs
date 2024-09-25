use std::collections::BTreeMap;

use super::{StorageReader, StorageWriter, Transaction, TransactionalStorage};

/// A storage implementation over a [BTreeMap].
#[derive(Debug, Default)]
pub struct BTreeStorage<K, V> {
    data: BTreeMap<K, V>,
}

impl<K, V> BTreeStorage<K, V> {
    /// Creates a new [BTreeStorage].
    pub fn new() -> Self {
        BTreeStorage {
            data: BTreeMap::new(),
        }
    }
}

impl<K, Vaue> StorageReader for BTreeStorage<K, Vaue>
where
    K: Clone + Ord,
    Vaue: Clone,
{
    type StorageKey = K;
    type StorageValue = Vaue;

    fn get(&self, key: &Self::StorageKey) -> Option<Self::StorageValue> {
        self.data.get(key).cloned()
    }

    fn get_lt(&self, key: &Self::StorageKey) -> Option<(Self::StorageKey, Self::StorageValue)> {
        self.data
            .range(..key)
            .next_back()
            .map(|(k, v)| (k.clone(), v.clone()))
    }
}

impl<K, V> StorageWriter for BTreeStorage<K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    fn set(&mut self, key: Self::StorageKey, value: Self::StorageValue) {
        self.data.insert(key, value);
    }
}

impl<K, V> TransactionalStorage for BTreeStorage<K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    type T<'a> = BTreeTransaction<'a, K, V> where K: 'a, V: 'a;

    fn transaction(&mut self) -> Self::T<'_> {
        BTreeTransaction::new(self)
    }
}

/// A storage transaction that can be created from a [BTreeStorage].
pub struct BTreeTransaction<'a, K, V> {
    storage: &'a mut BTreeStorage<K, V>,
    buffer: BTreeMap<K, V>,
}

impl<'a, K, V> BTreeTransaction<'a, K, V> {
    /// Creates a new [BTreeTransaction].
    pub fn new(storage: &'a mut BTreeStorage<K, V>) -> Self {
        BTreeTransaction {
            storage,
            buffer: BTreeMap::new(),
        }
    }
}

impl<'a, K, V> StorageReader for BTreeTransaction<'a, K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    type StorageKey = K;
    type StorageValue = V;

    fn get(&self, key: &Self::StorageKey) -> Option<Self::StorageValue> {
        self.buffer
            .get(key)
            .cloned()
            .or_else(|| self.storage.get(key))
    }

    fn get_lt(&self, key: &Self::StorageKey) -> Option<(Self::StorageKey, Self::StorageValue)> {
        let closest_buffer_entry = self.buffer.range(..key).next_back();
        let closest_storage_entry = self.storage.data.range(..key).next_back();

        match (closest_buffer_entry, closest_storage_entry) {
            (None, None) => None,
            (None, Some(storage_entry)) => Some((storage_entry.0.clone(), storage_entry.1.clone())),
            (Some(buffer_entry), None) => Some((buffer_entry.0.clone(), buffer_entry.1.clone())),
            (Some(buffer_entry), Some(storage_entry)) => {
                if buffer_entry.0 > storage_entry.0 {
                    Some((buffer_entry.0.clone(), buffer_entry.1.clone()))
                } else {
                    Some((storage_entry.0.clone(), storage_entry.1.clone()))
                }
            }
        }
    }
}

impl<'a, K, V> StorageWriter for BTreeTransaction<'a, K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    fn set(&mut self, key: Self::StorageKey, value: Self::StorageValue) {
        self.buffer.insert(key, value);
    }
}

impl<'a, K, V> Transaction for BTreeTransaction<'a, K, V>
where
    K: Clone + Ord,
    V: Clone,
{
    fn commit(self) {
        for (k, v) in self.buffer {
            self.storage.set(k, v);
        }
    }

    fn discard(self) {}
}
