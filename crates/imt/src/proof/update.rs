use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

use crate::{node::ImtNode, Hash256, Hasher, NodeKey, NodeValue};

use super::{imt_root_from_node, node_exists, Proof};

/// Update proof that can be verified for correctness.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ImtUpdate<K, V> {
    pub old_root: Hash256,
    pub size: u64,
    pub node: ImtNode<K, V>,
    pub node_siblings: Vec<Option<Hash256>>,
    pub new_value: V,
}

impl<H, K, V> Proof<H> for ImtUpdate<K, V>
where
    H: Hasher,
    K: NodeKey,
    V: NodeValue,
{
    /// Verifies the [ImtUpdate] and returns the new updated root.
    ///
    /// Before performing the update, the state is checked to make sure it is coherent.
    fn verify(&self, hasher_factory: fn() -> H, old_root: Hash256) -> Result<Hash256> {
        // Make sure the ImtMutate old_root matches the expected old_root.
        ensure!(old_root == self.old_root, "ImtMutate.old_root is stale");

        // Verify that the node to update is already in the imt.
        ensure!(
            node_exists(
                hasher_factory,
                &self.old_root,
                self.size,
                &self.node,
                &self.node_siblings
            ),
            "ImtMutate.node is not in the imt"
        );

        // Compute the new root from the updated node.
        let updated_node = ImtNode {
            value: self.new_value.clone(),
            ..self.node.clone()
        };

        Ok(imt_root_from_node(
            hasher_factory,
            self.size,
            &updated_node,
            &self.node_siblings,
        ))
    }
}

#[cfg(test)]
mod tests {
    use tiny_keccak::Keccak;

    use crate::{
        proof::Proof,
        storage::btree_imt_storage::BTreeImtStorage,
        tree::{Imt, ImtWriter},
    };

    #[test]
    fn test_verify_invalid_old_root() {
        // Instanciate an imt with a few nodes.
        let storage = BTreeImtStorage::default();
        let mut imt = Imt::writer(Keccak::v256, storage);
        imt.insert_node([1; 32], [42; 32]);
        imt.insert_node([2; 32], [42; 32]);
        imt.insert_node([3; 32], [42; 32]);

        // Create an ImtUpdate and call `.verify()` with a different `old_root`.
        let sut = imt.update_node([2; 32], [43; 32]);
        let res = sut.verify(Keccak::v256, [0xff; 32]);
        assert!(matches!(res, Err(e) if e.to_string() == "ImtMutate.old_root is stale"));
    }

    #[test]
    fn test_verify_node_does_not_exist() {
        // Instanciate an imt with a few nodes.
        let storage = BTreeImtStorage::default();
        let mut imt = Imt::writer(Keccak::v256, storage);
        imt.insert_node([1; 32], [42; 32]);
        imt.insert_node([2; 32], [42; 32]);
        imt.insert_node([3; 32], [42; 32]);

        // Create an ImtUpdate and call `.verify()` with a different `old_root`.
        let mut sut = imt.update_node([2; 32], [43; 32]);
        sut.node.key = [4; 32];
        let res = sut.verify(Keccak::v256, sut.old_root);
        assert!(matches!(res, Err(e) if e.to_string() == "ImtMutate.node is not in the imt"));
    }

    #[test]
    fn test_verify() {
        // Instanciate an imt with a few nodes.
        let storage = BTreeImtStorage::default();
        let mut imt = Imt::writer(Keccak::v256, storage);
        imt.insert_node([1; 32], [42; 32]);
        imt.insert_node([2; 32], [42; 32]);
        imt.insert_node([3; 32], [42; 32]);

        let keys = vec![[1; 32], [2; 32], [3; 32]];

        // Insert all the keys in the imt and ensure verifying the returned [ImtInsert] succeed.
        keys.into_iter().for_each(|node_key| {
            for i in 0..=255 {
                let sut = imt.update_node(node_key, [i; 32]);
                let res = sut.verify(Keccak::v256, sut.old_root);
                assert!(res.is_ok())
            }
        });
    }
}
