use std::collections::{BTreeMap, HashMap};

use crate::{node::ImtNode, Hash256, NodeKey, NodeValue};

use super::{ImtStorageReader, ImtStorageWriter};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BTreeImtStorage<NodeK, NodeV> {
    root: Option<Hash256>,
    size: Option<u64>,
    nodes: BTreeMap<NodeK, ImtNode<NodeK, NodeV>>,
    hashes: HashMap<u8, HashMap<u64, Hash256>>,
}

impl<NodeK, NodeV> ImtStorageReader for BTreeImtStorage<NodeK, NodeV>
where
    NodeK: NodeKey,
    NodeV: NodeValue,
{
    type NodeK = NodeK;
    type NodeV = NodeV;

    fn get_node(&self, key: &NodeK) -> Option<ImtNode<NodeK, NodeV>> {
        self.nodes.get(key).cloned()
    }

    fn get_ln_node(&self, key: &NodeK) -> Option<ImtNode<NodeK, NodeV>> {
        self.nodes
            .range(..key)
            .next_back()
            .map(|(_, ln)| ln)
            .cloned()
    }

    fn get_hash(&self, level: u8, index: u64) -> Option<Hash256> {
        self.hashes.get(&level)?.get(&index).cloned()
    }

    fn get_size(&self) -> Option<u64> {
        self.size
    }

    fn get_root(&self) -> Option<Hash256> {
        self.root
    }
}

impl<NodeK, NodeV> ImtStorageWriter for BTreeImtStorage<NodeK, NodeV>
where
    NodeK: NodeKey,
    NodeV: NodeValue,
{
    fn set_node(&mut self, node: ImtNode<NodeK, NodeV>) {
        self.nodes.insert(node.key.clone(), node);
    }

    fn set_hash(&mut self, level: u8, index: u64, hash: Hash256) {
        self.hashes.entry(level).or_default().insert(index, hash);
    }

    fn set_size(&mut self, size: u64) {
        self.size = Some(size)
    }

    fn set_root(&mut self, root: Hash256) {
        self.root = Some(root)
    }
}
