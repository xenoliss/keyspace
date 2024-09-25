use k256::{ecdsa::SigningKey, elliptic_curve::rand_core::OsRng};
use rand::Rng;
use sp1_sdk::{install::try_install_circuit_artifacts, HashableKey, ProverClient, SP1Stdin};
use tiny_keccak::{Hasher, Keccak};

use keyspace_programs_lib::{
    authorization_hash, authorization_key,
    ecdsa_record::{inputs::Inputs, signature::Signature},
    keyspace_value, storage_hash, Hash256,
};
use scripts::{read_forced_vk_hash, save_record_proof};

const ELF: &[u8] = include_bytes!("../../../../ecdsa-record/elf/riscv32im-succinct-zkvm-elf");

fn main() {
    // Make sure the circuits artifacts are downloaded.
    try_install_circuit_artifacts();

    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    // Setup the prover client.
    let client = ProverClient::new();

    // Setup the program.
    let (pk, vk) = client.setup(ELF);

    // Generate proofs.
    for i in 0..10 {
        // Generate random inputs.
        let (inputs, storage_hash) = random_inputs(vk.hash_bytes());

        // Setup the inputs.
        let mut stdin = SP1Stdin::new();
        stdin.write(&inputs);

        // let (mut public_values, execution_report) = client.execute(ELF, stdin).run().unwrap();
        // println!(
        //     "Executed program with {} cycles",
        //     execution_report.total_instruction_count() + execution_report.total_syscall_count()
        // );
        // println!("Full execution report:\n{:#?}", execution_report);

        // Generate the proof.
        let proof = client
            .prove(&pk, stdin)
            .compressed()
            .run()
            .expect("failed to generate proof");

        let file = format!("proofs/sp1/{i}_record_proof.json");
        println!("Saving record proof in {file}");
        save_record_proof(proof, storage_hash, file);
    }
}

/// Generate random [Inputs] for the ECDSA Record Program.
fn random_inputs(record_vk_hash: Hash256) -> (Inputs, Hash256) {
    let signing_key = SigningKey::random(&mut OsRng);
    let verifying_key = signing_key.verifying_key();

    let mut rng = rand::thread_rng();
    let sidecar_hash: Hash256 = rng.gen();

    let storage_hash = {
        let pk = verifying_key.to_encoded_point(false);
        let x = pk.x().unwrap();
        let y = pk.y().unwrap();

        let mut pk = [0; 64];
        pk[..32].copy_from_slice(x);
        pk[32..].copy_from_slice(y);

        let auth_hash = authorization_hash(&pk);
        storage_hash(&auth_hash, &sidecar_hash)
    };

    let authorization_key = authorization_key(&record_vk_hash, Some(&read_forced_vk_hash()));

    let keyspace_id = keyspace_value(&authorization_key, &storage_hash);
    let current_value = keyspace_id;

    let mut rng = rand::thread_rng();
    let new_value: Hash256 = rng.gen();

    let sig = sign_update(&signing_key, &keyspace_id, &current_value, &new_value);

    let inputs = Inputs {
        keyspace_id,
        current_value,
        new_value,

        sig,
        sidecar_hash,
        authorization_key,
    };

    (inputs, storage_hash)
}

/// Signs a KeySpace update with the given [SigningKey].
fn sign_update(
    signing_key: &SigningKey,
    keyspace_id: &Hash256,
    current_value: &Hash256,
    new_value: &Hash256,
) -> Signature {
    let msg_hash = {
        let mut k = Keccak::v256();
        let mut msg_hash = [0; 32];
        k.update(keyspace_id);
        k.update(current_value);
        k.update(new_value);
        k.finalize(&mut msg_hash);

        msg_hash
    };

    let (sig, recid) = signing_key.sign_prehash_recoverable(&msg_hash).unwrap();
    let sig_bytes = sig.to_bytes();

    Signature {
        sig: sig_bytes.into(),
        recid: recid.to_byte(),
    }
}
