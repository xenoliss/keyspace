use tiny_keccak::{Hasher, Keccak};

use crate::{
    node::ImtNode,
    proof::{insert::ImtInsert, mutate::ImtMutate, update::ImtUpdate},
    storage::{ImtStorageReader, ImtStorageWriter},
    Hash256, NodeKey, NodeValue,
};

// TODO: implement this and move it in the proof module.
pub struct InclusionProof<K, V> {
    pub root: Hash256,
    pub node: ImtNode<K, V>,
    pub siblings: Vec<Option<Hash256>>,
}

// TODO: implement this and move it in the proof module.
pub struct ExclusionProof<K, V> {
    pub root: Hash256,
    pub ln_node: ImtNode<K, V>,
    pub siblings: Vec<Option<Hash256>>,
}

/// A trait for reading the imt state and generating proofs.
///
/// This trait is not supposed to be implemented outside of this module.
pub trait ImtReader {
    type K;
    type V;

    /// Returns the imt size (including the 0 node).
    fn size(&self) -> u64;

    /// Returns the imt root (including the size).
    fn root(&self) -> Hash256;

    /// Generates an inclusion proof.
    fn inclusion_proof(&self) -> InclusionProof<Self::K, Self::V>;

    /// Generates an exclusion proof.
    fn exclusion_proof(&self) -> ExclusionProof<Self::K, Self::V>;
}

/// A trait for writing to the imt state.
///
/// This trait is not supposed to be implemented outside of this module.
pub trait ImtWriter: ImtReader {
    /// Sets a (key; value) pair in the imt and returns the corresponding [ImtMutate] proof.
    fn set_node(&mut self, key: Self::K, value: Self::V) -> ImtMutate<Self::K, Self::V>;

    /// Inserts a new (key; value) in the imt and returns the corresponding [ImtInsert] proof.
    fn insert_node(&mut self, key: Self::K, value: Self::V) -> ImtInsert<Self::K, Self::V>;

    /// Updates the given `key` to `value` in the imt and returns the corresponding [ImtUpdate] proof.
    fn update_node(&mut self, key: Self::K, value: Self::V) -> ImtUpdate<Self::K, Self::V>;
}

/// And Indexed Merkle Tree generic over its hash function (`H`), its storage layer (`S`) and its
/// keys and values types (`K` and `V`).
#[derive(Debug, Clone)]
pub struct Imt<H, S, K, V> {
    hasher_factory: fn() -> H,
    storage: S,

    _phantom_data_k: std::marker::PhantomData<K>,
    _phantom_data_v: std::marker::PhantomData<V>,
}

impl<H, S, K, V> Imt<H, S, K, V>
where
    S: ImtStorageReader<K = K, V = V>,
{
    /// Creates a new imt that only provides read access.
    ///
    /// Panics if the [Imt::storage] is empty.
    pub fn reader(hasher_factory: fn() -> H, storage: S) -> Self {
        let size = storage.get_size();

        let imt = Self {
            hasher_factory,
            storage,

            _phantom_data_k: std::marker::PhantomData,
            _phantom_data_v: std::marker::PhantomData,
        };

        size.and(Some(imt)).expect("imt is empty")
    }

    /// Returns the Low Nulifier node for the given `node_key`.
    fn low_nullifier(&self, node_key: &K) -> ImtNode<K, V> {
        self.storage
            .get_ln_node(node_key)
            .expect("failed to found ln node")
    }

    /// Returns the list of siblings for the given `node`.
    fn siblings(&self, depth: u8, node: &ImtNode<K, V>) -> Vec<Option<Hash256>> {
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
}

impl<H, S, K, V> Imt<H, S, K, V>
where
    H: Hasher,
    S: ImtStorageWriter<K = K, V = V>,
    K: NodeKey,
    V: NodeValue,
{
    /// Creates a new imt that provides read and write accesses.
    pub fn writer(hasher_factory: fn() -> H, storage: S) -> Self {
        let size = storage.get_size();

        let mut imt = Self {
            hasher_factory,
            storage,

            _phantom_data_k: std::marker::PhantomData,
            _phantom_data_v: std::marker::PhantomData,
        };

        // If the tree was empty in storage, insert the 0 node.
        if size.is_none() {
            let init_node = ImtNode {
                index: Default::default(),
                key: Default::default(),
                value: Default::default(),
                next_key: Default::default(),
            };

            // Save the size (1) in storage and set the node.
            imt.storage.set_size(1);
            imt.set_node(0, init_node);
        }

        imt
    }

    /// Sets the given [ImtNode] in the imt and returns the updated list of siblings for the given `node`.
    ///
    /// This refreshes the list of hashes based on the provided `node` and as well as the imt root.
    fn set_node(&mut self, depth: u8, node: ImtNode<K, V>) -> Vec<Option<Hash256>> {
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

        // Refreshes the imt root.
        self.refresh_root(depth);

        siblings
    }

    /// Refreshes the imt root.
    fn refresh_root(&mut self, depth: u8) {
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

impl<H, S, K, V> ImtReader for Imt<H, S, K, V>
where
    S: ImtStorageReader,
{
    type K = K;
    type V = V;

    fn root(&self) -> Hash256 {
        self.storage.get_root().unwrap_or_default()
    }

    fn size(&self) -> u64 {
        self.storage.get_size().unwrap_or(0)
    }

    fn inclusion_proof(&self) -> InclusionProof<Self::K, Self::V> {
        todo!("implement inclusion proofs")
    }

    fn exclusion_proof(&self) -> ExclusionProof<Self::K, Self::V> {
        todo!("implement exclusion proofs")
    }
}

impl<H, S, K, V> ImtWriter for Imt<H, S, K, V>
where
    H: Hasher,
    S: ImtStorageWriter<K = K, V = V>,
    K: NodeKey,
    V: NodeValue,
{
    fn set_node(&mut self, key: Self::K, value: Self::V) -> ImtMutate<Self::K, Self::V> {
        if self.storage.get_node(&key).is_some() {
            ImtMutate::Update(self.update_node(key, value))
        } else {
            ImtMutate::Insert(self.insert_node(key, value))
        }
    }

    fn insert_node(&mut self, key: Self::K, value: Self::V) -> ImtInsert<Self::K, Self::V> {
        // Ensure key does not already exist in the tree.
        assert!(self.storage.get_node(&key).is_none(), "node already exists");

        let old_size = self.size();
        let old_root = self.root();
        let old_depth = depth(old_size);

        // Get the ln node.
        let mut ln_node = self.low_nullifier(&key);
        let ln_siblings = self.siblings(old_depth, &ln_node);

        // Create the new node.
        let node = ImtNode {
            index: old_size,
            key: key.clone(),
            value,
            next_key: ln_node.next_key,
        };

        // Update the ln node and refresh the tree.
        ln_node.next_key = key;
        self.set_node(old_depth, ln_node.clone());

        // Increment the imt size.
        // NOTE: Must be done prior to inserting the new node.
        let new_size = old_size + 1;
        self.storage.set_size(new_size);

        // Insert the new node and refresh the tree.
        let new_depth = depth(new_size);
        let node_siblings = self.set_node(new_depth, node.clone());
        let updated_ln_siblings = self.siblings(new_depth, &ln_node);

        // NOTE: Reset the `ln_node.next_key` value before using it in ImtMutate::insert.
        // TODO: Improve this to avoid doing this hacky reset.
        ln_node.next_key = node.next_key.clone();

        // Return the ImtMutate insertion to use for proving.
        ImtInsert {
            old_root,
            old_size,
            ln_node,
            ln_siblings,
            node,
            node_siblings,
            updated_ln_siblings,
        }
    }

    fn update_node(&mut self, key: Self::K, value: Self::V) -> ImtUpdate<Self::K, Self::V> {
        let old_root = self.root();
        let size = self.size();

        let mut node = self.storage.get_node(&key).expect("node does not exist");
        let old_node = node.clone();
        node.value = value.clone();

        let node_siblings = self.set_node(depth(size), node);

        ImtUpdate {
            old_root,
            size,
            node: old_node,
            node_siblings,
            new_value: value,
        }
    }
}

/// Computes the depth of the tree based on its provided `size`.
fn depth(size: u64) -> u8 {
    let depth = (u64::BITS - size.leading_zeros() - 1) as u8;
    if size == (1_u64 << depth) {
        depth
    } else {
        depth + 1
    }
}
