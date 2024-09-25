// use sled::{self, IVec};
// use std::collections::BTreeMap;

// use super::{StorageReader, StorageWriter, Transaction, TransactionalStorage};

// TODO: Fix implementation to work with arbitrary keys and values.

// /// A storage implementation over a [sled::Db].
// #[derive(Debug)]
// pub struct SledStorage<K, V> {
//     db: sled::Db,

//     _phantom_data_k: std::marker::PhantomData<K>,
//     _phantom_data_v: std::marker::PhantomData<V>,
// }

// impl<K, V> SledStorage<K, V> {
//     /// Creates a new [SledStorage].
//     pub fn new(db: sled::Db) -> Self {
//         SledStorage {
//             db,
//             _phantom_data_k: std::marker::PhantomData,
//             _phantom_data_v: std::marker::PhantomData,
//         }
//     }
// }

// impl<K, V> StorageReader for SledStorage<K, V>
// where
//     K: AsRef<[u8]> + From<IVec>,
//     V: From<IVec>,
// {
//     type StorageKey = K;
//     type StorageValue = V;

//     fn get(&self, key: &Self::StorageKey) -> Option<Self::StorageValue> {
//         self.db.get(key).expect("sled get failed").map(|v| v.into())
//     }

//     fn get_lt(&self, key: &Self::StorageKey) -> Option<(Self::StorageKey, Self::StorageValue)> {
//         self.db
//             .get_lt(key)
//             .expect("sled get_lt failed")
//             .map(|(k, v)| (k.into(), v.into()))
//     }
// }

// impl<K, V> StorageWriter for SledStorage<K, V>
// where
//     K: AsRef<[u8]> + From<IVec>,
//     V: From<IVec> + Into<IVec>,
// {
//     fn set(&mut self, key: Self::StorageKey, value: Self::StorageValue) {
//         self.db.insert(key, value).expect("sled set failed");
//     }
// }

// impl<K, V> TransactionalStorage for SledStorage<K, V>
// where
//     K: Clone + Ord + AsRef<[u8]> + From<IVec> + Into<IVec>,
//     V: Clone + From<IVec> + Into<IVec>,
// {
//     type T<'a> = SledTransaction<'a, K, V> where K: 'a, V: 'a;

//     fn transaction(&mut self) -> Self::T<'_> {
//         SledTransaction::new(&mut self.db)
//     }
// }

// /// A storage transaction that can be created from a [SledStorage].
// pub struct SledTransaction<'a, K, V> {
//     db: &'a mut sled::Db,
//     batch: sled::Batch,
//     buffer: BTreeMap<K, V>,
// }

// impl<'a, K, V> SledTransaction<'a, K, V> {
//     /// Creates a new [SledTransaction].
//     pub fn new(db: &'a mut sled::Db) -> Self {
//         SledTransaction {
//             db,
//             batch: sled::Batch::default(),
//             buffer: BTreeMap::new(),
//         }
//     }
// }

// impl<'a, K, V> StorageReader for SledTransaction<'a, K, V>
// where
//     K: Clone + Ord + AsRef<[u8]> + From<IVec>,
//     V: Clone + From<IVec>,
// {
//     type StorageKey = K;
//     type StorageValue = V;

//     fn get(&self, key: &Self::StorageKey) -> Option<Self::StorageValue> {
//         self.buffer
//             .get(key)
//             .cloned()
//             .or_else(|| self.db.get(key).expect("sled get failed").map(Into::into))
//     }

//     fn get_lt(&self, key: &Self::StorageKey) -> Option<(Self::StorageKey, Self::StorageValue)> {
//         self.buffer
//             .range(..key)
//             .next_back()
//             .map(|r| (r.0.clone(), r.1.clone()))
//             .or_else(|| {
//                 self.db
//                     .get_lt(key)
//                     .expect("sled get_lt failed")
//                     .map(|(k, v)| (k.into(), v.into()))
//             })
//     }
// }

// impl<'a, K, V> StorageWriter for SledTransaction<'a, K, V>
// where
//     K: Clone + Ord + AsRef<[u8]> + From<IVec> + Into<IVec>,
//     V: Clone + From<IVec> + Into<IVec>,
// {
//     fn set(&mut self, key: Self::StorageKey, value: Self::StorageValue) {
//         self.batch.insert(key.clone(), value.clone());
//         self.buffer.insert(key, value);
//     }
// }

// impl<'a, K, V> Transaction for SledTransaction<'a, K, V>
// where
//     K: Clone + Ord + AsRef<[u8]> + From<IVec> + Into<IVec>,
//     V: Clone + From<IVec> + Into<IVec>,
// {
//     fn commit(self) {
//         self.db.apply_batch(self.batch).unwrap();
//     }

//     fn discard(self) {}
// }

// #[cfg(test)]
// mod test {

//     use super::*;

//     #[test]
//     fn test_transaction_commit() {
//         // let db = sled::open("/tmp/my_db").unwrap();
//         // let mut storage = SledStorage::new(db);

//         // Start a transaction
//         // let mut txn = storage.transaction();

//         // Perform operations
//         // storage.set("key1", "value1");
//         // txn.set("key2", "value2");

//         // // Read within transaction
//         // assert_eq!(
//         //     txn.get("key1").map(|v| v.as_ref().to_vec()),
//         //     Some(b"value1".to_vec())
//         // );

//         // // Use get_lt
//         // let result = txn.get_lt("key2");
//         // assert_eq!(
//         //     result.map(|(k, v)| (k.as_ref().to_vec(), v.as_ref().to_vec())),
//         //     Some((b"key1".to_vec(), b"value1".to_vec()))
//         // );

//         // // Commit transaction
//         // txn.commit();

//         // assert_eq!(
//         //     storage.get("key1").map(|v| v.as_ref().to_vec()),
//         //     Some(b"value1".to_vec())
//         // );

//         // assert_eq!(
//         //     storage.get("key2").map(|v| v.as_ref().to_vec()),
//         //     Some(b"value2".to_vec())
//         // );
//     }
// }
