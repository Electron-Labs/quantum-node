use std::{fs, str::FromStr};

use anyhow::{anyhow, Ok, Result as AnyhowResult};
use num_bigint::BigUint;
use quantum_circuits_interface::ffi::interactor::QuantumV2CircuitInteractor;
use quantum_db::repository::{
    proof_repository::{get_proof_by_proof_hash, update_reduction_data},
    reduction_circuit_repository::get_reduction_circuit_data_by_id,
    user_circuit_data_repository::get_user_circuit_data_by_circuit_hash,
};
use quantum_types::{
    enums::{proving_schemes::ProvingSchemes, task_type::TaskType},
    traits::{
        circuit_interactor::{CircuitInteractorFFI, GenerateReductionProofResult},
        pis::Pis,
        proof::Proof,
        vkey::Vkey,
    },
    types::{
        config::ConfigData,
        db::{task::Task, user_circuit_data, proof::Proof as DBProof},
        gnark_groth16::{GnarkGroth16Pis, GnarkGroth16Proof, GnarkGroth16Vkey},
        halo2_plonk::{Halo2PlonkPis, Halo2PlonkProof, Halo2PlonkVkey},
        snarkjs_groth16::{SnarkJSGroth16Pis, SnarkJSGroth16Proof, SnarkJSGroth16Vkey},
    },
};
use quantum_utils::{error_line, file::read_bytes_from_file};
use sqlx::{MySql, Pool};
use tokio::time::Instant;
use tracing::info;
use quantum_types::types::db::reduction_circuit::ReductionCircuit;
use quantum_types::types::db::user_circuit_data::UserCircuitData;
use crate::connection::get_pool;
use crate::utils::dump_reduction_proof_data;

pub async fn handle_proof_generation_task(
    proof_generation_task: Task,
    config: &ConfigData,
) -> AnyhowResult<()> {
    let user_circuit_hash = proof_generation_task.user_circuit_hash;
    let proof_hash = proof_generation_task.proof_id.unwrap();

    let user_circuit_data = get_user_circuit_data_by_circuit_hash(get_pool().await, &user_circuit_hash).await?;
    let proof_data = get_proof_by_proof_hash(get_pool().await, &proof_hash).await?;

    // TODO: remove the unwrap from here
    let reduction_circuit_id = user_circuit_data.reduction_circuit_id.clone().unwrap();
    let reduction_circuit_data = get_reduction_circuit_data_by_id(get_pool().await, &reduction_circuit_id).await?;

    // Call proof generation to quantum_reduction_circuit
    let (prove_result, reduction_time) = generate_reduced_proof(&user_circuit_data, &proof_data, &reduction_circuit_data).await?;

    if !prove_result.success {
        return Err(anyhow::Error::msg(error_line!(prove_result.msg)));
    }

    // Dump reduced proof and public inputs
    // TODO change proof bytes and pis bytes values
    let (reduced_proof_path, reduced_pis_path) = dump_reduction_proof_data(
        config,
        &user_circuit_hash,
        &proof_hash,
        prove_result.reduced_proof,
        prove_result.reduced_pis,
    )?;
    info!("Dumped reduced proof and pis");

    // update reduction data corresponding to proof
    update_reduction_data(
        get_pool().await,
        &proof_hash,
        &reduced_proof_path,
        &reduced_pis_path,
        reduction_time,
    )
    .await?;
    info!("Updated reduction data to corresponding proof");
    Ok(())
}

async fn generate_reduced_proof(user_circuit_data: &UserCircuitData, proof_data: &DBProof, reduction_circuit_data: &ReductionCircuit ) -> AnyhowResult<(GenerateReductionProofResult, u64)> {
    // Get outer_pk
    let outer_pk_path = &reduction_circuit_data.proving_key_path[..];
    let outer_pk_bytes = read_bytes_from_file(outer_pk_path)?;
    println!("outer_pk_path :: {:?}", outer_pk_path);

    // Get outer_vk
    let outer_vk_path = &reduction_circuit_data.vk_path[..];
    let outer_vk = GnarkGroth16Vkey::read_vk(&outer_vk_path)?;
    println!("outer_vk_path :: {:?}", outer_vk_path);

    let reduction_start_time = Instant::now();
    let prove_result: GenerateReductionProofResult;

    info!("Calling gnark groth16 proof generation");
    if user_circuit_data.proving_scheme == ProvingSchemes::GnarkGroth16 {
        prove_result = generate_gnark_groth16_reduced_proof(user_circuit_data, proof_data, outer_pk_bytes, outer_vk).await?;
    } else if user_circuit_data.proving_scheme == ProvingSchemes::Groth16 {
        prove_result = generate_snarkjs_groth16_reduced_proof(user_circuit_data, proof_data, outer_pk_bytes, outer_vk).await?;
    } else if user_circuit_data.proving_scheme == ProvingSchemes::Halo2Plonk {
        prove_result = generate_halo2_plonk_reduced_proof(user_circuit_data, proof_data, outer_pk_bytes, outer_vk).await?;
    } else {
        return Err(anyhow!(error_line!("unsupported proving scheme in proof reduction")));
    }

    let reduction_time = reduction_start_time.elapsed().as_secs();
    info!("Reduced Proof successfully generated in {:?}", reduction_time);

    Ok((prove_result, reduction_time))
}

async fn generate_gnark_groth16_reduced_proof(user_circuit_data: &UserCircuitData, proof_data: &DBProof, outer_pk_bytes: Vec<u8>, outer_vk: GnarkGroth16Vkey) -> AnyhowResult<GenerateReductionProofResult> {
    // Get inner_proof
    let inner_proof_path = &proof_data.proof_path;
    println!("inner_proof_path :: {:?}", inner_proof_path);

    // Get inner_vk
    let inner_vk_path = &user_circuit_data.vk_path;
    println!("inner_vk_path :: {:?}", inner_vk_path);

    // Get inner_pis
    let inner_pis_path = &proof_data.pis_path;
    println!("inner_pis_path :: {:?}", inner_pis_path);
    // 1.Reconstruct inner proof
    let gnark_inner_proof: GnarkGroth16Proof =
        GnarkGroth16Proof::read_proof(&inner_proof_path)?;
    let gnark_inner_vk: GnarkGroth16Vkey = GnarkGroth16Vkey::read_vk(&inner_vk_path)?;
    let gnark_inner_pis: GnarkGroth16Pis = GnarkGroth16Pis::read_pis(&inner_pis_path)?;
    // 2.Call reduced proof generator for gnark inner proof
    let prove_result = QuantumV2CircuitInteractor::generate_gnark_groth16_reduced_proof(
        gnark_inner_proof,
        gnark_inner_vk.clone(),
        gnark_inner_pis.clone(),
        outer_vk,
        outer_pk_bytes,
    );

    verify_proof_reduction_result(&prove_result, &user_circuit_data, gnark_inner_vk, gnark_inner_pis)?;
    Ok(prove_result)
}

async fn generate_snarkjs_groth16_reduced_proof(user_circuit_data: &UserCircuitData, proof_data: &DBProof, outer_pk_bytes: Vec<u8>, outer_vk: GnarkGroth16Vkey) -> AnyhowResult<GenerateReductionProofResult> {
    // Get inner_proof
    let inner_proof_path = &proof_data.proof_path;
    println!("inner_proof_path :: {:?}", inner_proof_path);

    // Get inner_vk
    let inner_vk_path = &user_circuit_data.vk_path;
    println!("inner_vk_path :: {:?}", inner_vk_path);

    // Get inner_pis
    let inner_pis_path = &proof_data.pis_path;
    println!("inner_pis_path :: {:?}", inner_pis_path);
    // 1.Reconstruct inner proof
    let snarkjs_inner_proof: SnarkJSGroth16Proof =
        SnarkJSGroth16Proof::read_proof(&inner_proof_path)?;
    let snarkjs_inner_vk: SnarkJSGroth16Vkey = SnarkJSGroth16Vkey::read_vk(&inner_vk_path)?;
    let snarkjs_inner_pis: SnarkJSGroth16Pis = SnarkJSGroth16Pis::read_pis(&inner_pis_path)?;
    // 2. Call reduced proof generator for circom inner proof
    let prove_result = QuantumV2CircuitInteractor::generate_snarkjs_groth16_reduced_proof(
        snarkjs_inner_proof,
        snarkjs_inner_vk.clone(),
        snarkjs_inner_pis.clone(),
        outer_vk,
        outer_pk_bytes,
    );
    verify_proof_reduction_result(&prove_result, &user_circuit_data, snarkjs_inner_vk, snarkjs_inner_pis)?;
    Ok(prove_result)
}

async fn generate_halo2_plonk_reduced_proof(user_circuit_data: &UserCircuitData, proof_data: &DBProof, outer_pk_bytes: Vec<u8>, outer_vk: GnarkGroth16Vkey) -> AnyhowResult<GenerateReductionProofResult> {
    // Get inner_proof
    let inner_proof_path = &proof_data.proof_path;
    println!("inner_proof_path :: {:?}", inner_proof_path);

    // Get inner_vk
    let inner_vk_path = &user_circuit_data.vk_path;
    println!("inner_vk_path :: {:?}", inner_vk_path);

    // Get inner_pis
    let inner_pis_path = &proof_data.pis_path;
    println!("inner_pis_path :: {:?}", inner_pis_path);

    let inner_proof = Halo2PlonkProof::read_proof(&inner_proof_path)?;
    let inner_vk = Halo2PlonkVkey::read_vk(&inner_vk_path)?;
    let inner_pis = Halo2PlonkPis::read_pis(&inner_pis_path)?;
    let prove_result = QuantumV2CircuitInteractor::generate_halo2_plonk_reduced_proof(
        inner_pis.clone(),
        inner_proof,
        inner_vk.clone(),
        outer_vk,
        outer_pk_bytes,
    );
    verify_proof_reduction_result(&prove_result, &user_circuit_data, inner_vk, inner_pis)?;
    Ok(prove_result)
}

fn verify_proof_reduction_result<V: Vkey, P: Pis>(prove_result: &GenerateReductionProofResult, user_circuit_data: &UserCircuitData, inner_vk: V, inner_pis: P) -> AnyhowResult<()>{
    let mut keccak_ip = Vec::<u8>::new();
    let vkey_hash = inner_vk.extended_keccak_hash(user_circuit_data.n_commitments)?;
    println!("vkey_hash {:?}", vkey_hash);
    keccak_ip.extend(vkey_hash);
    let pis_hash = inner_pis.extended_keccak_hash()?;
    println!("pis_hash {:?}", pis_hash);
    keccak_ip.extend(pis_hash);
    let hash = keccak_hash::keccak(keccak_ip).0;
    let pis1 = BigUint::from_bytes_be(&hash[0..16]).to_string();
    let pis2 = BigUint::from_bytes_be(&hash[16..32]).to_string();
    println!("pis1 {:?}", pis1);
    println!("pis2 {:?}", pis2);
    if prove_result.success {
        // worker will panic here
        assert_eq!(pis1, prove_result.reduced_pis.0[0]);
        assert_eq!(pis2, prove_result.reduced_pis.0[1]);
    }
    Ok(())
}


