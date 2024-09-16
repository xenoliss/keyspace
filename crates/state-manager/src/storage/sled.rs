use sled;
use std::collections::BTreeMap;

use super::{StorageReader, StorageWriter, Transaction, TransactionalStorage};

#[derive(Debug)]
pub struct SledStorage {
    db: sled::Db,
}

impl SledStorage {
    pub fn new(db: sled::Db) -> Self {
        SledStorage { db }
    }
}

impl StorageReader for SledStorage {
    fn get(&self, key: impl AsRef<[u8]>) -> Option<impl AsRef<[u8]>> {
        self.db.get(key).expect("sled get failed")
    }

    fn get_lt(&self, key: impl AsRef<[u8]>) -> Option<(impl AsRef<[u8]>, impl AsRef<[u8]>)> {
        self.db.get_lt(key).expect("sled get_lt failed")
    }
}

impl StorageWriter for SledStorage {
    fn set(&mut self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) {
        self.db
            .insert(key, value.as_ref())
            .expect("sled set failed");
    }
}

impl TransactionalStorage for SledStorage {
    type T<'a> = SledTransaction<'a>;

    fn transaction(&mut self) -> Self::T<'_> {
        SledTransaction::new(&mut self.db)
    }
}

pub struct SledTransaction<'a> {
    db: &'a mut sled::Db,
    batch: sled::Batch,
    buffer: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl<'a> SledTransaction<'a> {
    pub fn new(db: &'a mut sled::Db) -> Self {
        SledTransaction {
            db,
            batch: sled::Batch::default(),
            buffer: BTreeMap::new(),
        }
    }
}

impl<'a> StorageReader for SledTransaction<'a> {
    fn get(&self, key: impl AsRef<[u8]>) -> Option<impl AsRef<[u8]>> {
        let key_bytes = key.as_ref();

        self.buffer.get(key_bytes).cloned().or_else(|| {
            self.db
                .get(key_bytes)
                .expect("sled get failed")
                .map(|ivec| ivec.to_vec())
        })
    }

    fn get_lt(&self, key: impl AsRef<[u8]>) -> Option<(impl AsRef<[u8]>, impl AsRef<[u8]>)> {
        self.buffer
            .range(..key.as_ref().to_vec())
            .next_back()
            .map(|r| (r.0.clone(), r.1.clone()))
            .or_else(|| {
                self.db
                    .get_lt(key)
                    .expect("sled get_lt failed")
                    .map(|(k, v)| (k.to_vec(), v.to_vec()))
            })
    }
}

impl<'a> StorageWriter for SledTransaction<'a> {
    fn set(&mut self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) {
        let key_vec = key.as_ref().to_vec();
        let value_vec = value.as_ref().to_vec();

        self.batch.insert(key_vec.clone(), value_vec.clone());
        self.buffer.insert(key_vec, value_vec);
    }
}

impl<'a> Transaction for SledTransaction<'a> {
    fn commit(self) {
        self.db.apply_batch(self.batch).unwrap();
    }

    fn discard(self) {}
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_transaction_commit() {
        let db = sled::open("/tmp/my_db").unwrap();
        let mut storage = SledStorage::new(db);

        // Start a transaction
        let mut txn = storage.transaction();

        // Perform operations
        txn.set("key1", "value1");
        txn.set("key2", "value2");

        // Read within transaction
        assert_eq!(
            txn.get("key1").map(|v| v.as_ref().to_vec()),
            Some(b"value1".to_vec())
        );

        // Use get_lt
        let result = txn.get_lt("key2");
        assert_eq!(
            result.map(|(k, v)| (k.as_ref().to_vec(), v.as_ref().to_vec())),
            Some((b"key1".to_vec(), b"value1".to_vec()))
        );

        // Commit transaction
        txn.commit();

        assert_eq!(
            storage.get("key1").map(|v| v.as_ref().to_vec()),
            Some(b"value1".to_vec())
        );

        assert_eq!(
            storage.get("key2").map(|v| v.as_ref().to_vec()),
            Some(b"value2".to_vec())
        );
    }
}
