mod imt;

pub mod btree;
pub mod sled;

/// Trait providing ordered storage read access.
pub trait StorageReader {
    type StorageKey;
    type StorageValue;

    /// Fetches the `key` value from the storage.
    fn get(&self, key: &Self::StorageKey) -> Option<Self::StorageValue>;
    /// Fetches the closest (key; value) pair value right below the given `key`.
    fn get_lt(&self, key: &Self::StorageKey) -> Option<(Self::StorageKey, Self::StorageValue)>;
}

/// Trait providing storage write access.
pub trait StorageWriter: StorageReader {
    /// Sets the (key; value) pair in storage.
    fn set(&mut self, key: Self::StorageKey, value: Self::StorageValue);
}

/// Trait to implement for storages that allow atomic batch updates.
pub trait TransactionalStorage {
    type T<'a>: Transaction
    where
        Self: 'a;

    /// Returns a new [Transaction] for atomic batch updates.
    fn transaction(&mut self) -> Self::T<'_>;
}

/// A storage transaction that can be commited atomically or discarded.
pub trait Transaction: StorageWriter {
    /// Consumes the [Transaction] and apply its changes to the storage.
    fn commit(self);

    /// Consumes the [Transaction] without applying its changes to the storage.
    fn discard(self);
}
