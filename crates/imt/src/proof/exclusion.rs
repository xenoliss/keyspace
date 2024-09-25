use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use tiny_keccak::Hasher;

use crate::{node::ImtNode, proof::node_exists, Hash256, NodeKey, NodeValue};

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ExclusionProof<K, V> {
    pub root: Hash256,
    pub size: u64,
    pub ln_node: ImtNode<K, V>,
    pub ln_siblings: Vec<Option<Hash256>>,
    pub node_key: K,
}

impl<K, V> ExclusionProof<K, V>
where
    K: NodeKey,
    V: NodeValue,
{
    pub fn verify<H: Hasher>(&self, hasher_factory: fn() -> H) -> Result<()> {
        ensure!(
            node_exists(
                &self.root,
                hasher_factory,
                self.size,
                &self.ln_node,
                &self.ln_siblings,
            ),
            "ln node does not exist"
        );

        ensure!(
            self.ln_node.is_ln_of(&self.node_key),
            "ln node is invalid for the given key"
        );

        Ok(())
    }
}
