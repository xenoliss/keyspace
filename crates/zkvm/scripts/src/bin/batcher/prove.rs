use std::{fs::read_dir, path::PathBuf};

use k256::{
    elliptic_curve::ff::derive::bitvec::vec,
    sha2::{Digest, Sha256},
};
use keyspace_programs_lib::{
    batcher::{inputs::Inputs, proof::sp1::SP1Proof, transaction::Transaction},
    Hash256,
};
use sp1_sdk::{HashableKey, ProverClient, SP1Stdin};
use tiny_keccak::Keccak;

use keyspace_imt::tree::Imt;
use keyspace_state_manager::storage::btree::BTreeStorage;

const ELF: &[u8] = include_bytes!("../../../../batcher/elf/riscv32im-succinct-zkvm-elf");
const ECDSA_RECORD_ELF: &[u8] =
    include_bytes!("../../../../ecdsa-record/elf/riscv32im-succinct-zkvm-elf");

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    // Setup the prover client.
    let client = ProverClient::new();

    // Setup the program.
    let (batcher_pk, _) = client.setup(ELF);
    let (_, record_vk) = client.setup(ECDSA_RECORD_ELF);
    let record_vk_hash = record_vk.hash_bytes();
    let forced_vk_hash = read_forced_vk_hash();

    let storage = BTreeStorage::default();
    let mut imt = Imt::writer(Keccak::v256, storage);
    let old_root = imt.root();

    let mut txs_commitment = [0; 32];
    let txs = read_dir("proofs/sp1/")
        .expect("failed to read proofs/sp1/")
        .map(|entry| {
            let keyspace_id = vec![0; 32];
            let keyspace_value = vec![0_u8; 32];
            let storage_hash = [0; 32];

            let imt_mutate_proof = imt
                .set_node(keyspace_id, keyspace_value)
                .expect("failed to set node");

            let tx = Transaction::sequenced(
                todo!(),
                txs_commitment,
                SP1Proof::new(record_vk_hash, forced_vk_hash, storage_hash),
            );

            txs_commitment = tx
                .commitment(Some(txs_commitment))
                .expect("failed to compute the transaction commitment");

            tx
        })
        .collect::<Vec<_>>();

    let inputs = Inputs {
        old_root,
        new_root: imt.root(),
        txs_commitment,
        txs,
    };

    // Generate the proof for it.
    let mut stdin = SP1Stdin::new();
    stdin.write(&inputs);
    client
        .prove(&batcher_pk, stdin)
        .plonk()
        .run()
        .expect("batcher proving failed");
}

/// Reads the constant v2.0.0 PLONK vk and returns its [Sha256] hash.
fn read_forced_vk_hash() -> Hash256 {
    let plonk_vk = PathBuf::from(std::env::var("HOME").unwrap())
        .join(".sp1")
        .join("circuits")
        .join("v2.0.0")
        .join("plonk_vk.bin");

    let vk = std::fs::read(plonk_vk).expect("failed to read plonk VK");
    Sha256::digest(&vk).into()
}
