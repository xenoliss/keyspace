use crate::{node::ImtNode, Hash256};

#[cfg(test)]
pub(crate) mod btree_imt_storage;

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
