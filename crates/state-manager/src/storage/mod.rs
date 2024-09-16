mod imt;

pub mod btree;
pub mod keys;
pub mod sled;

pub trait StorageReader {
    fn get(&self, key: impl AsRef<[u8]>) -> Option<impl AsRef<[u8]>>;
    fn get_lt(&self, key: impl AsRef<[u8]>) -> Option<(impl AsRef<[u8]>, impl AsRef<[u8]>)>;
}

pub trait StorageWriter: StorageReader {
    fn set(&mut self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>);
}

pub trait TransactionalStorage {
    type T<'a>: Transaction
    where
        Self: 'a;

    fn transaction(&mut self) -> Self::T<'_>;
}

pub trait Transaction: StorageWriter {
    fn commit(self);
    fn discard(self);
}
