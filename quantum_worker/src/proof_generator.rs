use std::fs::{self, read};

use quantum_db::repository::{proof_repository::{get_proof_by_proof_hash, update_reduction_data}, reduction_circuit_repository::get_reduction_circuit_data_by_id, user_circuit_data_repository::get_user_circuit_data_by_circuit_hash};
use quantum_types::{enums::{proving_schemes::ProvingSchemes, task_type::TaskType}, traits::pis::Pis, types::{config::ConfigData, db::{task::Task, user_circuit_data}, gnark_groth16::GnarkGroth16Pis, snarkjs_groth16::SnarkJSGroth16Pis}};
use quantum_utils::file::read_bytes_from_file;
use sqlx::{MySql, Pool};
use anyhow::{Ok, Result as AnyhowResult};
use quantum_circuits_ffi::circuit_builder::{CircomProof, CircomVK, GnarkProof, GnarkVK, ProveResult};
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
    let inner_proof_data = fs::read_to_string(&inner_proof_path)?;
    println!("inner_proof_path :: {:?}", inner_proof_path);

    // Get inner_vk
    let inner_vk_path = user_circuit_data.vk_path;
    let inner_vk_data = fs::read_to_string(&inner_vk_path)?;
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
    let outer_vk_bytes = read_bytes_from_file(&outer_vk_path)?;
    println!("outer_vk_path :: {:?}", outer_vk_path);

    let prove_result: ProveResult;

    // Call proof generation to quantum_reduction_circuit
    println!("Calling gnark groth16 proof generation");
    let reduction_start_time = Instant::now();
    if user_circuit_data.proving_scheme == ProvingSchemes::GnarkGroth16 {
        // 1.Reconstruct inner proof
        let gnark_inner_proof: GnarkProof = serde_json::from_str(&inner_proof_data)?;
        let gnark_inner_vk: GnarkVK = serde_json::from_str(&inner_vk_data)?;
        let gnark_pis: GnarkGroth16Pis = GnarkGroth16Pis::read_pis(&inner_pis_path)?;
        // 2.Call .prove()
        prove_result = gnark_inner_proof.prove(gnark_inner_vk, outer_pk_bytes, outer_vk_bytes, gnark_pis.0);

    } else if user_circuit_data.proving_scheme == ProvingSchemes::Groth16 {
        // 1.Reconstruct inner proof
        let snarkjs_inner_proof: CircomProof = serde_json::from_str(&inner_proof_data)?;
        let snarkjs_inner_vk: CircomVK = serde_json::from_str(&inner_vk_data)?;
        let circom_pis: SnarkJSGroth16Pis = SnarkJSGroth16Pis::read_pis(&inner_pis_path)?;
        // 2. Call .prove()
        prove_result = snarkjs_inner_proof.prove(snarkjs_inner_vk, outer_pk_bytes, outer_vk_bytes, circom_pis.0);
    } else {
        return Ok(());
    }

    let reduction_time = reduction_start_time.elapsed().as_secs();

    // Check if build was done successfully
    if !prove_result.pass{
        return Err(anyhow::Error::msg(prove_result.msg));
    }

    println!("Reduced Proof successfully generated in {:?}", reduction_time);

    // Dump reduced proof and public inputs
    // TODO change proof bytes and pis bytes values
    let (reduced_proof_path, reduced_pis_path) = dump_reduction_proof_data(config, &user_circuit_hash, &proof_hash, prove_result.proof_raw_bytes, prove_result.pub_inputs)?;
    println!("Dumped reduced proof and pis");

    // update reduction data corresponding to proof
    update_reduction_data(pool, &proof_hash, &reduced_proof_path, &reduced_pis_path, reduction_time).await?;
    println!("Updated reduction data to corresponding proof");

    Ok(())
}