use std::{fs, str::FromStr};

use num_bigint::BigUint;
use quantum_db::repository::{proof_repository::{get_proof_by_proof_hash, update_reduction_data}, reduction_circuit_repository::get_reduction_circuit_data_by_id, user_circuit_data_repository::get_user_circuit_data_by_circuit_hash};
use quantum_types::{enums::{proving_schemes::ProvingSchemes, task_type::TaskType}, traits::{circuit_interactor::{CircuitInteractor, GenerateReductionProofResult}, pis::Pis, proof::Proof, vkey::Vkey}, types::{config::ConfigData, db::{task::Task, user_circuit_data}, gnark_groth16::{GnarkGroth16Pis, GnarkGroth16Proof, GnarkGroth16Vkey}, snarkjs_groth16::{SnarkJSGroth16Pis, SnarkJSGroth16Proof, SnarkJSGroth16Vkey}}};
use quantum_utils::{file::read_bytes_from_file, keccak::{self, convert_string_to_le_bytes}};
use sqlx::{MySql, Pool};
use anyhow::{Ok, Result as AnyhowResult};
use quantum_circuits_ffi:: interactor::QuantumV2CircuitInteractor;
use tokio::time::Instant;

use crate::utils::dump_reduction_proof_data;

pub async fn handle_proof_generation_task(pool: &Pool<MySql>, proof_generation_task: Task, config: &ConfigData) -> AnyhowResult<()> {
    assert_eq!(proof_generation_task.task_type, TaskType::ProofGeneration);
    let user_circuit_hash = proof_generation_task.user_circuit_hash;
    let proof_hash = proof_generation_task.proof_id.unwrap();

    // Get user_circuit_data
    let user_circuit_data = get_user_circuit_data_by_circuit_hash(pool, &user_circuit_hash).await?;

    // Get proof_data
    let proof_data = get_proof_by_proof_hash(pool, &proof_hash).await?;

    // Get corresponding reduction circuit
    let reduction_circuit_id = user_circuit_data.reduction_circuit_id.unwrap();
    let reduction_circuit_data = get_reduction_circuit_data_by_id(pool, &reduction_circuit_id).await?;

    // Get inner_proof
    let inner_proof_path = proof_data.proof_path;
    println!("inner_proof_path :: {:?}", inner_proof_path);

    // Get inner_vk
    let inner_vk_path = user_circuit_data.vk_path;
    println!("inner_vk_path :: {:?}", inner_vk_path);

    // Get inner_pis
    let inner_pis_path = proof_data.pis_path;
    println!("inner_pis_path :: {:?}", inner_pis_path);

    // Get outer_pk
    let outer_pk_path = reduction_circuit_data.proving_key_path;
    let outer_pk_bytes = read_bytes_from_file(&outer_pk_path)?;
    println!("outer_pk_path :: {:?}", outer_pk_path);

    // Get outer_vk
    let outer_vk_path = reduction_circuit_data.vk_path;
    let outer_vk = GnarkGroth16Vkey::read_vk(&outer_vk_path)?;
    println!("outer_vk_path :: {:?}", outer_vk_path);

    let prove_result: GenerateReductionProofResult;

    // Call proof generation to quantum_reduction_circuit
    println!("Calling gnark groth16 proof generation");
    let reduction_start_time = Instant::now();
    if user_circuit_data.proving_scheme == ProvingSchemes::GnarkGroth16 {
        // 1.Reconstruct inner proof
        let gnark_inner_proof: GnarkGroth16Proof = GnarkGroth16Proof::read_proof(&inner_proof_path)?;
        let gnark_inner_vk: GnarkGroth16Vkey = GnarkGroth16Vkey::read_vk(&inner_vk_path)?;
        let gnark_inner_pis: GnarkGroth16Pis = GnarkGroth16Pis::read_pis(&inner_pis_path)?;
        // 2.Call reduced proof generator for gnark inner proof
        prove_result = QuantumV2CircuitInteractor::generate_gnark_groth16_reduced_proof(gnark_inner_proof, gnark_inner_vk.clone(), gnark_inner_pis.clone(), outer_vk, outer_pk_bytes);

        let mut keccak_ip = Vec::<u8>::new();
        let vkey_hash = gnark_inner_vk.keccak_hash()?;
        println!("vkey_hash {:?}", vkey_hash);
        keccak_ip.extend(gnark_inner_vk.keccak_hash()?.to_vec().iter().cloned());
        for i in 0..gnark_inner_pis.0.len() {
            let pi = gnark_inner_pis.0[i].clone();
            keccak_ip.extend(convert_string_to_le_bytes(&pi).to_vec().iter().cloned());
        }
        let hash = keccak_hash::keccak(keccak_ip).0;
        let pis1 = BigUint::from_bytes_le(&hash[0..16]).to_string();
        let pis2 = BigUint::from_bytes_le(&hash[16..32]).to_string();
        println!("pis1 {:?}", pis1);
        println!("pis2 {:?}", pis2);
        println!("p1 {:?}", prove_result.reduced_pis.0[0]);
        println!("p2 {:?}", prove_result.reduced_pis.0[1]);
        assert_eq!(pis1, prove_result.reduced_pis.0[0]);
        assert_eq!(pis2, prove_result.reduced_pis.0[1]);
    } else if user_circuit_data.proving_scheme == ProvingSchemes::Groth16 {
        // 1.Reconstruct inner proof
        let snarkjs_inner_proof: SnarkJSGroth16Proof = SnarkJSGroth16Proof::read_proof(&inner_proof_path)?;
        let snarkjs_inner_vk: SnarkJSGroth16Vkey = SnarkJSGroth16Vkey::read_vk(&inner_vk_path)?;
        let snarkjs_inner_pis: SnarkJSGroth16Pis = SnarkJSGroth16Pis::read_pis(&inner_pis_path)?;
        // 2. Call reduced proof generator for circom inner proof
        prove_result = QuantumV2CircuitInteractor::generate_snarkjs_groth16_reduced_proof(snarkjs_inner_proof, snarkjs_inner_vk, snarkjs_inner_pis, outer_vk, outer_pk_bytes);
    } else {
        return Ok(());
    }

    let reduction_time = reduction_start_time.elapsed().as_secs();

    // Check if build was done successfully
    if !prove_result.success{
        return Err(anyhow::Error::msg(prove_result.msg));
    }

    println!("Reduced Proof successfully generated in {:?}", reduction_time);

    // Dump reduced proof and public inputs
    // TODO change proof bytes and pis bytes values
    let (reduced_proof_path, reduced_pis_path) = dump_reduction_proof_data(config, &user_circuit_hash, &proof_hash, prove_result.reduced_proof, prove_result.reduced_pis)?;
    println!("Dumped reduced proof and pis");

    // update reduction data corresponding to proof
    update_reduction_data(pool, &proof_hash, &reduced_proof_path, &reduced_pis_path, reduction_time).await?;
    println!("Updated reduction data to corresponding proof");

    Ok(())
}