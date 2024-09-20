use tiny_keccak::{Hasher, Keccak};

pub mod batcher;
pub mod ecdsa_record;

pub type Hash256 = [u8; 32];

pub fn hash_storage(storage: &[u8]) -> Hash256 {
    // Compute the `storage_hash`: keccack(storage).
    let mut k = Keccak::v256();
    let mut storage_hash = [0; 32];
    k.update(storage);
    k.finalize(&mut storage_hash);
    storage_hash
}

pub fn keyspace_key_from_storage(vk_hash: &Hash256, storage: &[u8]) -> Hash256 {
    keyspace_key_from_storage_hash(vk_hash, &hash_storage(storage))
}

pub fn keyspace_key_from_storage_hash(vk_hash: &Hash256, storage_hash: &Hash256) -> Hash256 {
    // Compute the Keyspace key: keccack(storage_hash, vk_hash).
    let mut k = Keccak::v256();
    let mut key = [0; 32];
    k.update(storage_hash);
    k.update(vk_hash);
    k.finalize(&mut key);

    key
}
