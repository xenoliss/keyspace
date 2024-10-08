use anyhow::{anyhow, ensure, Result};
use serde::{Deserialize, Serialize};

use crate::{node::ImtNode, Hash256, Hasher, NodeKey, NodeValue};

use super::{imt_root_from_node, node_exists};

/// Insertion proof that can be verified for correctness.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct InsertProof<K, V> {
    // TODO: Should this be replaced by an [ExclusionProof]? This adds a bit of data duplication
    //       in trade off better code factorization.
    pub old_root: Hash256,
    pub old_size: u64,
    pub ln_node: ImtNode<K, V>,
    pub ln_siblings: Vec<Option<Hash256>>,

    pub node: ImtNode<K, V>,
    pub node_siblings: Vec<Option<Hash256>>,
    pub updated_ln_siblings: Vec<Option<Hash256>>,
}

impl<K, V> InsertProof<K, V>
where
    K: NodeKey,
    V: NodeValue,
{
    /// Verifies the [InsertProof] and returns the new updated root.
    ///
    /// Before performing the insertion, the state is checked to make sure it is coherent.
    pub fn verify<H: Hasher>(
        &self,
        hasher_factory: fn() -> H,
        old_root: Hash256,
    ) -> Result<Hash256> {
        // Make sure the ImtMutate old_root matches the expected old_root.
        ensure!(old_root == self.old_root, "ImtMutate.old_root is stale");

        // Verify that the provided ln node is valid.
        ensure!(
            self.is_valid_ln(hasher_factory),
            "ImtMutate.ln_node is invalid"
        );

        // Compute the updated root from the node and the updated ln node.
        let updated_ln = ImtNode {
            next_key: self.node.key.clone(),
            ..self.ln_node.clone()
        };

        let new_size = self
            .old_size
            .checked_add(1)
            .ok_or(anyhow!("imt size overflow"))?;

        let root_from_node =
            imt_root_from_node(hasher_factory, new_size, &self.node, &self.node_siblings);

        let root_from_updated_ln = imt_root_from_node(
            hasher_factory,
            new_size,
            &updated_ln,
            &self.updated_ln_siblings,
        );

        // Make sure both roots are equal.
        ensure!(
            root_from_node == root_from_updated_ln,
            "ImtMutate.updated_ln_siblings is invalid"
        );

        Ok(root_from_node)
    }

    /// Returns `true` if `self.ln_node` is a valid ln node for `self.node`.
    fn is_valid_ln<H: Hasher>(&self, hasher_factory: fn() -> H) -> bool {
        self.ln_node.is_ln_of(&self.node.key)
            && node_exists(
                &self.old_root,
                hasher_factory,
                self.old_size,
                &self.ln_node,
                &self.ln_siblings,
            )
    }
}

#[cfg(test)]
mod tests {
    use tiny_keccak::Keccak;

    use crate::{node::ImtNode, storage::btree_imt_storage::BTreeImtStorage, tree::Imt};

    #[test]
    fn test_verify_invalid_old_root() {
        // Instanciate an imt with a few nodes.
        let storage = BTreeImtStorage::default();
        let mut imt = Imt::writer(Keccak::v256, storage);
        imt.insert_node([1; 32], [42; 32])
            .expect("insert [1] failed");
        imt.insert_node([2; 32], [42; 32])
            .expect("insert [2] failed");
        imt.insert_node([3; 32], [42; 32])
            .expect("insert [3] failed");

        // Create an InsertProof and call `.verify()` with a different `old_root`.
        let sut = imt
            .insert_node([4; 32], [42; 32])
            .expect("insert [4] failed");
        let res = sut.verify(Keccak::v256, [0xff; 32]);
        assert!(matches!(res, Err(e) if e.to_string() == "ImtMutate.old_root is stale"));

        // Create an InsertProof and call `.verify()` with a different `old_root`.
        let old_root = imt.root();
        let mut sut = imt
            .insert_node([5; 32], [42; 32])
            .expect("insert [5] failed");
        sut.old_root = [0xff; 32];
        let res = sut.verify(Keccak::v256, old_root);
        assert!(matches!(res, Err(e) if e.to_string() == "ImtMutate.old_root is stale"));
    }

    #[test]
    fn test_verify_invalid_ln() {
        // Instanciate an imt with a few nodes.
        let storage = BTreeImtStorage::default();
        let mut imt = Imt::writer(Keccak::v256, storage);

        let insert_1 = imt
            .insert_node([1; 32], [42; 32])
            .expect("insert [1] failed");
        let insert_5 = imt
            .insert_node([5; 32], [42; 32])
            .expect("insert [5] failed");

        // Use a `ln_node` with an invalid `key`.
        let ln_node = insert_5.node;
        let mut sut = imt
            .insert_node([4; 32], [42; 32])
            .expect("insert [4] failed");
        sut.ln_node = ln_node;
        let res = sut.verify(Keccak::v256, sut.old_root);
        assert!(matches!(res, Err(e) if e.to_string() == "ImtMutate.ln_node is invalid"));

        // Use a `ln_node` with an invalid `next_key`.
        let ln_node = insert_1.node;
        let mut sut = imt
            .insert_node([6; 32], [42; 32])
            .expect("insert [6] failed");
        sut.ln_node = ln_node;
        let res = sut.verify(Keccak::v256, sut.old_root);
        assert!(matches!(res, Err(e) if e.to_string() == "ImtMutate.ln_node is invalid"));

        // Use a `ln_node` that is not in the tree.
        let ln_node = ImtNode {
            index: 42,
            key: [7; 32],
            value: [42; 32],
            next_key: [15; 32],
        };
        let mut sut = imt
            .insert_node([8; 32], [42; 32])
            .expect("insert [8] failed");
        sut.ln_node = ln_node;
        let res = sut.verify(Keccak::v256, sut.old_root);
        assert!(matches!(res, Err(e) if e.to_string() == "ImtMutate.ln_node is invalid"));
    }

    #[test]
    fn test_verify_invalid_updated_ln_siblings() {
        // Instanciate an imt with a few nodes.
        let storage = BTreeImtStorage::default();
        let mut imt = Imt::writer(Keccak::v256, storage);
        imt.insert_node([1; 32], [42; 32])
            .expect("insert [1] failed");
        imt.insert_node([2; 32], [42; 32])
            .expect("insert [2] failed");
        imt.insert_node([3; 32], [42; 32])
            .expect("insert [3] failed");

        // Create an InsertProof, but update `updated_ln_siblings` to be incorrect, resulting in an
        // imt root that differs from the one computed from the inserted node.
        let mut sut = imt
            .insert_node([4; 32], [42; 32])
            .expect("insert [4] failed");
        sut.updated_ln_siblings[0] = Some([0xff; 32]);
        let res = sut.verify(Keccak::v256, sut.old_root);
        assert!(
            matches!(res, Err(e) if e.to_string() == "ImtMutate.updated_ln_siblings is invalid")
        );
    }

    #[test]
    fn test_verify() {
        let storage = BTreeImtStorage::default();
        let mut imt = Imt::writer(Keccak::v256, storage);
        let keys = vec![
            [1; 32], [2; 32], [3; 32], [4; 32], [5; 32], [10; 32], [15; 32], [11; 32], [20; 32],
            [16; 32], [25; 32],
        ];

        // Insert all the keys in the imt and ensure verifying the returned [InsertProof] succeed.
        keys.into_iter().for_each(|node_key| {
            let sut = imt
                .insert_node(node_key, [42; 32])
                .expect("insert [42] failed");
            let res = sut.verify(Keccak::v256, sut.old_root);
            assert!(res.is_ok())
        });
    }
}
