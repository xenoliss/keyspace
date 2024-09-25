use super::keys::{hash_storage_key, node_storage_key, root_storage_key, size_storage_key};
use crate::storage::{
    btree::{BTreeStorage, BTreeTransaction},
    StorageReader, StorageWriter,
};
use keyspace_imt::{
    node::ImtNode,
    storage::{ImtStorageReader, ImtStorageWriter},
    Hash256,
};

impl ImtStorageReader for BTreeStorage<Vec<u8>, Vec<u8>> {
    type NodeK = [u8; 32];
    type NodeV = [u8; 32];

    fn get_node(&self, key: &Self::NodeK) -> Option<ImtNode<Self::NodeK, Self::NodeV>> {
        self.get(&node_storage_key(key))
            .map(|v| bincode::deserialize(v.as_ref()).expect("failed to deserialize imt node"))
    }

    fn get_ln_node(&self, key: &Self::NodeK) -> Option<ImtNode<Self::NodeK, Self::NodeV>> {
        self.get_lt(&node_storage_key(key)).map(|(_k, v)| {
            bincode::deserialize(v.as_ref()).expect("failed to deserialize imt node")
        })
    }

    fn get_hash(&self, level: u8, index: u64) -> Option<Hash256> {
        self.get(&hash_storage_key(level, index))
            .map(|v| v.try_into().expect("failed to deserialize hash"))
    }

    fn get_size(&self) -> Option<u64> {
        self.get(&size_storage_key())
            .map(|v| u64::from_le_bytes(v.try_into().expect("failed to deserialize size")))
    }

    fn get_root(&self) -> Option<Hash256> {
        self.get(&root_storage_key())
            .map(|v| v.try_into().expect("failed to deserialize root"))
    }
}

impl ImtStorageWriter for BTreeStorage<Vec<u8>, Vec<u8>> {
    fn set_node(&mut self, node: ImtNode<Self::NodeK, Self::NodeV>) {
        self.set(
            node_storage_key(node.key),
            bincode::serialize(&node).expect("failed to serialize node"),
        );
    }

    fn set_hash(&mut self, level: u8, index: u64, hash: Hash256) {
        self.set(hash_storage_key(level, index), hash.to_vec());
    }

    fn set_size(&mut self, size: u64) {
        self.set(size_storage_key(), size.to_le_bytes().to_vec());
    }

    fn set_root(&mut self, root: Hash256) {
        self.set(root_storage_key(), root.to_vec());
    }
}

impl<'a> ImtStorageReader for BTreeTransaction<'a, Vec<u8>, Vec<u8>> {
    type NodeK = [u8; 32];
    type NodeV = [u8; 32];

    fn get_node(&self, key: &Self::NodeK) -> Option<ImtNode<Self::NodeK, Self::NodeV>> {
        self.get(&node_storage_key(key))
            .map(|v| bincode::deserialize(v.as_ref()).expect("failed to deserialize imt node"))
    }

    fn get_ln_node(&self, key: &Self::NodeK) -> Option<ImtNode<Self::NodeK, Self::NodeV>> {
        self.get_lt(&node_storage_key(key)).map(|(_k, v)| {
            bincode::deserialize(v.as_ref()).expect("failed to deserialize imt node")
        })
    }

    fn get_hash(&self, level: u8, index: u64) -> Option<Hash256> {
        self.get(&hash_storage_key(level, index))
            .map(|v| v.try_into().expect("failed to deserialize hash"))
    }

    fn get_size(&self) -> Option<u64> {
        self.get(&size_storage_key())
            .map(|v| u64::from_le_bytes(v.try_into().expect("failed to deserialize size")))
    }

    fn get_root(&self) -> Option<Hash256> {
        self.get(&root_storage_key())
            .map(|v| v.try_into().expect("failed to deserialize root"))
    }
}

impl<'a> ImtStorageWriter for BTreeTransaction<'a, Vec<u8>, Vec<u8>> {
    fn set_node(&mut self, node: ImtNode<Self::NodeK, Self::NodeV>) {
        self.set(
            node_storage_key(node.key.as_ref()),
            bincode::serialize(&node).expect("failed to serialize node"),
        );
    }

    fn set_hash(&mut self, level: u8, index: u64, hash: Hash256) {
        self.set(hash_storage_key(level, index), hash.to_vec());
    }

    fn set_size(&mut self, size: u64) {
        self.set(size_storage_key(), size.to_le_bytes().to_vec());
    }

    fn set_root(&mut self, root: Hash256) {
        self.set(root_storage_key(), root.to_vec());
    }
}
