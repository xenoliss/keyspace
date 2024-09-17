use std::collections::BTreeMap;

use super::{StorageReader, StorageWriter, Transaction, TransactionalStorage};

/// A storage implementation over a [BTreeMap].
#[derive(Debug, Default)]
pub struct BTreeStorage {
    data: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl BTreeStorage {
    /// Creates a new [BTreeStorage].
    pub fn new() -> Self {
        BTreeStorage {
            data: BTreeMap::new(),
        }
    }
}

impl StorageReader for BTreeStorage {
    fn get(&self, key: impl AsRef<[u8]>) -> Option<impl AsRef<[u8]>> {
        self.data.get(key.as_ref()).cloned()
    }

    fn get_lt(&self, key: impl AsRef<[u8]>) -> Option<(impl AsRef<[u8]>, impl AsRef<[u8]>)> {
        self.data
            .range(..key.as_ref().to_vec())
            .next_back()
            .map(|(k, v)| (k.clone(), v.clone()))
    }
}

impl StorageWriter for BTreeStorage {
    fn set(&mut self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) {
        self.data
            .insert(key.as_ref().to_vec(), value.as_ref().to_vec());
    }
}

impl TransactionalStorage for BTreeStorage {
    type T<'a> = BTreeTransaction<'a>;

    fn transaction(&mut self) -> Self::T<'_> {
        BTreeTransaction::new(self)
    }
}

/// A storage transaction that can be created from a [BTreeStorage].
pub struct BTreeTransaction<'a> {
    storage: &'a mut BTreeStorage,
    buffer: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl<'a> BTreeTransaction<'a> {
    /// Creates a new [BTreeTransaction].
    pub fn new(storage: &'a mut BTreeStorage) -> Self {
        BTreeTransaction {
            storage,
            buffer: BTreeMap::new(),
        }
    }
}

impl<'a> StorageReader for BTreeTransaction<'a> {
    fn get(&self, key: impl AsRef<[u8]>) -> Option<impl AsRef<[u8]>> {
        let key_bytes = key.as_ref();

        self.buffer
            .get(key_bytes)
            .cloned()
            .or_else(|| self.storage.get(key_bytes).map(|v| v.as_ref().to_vec()))
    }

    fn get_lt(&self, key: impl AsRef<[u8]>) -> Option<(impl AsRef<[u8]>, impl AsRef<[u8]>)> {
        let key_bytes = key.as_ref().to_vec();

        let closest_buffer_entry = self.buffer.range(..key_bytes.clone()).next_back();
        let closest_storage_entry = self.storage.data.range(..key_bytes.clone()).next_back();

        match (closest_buffer_entry, closest_storage_entry) {
            (None, None) => None,
            (None, Some(storage_entry)) => Some(storage_entry),
            (Some(buffer_entry), None) => Some(buffer_entry),
            (Some(buffer_entry), Some(storage_entry)) => {
                if buffer_entry.0 > storage_entry.0 {
                    Some(buffer_entry)
                } else {
                    Some(storage_entry)
                }
            }
        }
    }
}

impl<'a> StorageWriter for BTreeTransaction<'a> {
    fn set(&mut self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) {
        self.buffer
            .insert(key.as_ref().to_vec(), value.as_ref().to_vec());
    }
}

impl<'a> Transaction for BTreeTransaction<'a> {
    fn commit(self) {
        for (k, v) in self.buffer {
            self.storage.set(k, v);
        }
    }

    fn discard(self) {}
}
