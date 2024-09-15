use crate::{node::IMTNode, Hash256};

pub(crate) mod btree_imt_storage;

pub trait IMTStorage {
    type K;
    type V;

    fn get_node(&self, key: &Self::K) -> Option<IMTNode<Self::K, Self::V>>;
    fn get_ln_node(&self, key: &Self::K) -> Option<IMTNode<Self::K, Self::V>>;
    fn get_hash(&self, level: u8, index: u64) -> Option<Hash256>;
    fn get_size(&self) -> Option<u64>;
    fn get_root(&self) -> Option<Hash256>;

    fn set_node(&mut self, node: IMTNode<Self::K, Self::V>);
    fn set_hash(&mut self, level: u8, index: u64, hash: Hash256);
    fn set_size(&mut self, size: u64);
    fn set_root(&mut self, root: Hash256);
}
