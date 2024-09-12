use std::{fs, str::FromStr, time::Duration};
use anyhow::{anyhow, Ok, Result as AnyhowResult};
use ark_bn254::{Bn254, Config, Fq, Fq2, Fr, G1Affine, G2Affine};
use ark_groth16::{verifier, VerifyingKey, Proof as ArkProof};
use ark_serialize::CanonicalSerialize;
use num_bigint::BigUint;
use bonsai_sdk::non_blocking::Client;
use quantum_db::repository::{
    proof_repository::{update_reduction_data},
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
        config::ConfigData, db::{proof::Proof as DBProof, task::Task, user_circuit_data}, gnark_groth16::{GnarkGroth16Pis, GnarkGroth16Proof, GnarkGroth16Vkey}, gnark_plonk::{GnarkPlonkSolidityProof, GnarkPlonkPis, GnarkPlonkVkey}, halo2_plonk::{Halo2PlonkPis, Halo2PlonkProof, Halo2PlonkVkey}, snarkjs_groth16::{SnarkJSGroth16Pis, SnarkJSGroth16Proof, SnarkJSGroth16Vkey}
    },
};
use quantum_utils::{error_line, file::{dump_object, read_bytes_from_file}};
use risc0_zkvm::{compute_image_id, default_prover, serde::to_vec, Assumption, ExecutorEnv, Receipt};
use sqlx::{MySql, Pool};
use tokio::time::Instant;
use tracing::info;
use quantum_db::repository::proof_repository::get_proof_by_proof_id;
use quantum_types::types::db::reduction_circuit::ReductionCircuit;
use quantum_types::types::db::user_circuit_data::UserCircuitData;
use crate::{connection::get_pool, AVAIL_BH};
use crate::{bonsai::execute_proof_reduction, connection::get_pool};
use crate::utils::dump_reduction_proof_data;

pub async fn handle_proof_generation_and_updation(
    proof_id: u64,
    proof_hash: &str,
    user_circuit_hash: &str,
    config: &ConfigData,
) -> AnyhowResult<()> {

    let (receipt, reduction_time) = handle_proof_generation(proof_id).await?;

    let receipt_path= dump_reduction_proof_data(
        config,
        user_circuit_hash,
        &proof_hash,
        receipt,
    )?;
    info!("Dumped reduced proof receipt");

    // update reduction data corresponding to proof
    update_reduction_data(
        get_pool().await,
        proof_id,
        &receipt_path,
        reduction_time,
    )
    .await?;
    info!("Updated reduction data to corresponding proof");
    Ok(())
}

async fn handle_proof_generation(proof_id: u64) ->AnyhowResult<(Receipt, u64)>{
    let proof_data = get_proof_by_proof_id(get_pool().await, proof_id).await?;
    let user_circuit_data = get_user_circuit_data_by_circuit_hash(get_pool().await, &proof_data.user_circuit_hash).await?;

    // Call proof generation to quantum_reduction_circuit
    let (receipt, reduction_time) = generate_reduced_proof(&user_circuit_data, &proof_data).await?;
    let receipt = receipt.unwrap();
    return Ok((receipt, reduction_time))
}

async fn generate_reduced_proof(user_circuit_data: &UserCircuitData, proof_data: &DBProof ) -> AnyhowResult<(Option<Receipt>, u64)> {

    let receipt: Option<Receipt>;
    let reduction_time: u64;

    // if user_circuit_data.proving_scheme == ProvingSchemes::GnarkGroth16 {
        // (prove_result, reduction_time) = generate_gnark_groth16_reduced_proof(user_circuit_data, proof_data, outer_pk_bytes, outer_vk).await?;
    // } else 
    if user_circuit_data.proving_scheme == ProvingSchemes::Groth16 {
        (receipt, reduction_time) = generate_snarkjs_groth16_reduced_proof(user_circuit_data, proof_data).await?;
    } 
    else if user_circuit_data.proving_scheme == ProvingSchemes::Halo2Plonk {
        (receipt, reduction_time) = generate_halo2_plonk_reduced_proof(user_circuit_data, proof_data).await?;
    } 
    else {
        return Err(anyhow!(error_line!("unsupported proving scheme in proof reduction")));
    }

    // let reduction_time = reduction_start_time.elapsed().as_secs();
    info!("Reduced Proof successfully generated in {:?}", reduction_time);
    Ok((receipt, reduction_time))
}

// async fn generate_gnark_groth16_reduced_proof(user_circuit_data: &UserCircuitData, proof_data: &DBProof, outer_pk_bytes: Vec<u8>, outer_vk: GnarkGroth16Vkey) -> AnyhowResult<(GenerateReductionProofResult, u64)> {
//     // Get inner_proof
//     let inner_proof_path = &proof_data.proof_path;
//     println!("inner_proof_path :: {:?}", inner_proof_path);

//     // Get inner_vk
//     let inner_vk_path = &user_circuit_data.vk_path;
//     println!("inner_vk_path :: {:?}", inner_vk_path);

//     // Get inner_pis
//     let inner_pis_path = &proof_data.pis_path;
//     println!("inner_pis_path :: {:?}", inner_pis_path);
//     // 1.Reconstruct inner proof
//     let gnark_inner_proof: GnarkGroth16Proof =
//         GnarkGroth16Proof::read_proof(&inner_proof_path)?;
//     let gnark_inner_vk: GnarkGroth16Vkey = GnarkGroth16Vkey::read_vk(&inner_vk_path)?;
//     let gnark_inner_pis: GnarkGroth16Pis = GnarkGroth16Pis::read_pis(&inner_pis_path)?;

//     let reduction_start_time = Instant::now();

//     // 2.Call reduced proof generator for gnark inner proof
//     let prove_result = QuantumV2CircuitInteractor::generate_gnark_groth16_reduced_proof(
//         gnark_inner_proof,
//         gnark_inner_vk.clone(),
//         gnark_inner_pis.clone(),
//         outer_vk,
//         outer_pk_bytes,
//     );
//     let reduction_time = reduction_start_time.elapsed().as_secs();

//     // verify_proof_reduction_result(&prove_result, &user_circuit_data, gnark_inner_vk, gnark_inner_pis)?;
//     Ok((prove_result, reduction_time))
// }
fn form_snarkjs_groth16_bonsai_inputs(vk: SnarkJSGroth16Vkey, proof: SnarkJSGroth16Proof, pis: SnarkJSGroth16Pis) ->  AnyhowResult<Vec<u8>>{
    let ark_vk = vk.get_ark_vk_for_snarkjs_groth16()?;
    let pvk = verifier::prepare_verifying_key(&ark_vk);

    let ark_proof = proof.get_ark_proof_for_snarkjs_groth16_proof()?;
    let ark_public_inputs = pis.get_ark_pis_for_snarkjs_groth16_pis()?;


    let mut pvk_bytes = vec![];
    pvk.serialize_uncompressed(&mut pvk_bytes)?;

    let mut proof_bytes = vec![];
    ark_proof.serialize_uncompressed(&mut proof_bytes)?;

    let mut public_inputs_bytes = vec![];
    ark_public_inputs.serialize_uncompressed(&mut public_inputs_bytes)?;


    let input_data = to_vec(&pvk_bytes)?;
    let mut input_data_vec: Vec<u8> = bytemuck::cast_slice(&input_data).to_vec();

    let input_data = to_vec(&proof_bytes)?;
    input_data_vec.extend_from_slice( bytemuck::cast_slice(&input_data));


    let input_data = to_vec(&public_inputs_bytes)?;
    input_data_vec.extend_from_slice( bytemuck::cast_slice(&input_data));

    Ok(input_data_vec)
}

async fn  generate_snarkjs_groth16_reduced_proof(user_circuit_data: &UserCircuitData, proof_data: &DBProof) -> AnyhowResult<(Option<Receipt>, u64)> {
    let vk = SnarkJSGroth16Vkey::read_vk(&user_circuit_data.vk_path)?;
    let proof = SnarkJSGroth16Proof::read_proof(&proof_data.proof_path)?;
    let public_inputs = SnarkJSGroth16Pis::read_pis(&proof_data.pis_path)?;

    let input_data_vec = form_snarkjs_groth16_bonsai_inputs(vk, proof, public_inputs)?;
    
    let reduction_start_time = Instant::now();
    let receipt = execute_proof_reduction(input_data_vec, &user_circuit_data.bonsai_image_id, proof_data.id.unwrap()).await?;
    let reduction_time = reduction_start_time.elapsed().as_secs();
    Ok((receipt,reduction_time))
}

fn form_halo2_plonk_bonsai_inputs(proof: &Halo2PlonkProof, vk: &Halo2PlonkVkey, pis: &Halo2PlonkPis) -> AnyhowResult<Vec<u8>> {
    let protocol = vk.get_protocol()?;
    let s_g2 = vk.get_sg2()?;
    let instances = pis.get_instance()?;
    let proof = &proof.proof_bytes;

    let protocol_bytes = to_vec(&protocol)?;
    let s_g2_bytes = to_vec(&s_g2)?;
    let instances_bytes = to_vec(&instances)?;
    let proof_bytes = to_vec(&proof)?;

    let mut input_data_vec: Vec<u8> = bytemuck::cast_slice(&protocol_bytes).to_vec();
    input_data_vec.extend_from_slice(bytemuck::cast_slice(&s_g2_bytes));
    input_data_vec.extend_from_slice(bytemuck::cast_slice(&instances_bytes));
    input_data_vec.extend_from_slice(bytemuck::cast_slice(&proof_bytes));

    Ok(input_data_vec)
}


async fn generate_halo2_plonk_reduced_proof(user_circuit_data: &UserCircuitData, proof_data: &DBProof) -> AnyhowResult<(Option<Receipt>, u64)> {
    // Get inner_proof
    let proof_path = &proof_data.proof_path;
    println!("proof_path :: {:?}", proof_path);

    // Get inner_vk
    let vk_path = &user_circuit_data.vk_path;
    println!("vk_path :: {:?}", vk_path);

    // Get inner_pis
    let pis_path = &proof_data.pis_path;
    println!("pis_path :: {:?}", pis_path);

    let proof = Halo2PlonkProof::read_proof(&proof_path)?;
    let vk = Halo2PlonkVkey::read_vk(&vk_path)?;
    let pis = Halo2PlonkPis::read_pis(&pis_path)?;
    
    let input_data = form_halo2_plonk_bonsai_inputs(&proof, &vk, &pis)?;

    let reduction_start_time = Instant::now();
    let receipt = execute_proof_reduction(input_data, &user_circuit_data.bonsai_image_id, proof_data.id.unwrap()).await?;
    let reduction_time = reduction_start_time.elapsed().as_secs();

    Ok((receipt, reduction_time))
}


// async fn generate_halo2_plonk_reduced_proof(user_circuit_data: &UserCircuitData, proof_data: &DBProof, outer_pk_bytes: Vec<u8>, outer_vk: GnarkGroth16Vkey) -> AnyhowResult<(GenerateReductionProofResult, u64)> {
//     // Get inner_proof
//     let inner_proof_path = &proof_data.proof_path;
//     println!("inner_proof_path :: {:?}", inner_proof_path);

//     // Get inner_vk
//     let inner_vk_path = &user_circuit_data.vk_path;
//     println!("inner_vk_path :: {:?}", inner_vk_path);

//     // Get inner_pis
//     let inner_pis_path = &proof_data.pis_path;
//     println!("inner_pis_path :: {:?}", inner_pis_path);

//     let inner_proof = Halo2PlonkProof::read_proof(&inner_proof_path)?;
//     let inner_vk = Halo2PlonkVkey::read_vk(&inner_vk_path)?;
//     let inner_pis = Halo2PlonkPis::read_pis(&inner_pis_path)?;
    
//     let reduction_start_time = Instant::now();
//     let prove_result = QuantumV2CircuitInteractor::generate_halo2_plonk_reduced_proof(
//         inner_pis.clone(),
//         inner_proof,
//         inner_vk.clone(),
//         outer_vk,
//         outer_pk_bytes,
//     );
//     let reduction_time = reduction_start_time.elapsed().as_secs();

//     // verify_proof_reduction_result(&prove_result, &user_circuit_data, inner_vk, inner_pis)?;
//     Ok((prove_result, reduction_time))
// }

async fn generate_gnark_plonk_reduced_proof(user_circuit_data: &UserCircuitData, proof_data: &DBProof, outer_pk_bytes: Vec<u8>, outer_vk: GnarkGroth16Vkey) -> AnyhowResult<(GenerateReductionProofResult, u64)> {
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
    let inner_proof = GnarkPlonkSolidityProof::read_proof(&inner_proof_path)?;
    let inner_vk = GnarkPlonkVkey::read_vk(&inner_vk_path)?;
    let inner_pis = GnarkPlonkPis::read_pis(&inner_pis_path)?;

    let reduction_start_time = Instant::now();

    let prove_result = QuantumV2CircuitInteractor::generate_gnark_plonk_reduced_proof(
        inner_proof,
        inner_vk.clone(),
        inner_pis.clone(),
        outer_vk,
        outer_pk_bytes,
        AVAIL_BH
    );
    let reduction_time = reduction_start_time.elapsed().as_secs();

    verify_proof_reduction_result(&prove_result, &user_circuit_data, inner_vk, inner_pis)?;
    Ok((prove_result, reduction_time))
}

// fn verify_proof_reduction_result<V: Vkey, P: Pis>(prove_result: &GenerateReductionProofResult, user_circuit_data: &UserCircuitData, inner_vk: V, inner_pis: P) -> AnyhowResult<()>{
//     let mut keccak_ip = Vec::<u8>::new();
//     let vkey_hash = inner_vk.extended_keccak_hash(user_circuit_data.n_commitments)?;
//     println!("vkey_hash {:?}", vkey_hash);
//     keccak_ip.extend(vkey_hash);
//     let pis_hash = inner_pis.extended_keccak_hash()?;
//     println!("pis_hash {:?}", pis_hash);
//     keccak_ip.extend(pis_hash);
//     let hash = keccak_hash::keccak(keccak_ip).0;
//     let pis1 = BigUint::from_bytes_be(&hash[0..16]).to_string();
//     let pis2 = BigUint::from_bytes_be(&hash[16..32]).to_string();
//     println!("pis1 {:?}", pis1);
//     println!("pis2 {:?}", pis2);
//     if prove_result.success {
//         // worker will panic here
//         assert_eq!(pis1, prove_result.reduced_pis.0[0]);
//         assert_eq!(pis2, prove_result.reduced_pis.0[1]);
// fn verify_proof_reduction_result<V: Vkey, P: Pis>(prove_result: &GenerateReductionProofResult, user_circuit_data: &UserCircuitData, inner_vk: V, inner_pis: P) -> AnyhowResult<()>{
//     let mut keccak_ip = Vec::<u8>::new();
//     let vkey_hash = inner_vk.extended_keccak_hash(user_circuit_data.n_commitments)?;
//     println!("vkey_hash {:?}", vkey_hash);
//     keccak_ip.extend(vkey_hash);
//     let pis_hash = inner_pis.extended_keccak_hash()?;
//     println!("pis_hash {:?}", pis_hash);
//     keccak_ip.extend(pis_hash);
//     let hash = keccak_hash::keccak(keccak_ip).0;
//     let pis1 = BigUint::from_bytes_be(&hash[0..16]).to_string();
//     let pis2 = BigUint::from_bytes_be(&hash[16..32]).to_string();
//     println!("pis1 {:?}", pis1);
//     println!("pis2 {:?}", pis2);
//     if prove_result.success {
//         // worker will panic here
//         assert_eq!(pis1, prove_result.reduced_pis.0[0]);
//         assert_eq!(pis2, prove_result.reduced_pis.0[1]);
//     }
//     Ok(())
// }

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    #[tokio::test]
    #[ignore]
    pub async fn test_proof_reduction_by_proof_hash() {
        // NOTE: it connect to database mentioned in the env file, to connect to the test db use .env.test file
        // dotenv::from_filename("../.env.test").ok();
        dotenv().ok();
        let proof_id = 2; // change the proof id
        let (result, reduction_time) = handle_proof_generation(proof_id).await.unwrap();
        println!("{:?}", result);
        // assert_eq!(result.success, true);
    }
}
