pub mod insert;
pub mod mutate;
pub mod update;

use anyhow::Result;

use crate::{node::ImtNode, Hash256, Hasher, NodeKey, NodeValue};

/// Trait that provides a method to verify the validity of an imt proof.
pub trait Proof<H> {
    /// Verifies the imt proof and returns the new imt root if the verification passed, else an error is returned.
    fn verify(&self, hasher_factory: fn() -> H, old_root: Hash256) -> Result<Hash256>;
}

impl<H, T> Proof<H> for &T
where
    T: Proof<H>,
{
    fn verify(&self, hasher_factory: fn() -> H, old_root: Hash256) -> Result<Hash256> {
        T::verify(*self, hasher_factory, old_root)
    }
}

impl<H, T> Proof<H> for &mut T
where
    T: Proof<H>,
{
    fn verify(&self, hasher_factory: fn() -> H, old_root: Hash256) -> Result<Hash256> {
        T::verify(*self, hasher_factory, old_root)
    }
}

impl<H, T> Proof<H> for Box<T>
where
    T: Proof<H>,
{
    fn verify(&self, hasher_factory: fn() -> H, old_root: Hash256) -> Result<Hash256> {
        T::verify(self, hasher_factory, old_root)
    }
}

/// Computes the imt root from the given [ImtNode] and its siblings.
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

/// Returns `true` if the given [ImtNode] is part of the tree commited to in `root`.
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
