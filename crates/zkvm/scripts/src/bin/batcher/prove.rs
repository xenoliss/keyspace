use hex::ToHex;
use sp1_sdk::{
    install::try_install_circuit_artifacts, HashableKey, ProverClient, SP1Proof, SP1Stdin,
};
use std::fs::read_dir;
use tiny_keccak::Keccak;

use keyspace_imt::tree::Imt;
use keyspace_programs_lib::batcher::{
    inputs::Inputs, proof::sp1::SP1Proof as KeySpaceSP1Proof, transaction::Transaction,
};
use keyspace_state_manager::storage::btree::BTreeStorage;
use scripts::{load_record_proof, read_forced_vk_hash};

const ELF: &[u8] = include_bytes!("../../../../batcher/elf/riscv32im-succinct-zkvm-elf");
const ECDSA_RECORD_ELF: &[u8] =
    include_bytes!("../../../../ecdsa-record/elf/riscv32im-succinct-zkvm-elf");

fn main() {
    // Make sure the circuits artifacts are downloaded.
    try_install_circuit_artifacts();

    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    // Setup the prover client.
    let client = ProverClient::new();

    // Setup the program.
    let (batcher_pk, _) = client.setup(ELF);
    let (_, record_vk) = client.setup(ECDSA_RECORD_ELF);
    let record_vk_hash = record_vk.hash_bytes();
    let forced_vk_hash = read_forced_vk_hash();

    let mut stdin = SP1Stdin::new();

    let storage = BTreeStorage::default();
    let mut imt = Imt::writer(Keccak::v256, storage);
    let old_root = imt.root();
    println!("imt old root: 0x{}", hex::encode(old_root));

    let mut proof_files = read_dir("proofs/sp1/")
        .expect("failed to read proofs/sp1/ directory")
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    proof_files.sort_by_key(|entry| entry.path());

    let mut txs_commitment = [0; 32];
    let txs = proof_files
        .into_iter()
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension()?.to_str()? != "json" {
                return None;
            }

            println!("loading record proof from {path:?}");

            let (record_proof, storage_hash) = load_record_proof(&path);
            match record_proof.proof {
                SP1Proof::Compressed(proof) => stdin.write_proof(*proof, record_vk.vk.clone()),
                _ => panic!("record proof should be compressed to be recursively verified"),
            };

            // Fetch the KeySpace id and the new key from the record proof public inputs.
            let keyspace_id = record_proof.public_values.as_slice()[..32]
                .try_into()
                .expect("failed to read keyspace_id from record proof");

            let keyspace_value = record_proof.public_values.as_slice()[64..]
                .try_into()
                .expect("failed to read new keyspace_value from record proof");

            println!("keyspace_id: 0x{}", hex::encode(keyspace_id));
            println!("keyspace_value: 0x{}", hex::encode(keyspace_value));

            let imt_mutate_proof = imt
                .set_node(keyspace_id, keyspace_value)
                .expect("failed to set node");

            let tx = Transaction::sequenced(
                imt_mutate_proof,
                txs_commitment,
                KeySpaceSP1Proof::new(record_vk_hash, forced_vk_hash, storage_hash),
            );

            txs_commitment = tx
                .commitment(Some(txs_commitment))
                .expect("failed to compute the transaction commitment");

            Some(tx)
        })
        .collect::<Vec<_>>();

    let new_root = imt.root();
    println!("imt new root: 0x{}", hex::encode(new_root));

    let inputs = Inputs {
        old_root,
        new_root: imt.root(),
        txs_commitment,
        txs,
    };

    // Generate the proof for it.
    stdin.write(&inputs);
    let proof = client
        .prove(&batcher_pk, stdin)
        .groth16()
        .run()
        .expect("batcher proving failed");

    // Get the proof as bytes.
    let solidity_proof = proof.bytes().encode_hex::<String>();
    println!("solidity_proof: {:?}", format!("0x{}", solidity_proof));
}
