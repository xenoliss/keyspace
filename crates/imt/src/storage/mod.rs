use crate::{node::ImtNode, Hash256};

#[cfg(test)]
pub(crate) mod btree_imt_storage;

/// Trait for reading and parsing an imt from storage.
pub trait ImtStorageReader {
    type K;
    type V;

    /// Returns the [ImtNode] from the imt form the given `key`.
    fn get_node(&self, key: &Self::K) -> Option<ImtNode<Self::K, Self::V>>;

    /// Returns the low nullifier [ImtNode] from the imt for the given `key`.
    fn get_ln_node(&self, key: &Self::K) -> Option<ImtNode<Self::K, Self::V>>;

    /// Returns the [Hash256] cached for the given (`level`; `index`) pair.
    fn get_hash(&self, level: u8, index: u64) -> Option<Hash256>;

    /// Returns the imt size (including the 0 node).
    fn get_size(&self) -> Option<u64>;

    /// Returns the imt root (including the size).
    fn get_root(&self) -> Option<Hash256>;
}

impl<T> ImtStorageReader for &T
where
    T: ImtStorageReader,
{
    type K = T::K;
    type V = T::V;

    fn get_node(&self, key: &Self::K) -> Option<ImtNode<Self::K, Self::V>> {
        T::get_node(*self, key)
    }

    fn get_ln_node(&self, key: &Self::K) -> Option<ImtNode<Self::K, Self::V>> {
        T::get_ln_node(*self, key)
    }

    fn get_hash(&self, level: u8, index: u64) -> Option<Hash256> {
        T::get_hash(*self, level, index)
    }

    fn get_size(&self) -> Option<u64> {
        T::get_size(*self)
    }

    fn get_root(&self) -> Option<Hash256> {
        T::get_root(*self)
    }
}

impl<T> ImtStorageReader for &mut T
where
    T: ImtStorageReader,
{
    type K = T::K;
    type V = T::V;

    fn get_node(&self, key: &Self::K) -> Option<ImtNode<Self::K, Self::V>> {
        T::get_node(*self, key)
    }

    fn get_ln_node(&self, key: &Self::K) -> Option<ImtNode<Self::K, Self::V>> {
        T::get_ln_node(*self, key)
    }

    fn get_hash(&self, level: u8, index: u64) -> Option<Hash256> {
        T::get_hash(*self, level, index)
    }

    fn get_size(&self) -> Option<u64> {
        T::get_size(*self)
    }

    fn get_root(&self) -> Option<Hash256> {
        T::get_root(*self)
    }
}

/// Trait for writing an imt to storage.
pub trait ImtStorageWriter: ImtStorageReader {
    /// Registers the given [ImtNode].
    fn set_node(&mut self, node: ImtNode<Self::K, Self::V>);

    /// Registers the given [Hash256].
    fn set_hash(&mut self, level: u8, index: u64, hash: Hash256);

    /// Registers the given imt size.
    fn set_size(&mut self, size: u64);

    /// Registers the given imt root.
    fn set_root(&mut self, root: Hash256);
}

impl<T> ImtStorageWriter for &mut T
where
    T: ImtStorageWriter,
{
    fn set_node(&mut self, node: ImtNode<Self::K, Self::V>) {
        T::set_node(*self, node);
    }

    fn set_hash(&mut self, level: u8, index: u64, hash: Hash256) {
        T::set_hash(*self, level, index, hash);
    }

    fn set_size(&mut self, size: u64) {
        T::set_size(*self, size);
    }

    fn set_root(&mut self, root: Hash256) {
        T::set_root(*self, root);
    }
}
