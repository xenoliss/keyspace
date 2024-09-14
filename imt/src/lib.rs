#![feature(trait_alias)]
#![feature(btree_cursors)]

use node::IMTNode;
use storage::IMTStorage;
use tiny_keccak::{Hasher, Keccak};
use zkvm::{insert::IMTInsert, update::IMTUpdate};

mod node;

pub mod storage;
pub mod zkvm;

type Hash256 = [u8; 32];

pub trait NodeKey = Default + Clone + Copy + Ord + AsRef<[u8]>;
pub trait NodeValue = Default + Clone + Copy + AsRef<[u8]>;

#[derive(Debug, Clone)]
pub struct IMT<H, S, K, V> {
    hasher_factory: fn() -> H,
    storage: S,

    _phantom_data_k: std::marker::PhantomData<K>,
    _phantom_data_v: std::marker::PhantomData<V>,
}

impl<H, S, K, V> IMT<H, S, K, V>
where
    H: Hasher,
    S: IMTStorage<K, V>,
    K: NodeKey,
    V: NodeValue,
{
    /// Instanciate a new IMT linked to the given `storage`.
    pub fn new(hasher_factory: fn() -> H, storage: S) -> Self {
        let size = storage.get_size();

        let mut imt = Self {
            hasher_factory,
            storage,

            _phantom_data_k: std::marker::PhantomData,
            _phantom_data_v: std::marker::PhantomData,
        };

        // If the tree was empty in storage, insert the 0 node.
        if size.is_none() {
            let init_node = IMTNode {
                index: Default::default(),
                key: Default::default(),
                value: Default::default(),
                next_key: Default::default(),
            };

            // Save the size (1) in storage and set the node.
            imt.storage.set_size(1);

            imt._set_node(0, init_node);
        }

        imt
    }

    /// Return the IMT size (including the 0 node)
    pub fn size(&self) -> u64 {
        self.storage.get_size().unwrap_or(0)
    }

    /// Return the IMT root
    pub fn root(&self) -> Hash256 {
        self.storage.get_root().unwrap_or_default()
    }

    /// Inserts a new (key; value) in the IMT.
    ///
    /// Returns the corresponding `IMTInsert` to use for zkVM verification.
    pub fn insert_node(&mut self, key: K, value: V) -> IMTInsert<K, V> {
        // Ensure key does not already exist in the tree.
        assert!(self.storage.get_node(&key).is_none(), "node already exists");

        let old_size = self.size();
        let old_root = self.root();
        let old_depth = depth(old_size);

        // Get the ln node.
        let mut ln_node = self.low_nullifier(&key);
        let ln_siblings = self._siblings(old_depth, &ln_node);

        // Create the new node.
        let node = IMTNode {
            index: old_size,
            key,
            value,
            next_key: ln_node.next_key,
        };

        // Update the ln node and refresh the tree.
        ln_node.next_key = key;
        self._set_node(old_depth, ln_node.clone());

        // Increment the IMT size.
        // NOTE: Must be done prior to inserting the new node.
        let new_size = old_size + 1;
        self.storage.set_size(new_size);

        // Insert the new node and refresh the tree.
        let new_depth = depth(new_size);
        let node_siblings = self._set_node(new_depth, node.clone());
        let updated_ln_siblings = self._siblings(new_depth, &ln_node);

        // NOTE: Reset the `ln_node.next_key` value before using it in IMTMutate::insert.
        // TODO: Improve this to avoid doing this hacky reset.
        ln_node.next_key = node.next_key;

        // Return the IMTMutate insertion to use for proving.
        IMTInsert {
            old_root,
            old_size,
            ln_node,
            ln_siblings,
            node,
            node_siblings,
            updated_ln_siblings,
        }
    }

    /// Updates the given `key` to `value` in the IMT.
    ///
    /// Returns the corresponding `IMTUpdate` to use for zkVM verification.
    pub fn update_node(&mut self, key: K, value: V) -> IMTUpdate<K, V> {
        let old_root = self.root();
        let size = self.size();

        let mut node = self.storage.get_node(&key).expect("node does not exist");
        let old_node = node.clone();
        node.value = value;

        let node_siblings = self._set_node(depth(size), node);

        IMTUpdate {
            old_root,
            size,
            node: old_node,
            node_siblings,
            new_value: value,
        }
    }

    /// Returns the Low Nulifier node for the given `node_key`.
    pub fn low_nullifier(&self, node_key: &K) -> IMTNode<K, V> {
        self.storage
            .get_ln_node(node_key)
            .expect("failed to found ln node")
    }

    /// Returns the list of siblings for the given `node`.
    fn _siblings(&self, depth: u8, node: &IMTNode<K, V>) -> Vec<Option<Hash256>> {
        let mut siblings = Vec::with_capacity(depth as _);
        let mut index = node.index;

        for level in 0..depth {
            let sibling_index = if index % 2 == 0 { index + 1 } else { index - 1 };
            let sibling_hash = self.storage.get_hash(level, sibling_index);

            siblings.push(sibling_hash);
            index /= 2;
        }

        siblings
    }

    /// Sets the given `node` in the IMT.
    /// This also refreshes the list of hashes based on the provided `node` and as well as the IMT root.
    ///
    /// Returns the updated list of siblings for the given `node`.
    fn _set_node(&mut self, depth: u8, node: IMTNode<K, V>) -> Vec<Option<Hash256>> {
        let mut index = node.index;
        let hasher_factory = self.hasher_factory;
        let mut hash = node.hash(hasher_factory());

        self.storage.set_node(node);

        // Cache the node hash.
        self.storage.set_hash(0, index, hash);

        // Climb up the tree and refresh the hashes.
        let mut siblings = Vec::with_capacity(depth as _);
        for level in 0..depth {
            let sibling_index = if index % 2 == 0 { index + 1 } else { index - 1 };
            let sibling_hash = self.storage.get_hash(level, sibling_index);
            siblings.push(sibling_hash);

            let (left, right) = if index % 2 == 0 {
                (Some(hash), sibling_hash)
            } else {
                (sibling_hash, Some(hash))
            };

            let mut hasher = hasher_factory();
            match (left, right) {
                (None, None) => unreachable!(),
                (None, Some(right)) => hasher.update(&right),
                (Some(left), None) => hasher.update(&left),
                (Some(left), Some(right)) => {
                    hasher.update(&left);
                    hasher.update(&right);
                }
            };

            hasher.finalize(&mut hash);

            index /= 2;

            self.storage.set_hash(level + 1, index, hash);
        }

        // Refreshes the IMT root.
        self._refresh_root(depth);

        siblings
    }

    /// Returns the IMT root (including its size).
    fn _refresh_root(&mut self, depth: u8) {
        let size = self.size();

        // TODO: Is it always safe to unwrap_or_default here?
        let root = self.storage.get_hash(depth, 0).unwrap_or_default();

        let mut root_with_size = [0; 32];
        let mut k = Keccak::v256();
        k.update(&root);
        k.update(&size.to_be_bytes());
        k.finalize(&mut root_with_size);

        self.storage.set_root(root_with_size);
    }
}

fn depth(size: u64) -> u8 {
    let depth = (u64::BITS - size.leading_zeros() - 1) as u8;
    if size == (1_u64 << depth) {
        depth
    } else {
        depth + 1
    }
}
