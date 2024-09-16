const VK_STORAGE_PREFIX: u8 = 5;

pub fn vk_storage_key(vk_hash: impl AsRef<[u8]>) -> Vec<u8> {
    let as_ref = vk_hash.as_ref();

    let mut v = Vec::with_capacity(as_ref.len() + 1);
    v[0] = VK_STORAGE_PREFIX;
    v[1..].copy_from_slice(as_ref);

    v
}
