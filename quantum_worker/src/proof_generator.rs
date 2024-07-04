use std::{fs, str::FromStr};

use anyhow::{Ok, Result as AnyhowResult};
use num_bigint::BigUint;
use quantum_circuits_ffi::interactor::QuantumV2CircuitInteractor;
use quantum_db::repository::{
    proof_repository::{get_proof_by_proof_hash, update_reduction_data},
    reduction_circuit_repository::get_reduction_circuit_data_by_id,
    user_circuit_data_repository::get_user_circuit_data_by_circuit_hash,
};
use quantum_types::{
    enums::{proving_schemes::ProvingSchemes, task_type::TaskType},
    traits::{
        circuit_interactor::{CircuitInteractor, GenerateReductionProofResult},
        pis::Pis,
        proof::Proof,
        vkey::Vkey,
    },
    types::{
        config::ConfigData,
        db::{reduction_circuit::ReductionCircuit, task::Task, user_circuit_data::UserCircuitData, proof::Proof as ProofData},
        gnark_groth16::{GnarkGroth16Pis, GnarkGroth16Proof, GnarkGroth16Vkey},
        halo2_plonk::{Halo2PlonkPis, Halo2PlonkProof, Halo2PlonkVkey},
        snarkjs_groth16::{SnarkJSGroth16Pis, SnarkJSGroth16Proof, SnarkJSGroth16Vkey},
        proof_gen_configs::{InnerProofGenerationConfig, ProofGenerationConfig},
    },
};
use quantum_utils::{error_line, file::read_bytes_from_file};
use sqlx::{MySql, Pool};
use tokio::time::Instant;
use tracing::info;

use crate::utils::dump_reduction_proof_data;

pub fn get_proof_generation_config(
    user_circuit_data: &UserCircuitData, 
    proof_data: &ProofData, 
    reduction_circuit_data: &ReductionCircuit
) -> AnyhowResult<ProofGenerationConfig>{
    let proof_gen_config  = ProofGenerationConfig{
        inner_proof_path: proof_data.proof_path.clone(),
        inner_vk_path: user_circuit_data.vk_path.clone(),
        inner_pis_path: proof_data.pis_path.clone(),
        outer_pk_bytes: read_bytes_from_file(&reduction_circuit_data.proving_key_path)?,
        outer_vk: GnarkGroth16Vkey::read_vk(&reduction_circuit_data.vk_path)?
    };
    Ok(proof_gen_config)
}

pub fn get_inner_proof_generation_config<T:Proof, V:Vkey, P:Pis>(proof_gen_config: &ProofGenerationConfig) -> AnyhowResult<InnerProofGenerationConfig<T,V,P>>{
    let scheme_inner_proof: T = T::read_proof(&proof_gen_config.inner_proof_path)?; 
    let scheme_inner_vk: V = V::read_vk(&proof_gen_config.inner_vk_path)?;
    let scheme_inner_pis: P = Pis::read_pis(&proof_gen_config.inner_pis_path)?;

    let inner_proof_gen_config = InnerProofGenerationConfig{
        scheme_inner_proof,
        scheme_inner_vk,
        scheme_inner_pis
    };
    Ok(inner_proof_gen_config)
}

pub fn verify_prove_result<V:Vkey, P:Pis>(prove_result: &GenerateReductionProofResult, scheme_inner_vk: V, scheme_inner_pis: P) -> AnyhowResult<()>{
    let mut keccak_ip = Vec::<u8>::new();
    let vkey_hash = scheme_inner_vk.keccak_hash()?;
    println!("vkey_hash {:?}", vkey_hash);
    keccak_ip.extend(vkey_hash);
    let pis_hash = scheme_inner_pis.keccak_hash()?;
    println!("pis_hash {:?}", pis_hash);
    keccak_ip.extend(pis_hash);
    let hash = keccak_hash::keccak(keccak_ip).0;
    let pis1 = BigUint::from_bytes_be(&hash[0..16]).to_string();
    let pis2 = BigUint::from_bytes_be(&hash[16..32]).to_string();
    println!("pis1 {:?}", pis1);
    println!("pis2 {:?}", pis2);
    println!("p1 {:?}", prove_result.reduced_pis.0[0]);
    println!("p2 {:?}", prove_result.reduced_pis.0[1]);
    assert_eq!(pis1, prove_result.reduced_pis.0[0]);
    assert_eq!(pis2, prove_result.reduced_pis.0[1]);
    Ok(())
}

pub fn generate_reduction_proof_result(proof_gen_config: &ProofGenerationConfig, proving_scheme: ProvingSchemes) -> AnyhowResult<Option<GenerateReductionProofResult>> {
    
    let prove_result: GenerateReductionProofResult;

    if proving_scheme == ProvingSchemes::GnarkGroth16 {
        // 1.Reconstruct inner proof
        let gnark_inner_proof_gen_config = get_inner_proof_generation_config::<GnarkGroth16Proof, GnarkGroth16Vkey, GnarkGroth16Pis>(&proof_gen_config)?;
        // 2.Call reduced proof generator for gnark inner proof
        prove_result = QuantumV2CircuitInteractor::generate_gnark_groth16_reduced_proof(
            gnark_inner_proof_gen_config.scheme_inner_proof,
            gnark_inner_proof_gen_config.scheme_inner_vk.clone(),
            gnark_inner_proof_gen_config.scheme_inner_pis.clone(),
            proof_gen_config.outer_vk.clone(),
            proof_gen_config.outer_pk_bytes.clone(),
        );
        verify_prove_result::<GnarkGroth16Vkey, GnarkGroth16Pis>(&prove_result, gnark_inner_proof_gen_config.scheme_inner_vk, gnark_inner_proof_gen_config.scheme_inner_pis)?;
        return Ok(Some(prove_result));
    } else if proving_scheme == ProvingSchemes::Groth16 {
        let snark_inner_proof_gen_config = get_inner_proof_generation_config::<SnarkJSGroth16Proof, SnarkJSGroth16Vkey, SnarkJSGroth16Pis>(&proof_gen_config)?;
        prove_result = QuantumV2CircuitInteractor::generate_snarkjs_groth16_reduced_proof(
            snark_inner_proof_gen_config.scheme_inner_proof,
            snark_inner_proof_gen_config.scheme_inner_vk.clone(),
            snark_inner_proof_gen_config.scheme_inner_pis.clone(),
            proof_gen_config.outer_vk.clone(),
            proof_gen_config.outer_pk_bytes.clone(),
        );
        verify_prove_result::<SnarkJSGroth16Vkey, SnarkJSGroth16Pis>(&prove_result, snark_inner_proof_gen_config.scheme_inner_vk, snark_inner_proof_gen_config.scheme_inner_pis)?;
        return Ok(Some(prove_result));
    } else if proving_scheme == ProvingSchemes::Halo2Plonk {
        let halo2_inner_proof_gen_config = get_inner_proof_generation_config::<Halo2PlonkProof, Halo2PlonkVkey, Halo2PlonkPis>(&proof_gen_config)?;
        prove_result = QuantumV2CircuitInteractor::generate_halo2_plonk_reduced_proof(
            halo2_inner_proof_gen_config.scheme_inner_pis.clone(),
            halo2_inner_proof_gen_config.scheme_inner_proof,
            halo2_inner_proof_gen_config.scheme_inner_vk.clone(),
            proof_gen_config.outer_vk.clone(),
            proof_gen_config.outer_pk_bytes.clone(),
        );
        verify_prove_result::<Halo2PlonkVkey, Halo2PlonkPis>(&prove_result, halo2_inner_proof_gen_config.scheme_inner_vk, halo2_inner_proof_gen_config.scheme_inner_pis)?;
        return Ok(Some(prove_result));
    } else {
        return Ok(None);
    }
}

pub async fn handle_proof_generation_task(
    pool: &Pool<MySql>,
    proof_generation_task: Task,
    config: &ConfigData,
) -> AnyhowResult<()> {
    assert_eq!(proof_generation_task.task_type, TaskType::ProofGeneration);
    let user_circuit_hash = proof_generation_task.user_circuit_hash;
    let proof_hash = proof_generation_task.proof_id.unwrap();

    // Get user_circuit_data
    let user_circuit_data = get_user_circuit_data_by_circuit_hash(pool, &user_circuit_hash).await?;

    // Get proof_data
    let proof_data = get_proof_by_proof_hash(pool, &proof_hash).await?;

    // Get corresponding reduction circuit
    let reduction_circuit_id = user_circuit_data.reduction_circuit_id.clone().unwrap();
    let reduction_circuit_data =
        get_reduction_circuit_data_by_id(pool, &reduction_circuit_id).await?;

    let proof_gen_config = get_proof_generation_config(&user_circuit_data, &proof_data, &reduction_circuit_data)?;
    
    // Call proof generation to quantum_reduction_circuit
    info!("Calling gnark groth16 proof generation");
    let reduction_start_time = Instant::now();
    
    let prove_result: Option<GenerateReductionProofResult> = generate_reduction_proof_result(&proof_gen_config, user_circuit_data.proving_scheme)?;

    if prove_result.is_none(){
        return Ok(());
    }
    
    let prove_result = prove_result.unwrap();

    let reduction_time = reduction_start_time.elapsed().as_secs();

    // Check if build was done successfully
    if !prove_result.success {
        return Err(anyhow::Error::msg(error_line!(prove_result.msg)));
    }

    info!(
        "Reduced Proof successfully generated in {:?}",
        reduction_time
    );

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
        pool,
        &proof_hash,
        &reduced_proof_path,
        &reduced_pis_path,
        reduction_time,
    )
    .await?;
    info!("Updated reduction data to corresponding proof");

    Ok(())
}
