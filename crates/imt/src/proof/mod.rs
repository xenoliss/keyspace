mod mutate;

pub mod insert;
pub mod update;

use crate::{node::ImtNode, Hash256, Hasher, NodeKey, NodeValue};

/// Computes the imt root.
fn imt_root_from_node<H, K, V>(
    hasher_factory: fn() -> H,
    size: u64,
    node: &ImtNode<K, V>,
    siblings: &Vec<Option<Hash256>>,
) -> Hash256
where
    H: Hasher,
    K: NodeKey,
    V: NodeValue,
{
    let mut hash = node.hash(hasher_factory());

    let mut index = node.index;
    for sibling in siblings {
        let node_hash = Some(hash);

        let (left, right) = if index % 2 == 0 {
            (&node_hash, sibling)
        } else {
            (sibling, &node_hash)
        };

        let mut hasher = hasher_factory();
        match (left, right) {
            (None, None) => unreachable!(),
            (None, Some(right)) => hasher.update(right),
            (Some(left), None) => hasher.update(left),
            (Some(left), Some(right)) => {
                hasher.update(left);
                hasher.update(right);
            }
        };

        hasher.finalize(&mut hash);

        index /= 2;
    }

    let mut hasher = hasher_factory();
    hasher.update(&hash);
    hasher.update(&size.to_be_bytes());
    hasher.finalize(&mut hash);

    hash
}

/// Returns `true` if the given `node` is part of the tree commited to in `root`.
fn node_exists<H, K, V>(
    hasher_factory: fn() -> H,
    root: &Hash256,
    size: u64,
    node: &ImtNode<K, V>,
    siblings: &Vec<Option<Hash256>>,
) -> bool
where
    H: Hasher,
    K: NodeKey,
    V: NodeValue,
{
    *root == imt_root_from_node(hasher_factory, size, node, siblings)
}
