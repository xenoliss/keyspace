use anyhow::{ensure, Ok, Result};
use serde::{Deserialize, Serialize};
use tiny_keccak::Hasher;

use crate::{node::ImtNode, Hash256, NodeKey, NodeValue};

use super::node_exists;

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct InclusionProof<K, V> {
    pub root: Hash256,
    pub size: u64,
    pub node: ImtNode<K, V>,
    pub siblings: Vec<Option<Hash256>>,
}

impl<K, V> InclusionProof<K, V>
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
                &self.node,
                &self.siblings,
            ),
            "node does not exist"
        );

        Ok(())
    }
}
