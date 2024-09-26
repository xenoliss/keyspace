use tiny_keccak::{Hasher, Keccak};

pub mod batcher;
pub mod ecdsa_record;

pub type Hash256 = [u8; 32];

/// Computes the authorization hash.
pub fn authorization_hash(authorization_data: &[u8]) -> Hash256 {
    let mut k = Keccak::v256();
    let mut authorization_hash = [0; 32];
    k.update(authorization_data);
    k.finalize(&mut authorization_hash);

    authorization_hash
}

/// Computes the storage hash.
pub fn storage_hash(authorization_hash: &Hash256, sidecar_hash: &Hash256) -> Hash256 {
    let mut k = Keccak::v256();
    let mut storage_hash = [0; 32];
    k.update(authorization_hash);
    k.update(sidecar_hash);
    k.finalize(&mut storage_hash);

    storage_hash
}

/// Computes the authorization key.
pub fn authorization_key(record_vk_hash: &Hash256, forced_vk_hash: Option<&Hash256>) -> Hash256 {
    match forced_vk_hash {
        Some(forced_vk_hash) => {
            let mut k = Keccak::v256();
            let mut authorization_key = [0; 32];
            k.update(record_vk_hash);
            k.update(forced_vk_hash);
            k.finalize(&mut authorization_key);

            authorization_key
        }
        None => *record_vk_hash,
    }
}

/// Computes the KeySpace value.
pub fn keyspace_value(authorization_key: &Hash256, storage_hash: &Hash256) -> Hash256 {
    let mut k = Keccak::v256();
    let mut value = [0; 32];
    k.update(authorization_key);
    k.update(storage_hash);
    k.finalize(&mut value);

    value
}

/// Performs a full KeySpace value recovery.
pub fn recover_keyspace_value(
    authorization_key: &Hash256,
    authorization_data: &[u8],
    sidecar_hash: &Hash256,
) -> Hash256 {
    let authorization_hash = authorization_hash(authorization_data);
    let storage_hash = storage_hash(&authorization_hash, sidecar_hash);

    keyspace_value(authorization_key, &storage_hash)
}
