use std::{
    fs::{read_to_string, File},
    io::Write,
    path::{Path, PathBuf},
};

use k256::sha2::{Digest, Sha256};
use keyspace_programs_lib::Hash256;
use serde::{Deserialize, Serialize};
use sp1_sdk::SP1ProofWithPublicValues;

#[derive(Serialize, Deserialize)]
struct StoredProof {
    sp1_proof: SP1ProofWithPublicValues,
    storage_hash: Hash256,
}

pub fn save_record_proof(
    sp1_proof: SP1ProofWithPublicValues,
    storage_hash: Hash256,
    path: impl AsRef<Path>,
) {
    let stored_proof = StoredProof {
        sp1_proof,
        storage_hash,
    };

    let mut file = File::create(path).expect("failed to create file");
    file.write_all(
        serde_json::to_string(&stored_proof)
            .expect("faield to serialize stored record proof")
            .as_bytes(),
    )
    .expect("failed to save proof in storage");
}

pub fn load_record_proof(path: impl AsRef<Path>) -> (SP1ProofWithPublicValues, Hash256) {
    let content = read_to_string(&path).expect("failed to read proof file");
    let stored_proof: StoredProof =
        serde_json::from_str(&content).expect("failed to deserialize stored record proof");

    (stored_proof.sp1_proof, stored_proof.storage_hash)
}

/// Reads the constant v3.0.0-rc1 PLONK vk and returns its [Sha256] hash.
pub fn read_forced_vk_hash() -> Hash256 {
    let plonk_vk = PathBuf::from(std::env::var("HOME").unwrap())
        .join(".sp1")
        .join("circuits")
        .join("v3.0.0-rc1")
        .join("plonk_vk.bin");

    let vk = std::fs::read(plonk_vk).expect("failed to read plonk VK");
    Sha256::digest(&vk).into()
}
