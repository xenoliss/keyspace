// The storage keys must be prefixed to ensure their correct ordering in the storage.
// Specifically the storage has a lexicographical order meaning "0" < "0000001" < "1".
//
// The node keys MUST be stored first followed by others elements to store. This is
// necessary as nodes MIGTH be fetched from the storage using `get_lt` that returns the
// key immediately preceding the given one. For this `NODE_STORAGE_PREFIX` MUST be the
// lowest key prefix.

const NODE_STORAGE_PREFIX: u8 = 0;
const HASH_STORAGE_PREFIX: u8 = 1;
const SIZE_STORAGE_PREFIX: u8 = 2;
const ROOT_STORAGE_PREFIX: u8 = 3;
const VK_STORAGE_PREFIX: u8 = 4;

/// Returns the node storage key to use for persistence.
pub fn node_storage_key(key: impl AsRef<[u8]>) -> Vec<u8> {
    let as_ref = key.as_ref();

    let mut v = vec![0; as_ref.len() + 1];
    v[0] = NODE_STORAGE_PREFIX;
    v[1..].copy_from_slice(as_ref);

    v
}

/// Returns the vk storage key to use for persistence.
pub fn vk_storage_key(vk_hash: impl AsRef<[u8]>) -> Vec<u8> {
    let as_ref = vk_hash.as_ref();

    let mut v = vec![0; as_ref.len() + 1];
    v[0] = VK_STORAGE_PREFIX;
    v[1..].copy_from_slice(as_ref);

    v
}

/// Returns the hash storage key to use for persistence.
pub fn hash_storage_key(level: u8, index: u64) -> Vec<u8> {
    let mut v = vec![0; 1 + 1 + 8];
    v[0] = HASH_STORAGE_PREFIX;
    v[1] = level;
    v[2..].copy_from_slice(&index.to_be_bytes());

    v
}

/// Returns the size storage key to use for persistence.
pub fn size_storage_key() -> Vec<u8> {
    vec![SIZE_STORAGE_PREFIX]
}

/// Returns the root storage key to use for persistence.
pub fn root_storage_key() -> Vec<u8> {
    vec![ROOT_STORAGE_PREFIX]
}
