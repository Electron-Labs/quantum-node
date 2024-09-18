use std::{fs::{self, File}, io::{BufWriter, Read}, path::PathBuf, time::{Duration, Instant}};

use agg_core::{inputs::get_agg_inputs, types::AggInputs};
use anyhow::Result as AnyhowResult;
use bonsai_sdk::responses::SnarkReceipt;
use num_bigint::BigUint;
use quantum_circuits_interface::ffi::circuit_builder::{CircomProof, CircomVKey, CircuitBuilder, CircuitBuilderImpl, GnarkVKey, ProveResult, Risc0SnarkProveArgs};
use quantum_db::repository::{
    bonsai_image::get_aggregate_circuit_bonsai_image, superproof_repository::{
        get_last_verified_superproof, update_previous_superproof_root, update_superproof_agg_time, update_superproof_leaves_path, update_superproof_proof_path, update_superproof_root, update_superproof_total_proving_time
    }, user_circuit_data_repository::get_user_circuit_data_by_circuit_hash
};
use quantum_types::{
    enums::proving_schemes::ProvingSchemes,
    traits::{pis::Pis, vkey::Vkey},
    types::{
        config::ConfigData,
        db::proof::Proof as DBProof,
        halo2_plonk::{Halo2PlonkPis, Halo2PlonkVkey},
        snarkjs_groth16::{SnarkJSGroth16Pis, SnarkJSGroth16Vkey},
    },
};
use quantum_utils::{
    error_line, file::{dump_object, read_bytes_from_file, read_file, write_bytes_to_file}, keccak::encode_keccak_hash, paths::{get_cs_bytes_path, get_inner_vkey_path, get_snark_reduction_pk_bytes_path, get_snark_reduction_vk_path, get_superproof_leaves_path, get_superproof_proof_receipt_path, get_superproof_snark_receipt_path, get_user_vk_path}
};
use risc0_zkvm::{serde::to_vec, Receipt};
use serde::Serialize;
use tracing::info;
use utils::hash::{Hasher, KeccakHasher};
use crate::{bonsai::{execute_aggregation, run_stark2snark}, connection::get_pool};
use crate::utils::get_last_superproof_leaves;
// use 
pub async fn handle_proof_aggregation_and_updation(
    proofs: Vec<DBProof>,
    superproof_id: u64,
    config: &ConfigData,
) -> AnyhowResult<()> {

    let (receipt, snark_receipt, aggregation_result, aggregation_time) = handle_proof_aggregation(proofs.clone(), superproof_id, config).await?;
    info!("aggregation done in time : {:?}", aggregation_time);

    if !aggregation_result.pass {
        return Err(anyhow::Error::msg(error_line!(aggregation_result.msg)));
    }
    // TODO: Dump superproof receipt and add to the DB
    let superproof_proof_path = get_superproof_proof_receipt_path(
        &config.storage_folder_path,
        &config.supperproof_path,
        superproof_id,
    );
    dump_object(receipt.unwrap(), &superproof_proof_path)?;

    let superproof_proof_path = get_superproof_snark_receipt_path(
        &config.storage_folder_path,
        &config.supperproof_path,
        superproof_id,
    );
    dump_object(snark_receipt, &superproof_proof_path)?;
    // superproof_proof.dump_proof(&superproof_proof_path)?;

    // TODO: Add new field superproof receipt path
    update_superproof_proof_path(get_pool().await, &superproof_proof_path, superproof_id).await?;

    // Add agg_time to the db
    update_superproof_agg_time(get_pool().await, aggregation_time.as_secs(), superproof_id).await?;

    let proof_with_max_reduction_time = proofs.iter().max_by_key(|proof| proof.reduction_time);
    // TODO: remove unwrap , check reduction time is not getting update in db
    let total_proving_time = proof_with_max_reduction_time
        .unwrap()
        .reduction_time
        .unwrap()
        + aggregation_time.as_secs();
    update_superproof_total_proving_time(get_pool().await, total_proving_time, superproof_id).await?;
    Ok(())
}

async fn handle_proof_aggregation(proofs: Vec<DBProof>, superproof_id: u64, config: &ConfigData) -> AnyhowResult<(Option<Receipt>, SnarkReceipt, ProveResult, Duration)> {
    info!("superproof_id {:?}", superproof_id);
    
    let last_verified_superproof = get_last_verified_superproof(get_pool().await).await?;
    let mut protocol_ids: Vec<u8> = vec![];
    let mut protocol_vkey_hashes: Vec<[u8;32]> = vec![];
    let mut protocol_pis_hashes: Vec<[u8;32]> = vec![];
    let mut assumptions = vec![];
    for proof in &proofs {
        assumptions.push(proof.session_id.clone().unwrap());

        let user_circuit_data = get_user_circuit_data_by_circuit_hash(get_pool().await, &proof.user_circuit_hash).await?;
        let protocol_circuit_vkey_path = user_circuit_data.vk_path;
        let protocol_pis_path = proof.pis_path.clone();

        match user_circuit_data.proving_scheme {
            // ProvingSchemes::GnarkGroth16 => {
            //     let protocol_vkey = GnarkGroth16Vkey::read_vk(&protocol_circuit_vkey_path)?;
            //     protocol_vkey_hashes.push(protocol_vkey.extended_keccak_hash(user_circuit_data.n_commitments)?.to_vec());

            //     let protocol_pis = GnarkGroth16Pis::read_pis(&protocol_pis_path)?;
            //     protocol_pis_hashes.push(protocol_pis.extended_keccak_hash()?.to_vec());
            // }
            ProvingSchemes::Groth16 => {
                let protocol_vkey = SnarkJSGroth16Vkey::read_vk(&protocol_circuit_vkey_path)?;
                protocol_vkey_hashes.push(protocol_vkey.keccak_hash()?);

                let protocol_pis = SnarkJSGroth16Pis::read_pis(&protocol_pis_path)?;
                protocol_pis_hashes.push(protocol_pis.keccak_hash()?);
                protocol_ids.push(0);
                
            }
            ProvingSchemes::Halo2Plonk => {
                let protocol_vkey = Halo2PlonkVkey::read_vk(&protocol_circuit_vkey_path)?;
                protocol_vkey_hashes.push(protocol_vkey.keccak_hash()?);

                let protocol_pis = Halo2PlonkPis::read_pis(&protocol_pis_path)?;
                protocol_pis_hashes.push(protocol_pis.keccak_hash()?);
                protocol_ids.push(1);
            }
            _ => todo!(),
        }
    }

    let last_leaves = get_last_superproof_leaves(config).await?;

    let (agg_input, new_leaves, new_superproof_root ) = get_agg_inputs::<KeccakHasher>(protocol_ids, protocol_vkey_hashes, protocol_pis_hashes, proofs.len(), last_leaves, config.imt_depth)?;

    let superproof_leaves_path = get_superproof_leaves_path(
        &config.storage_folder_path,
        &config.supperproof_path,
        superproof_id,
    );
    // TODO: check this.
    let new_leaves_bytes = bincode::serialize(&new_leaves)?;
    write_bytes_to_file(&new_leaves_bytes, &superproof_leaves_path)?;
    update_superproof_leaves_path(get_pool().await, &superproof_leaves_path, superproof_id).await?;

    //TODO: fix this
    // let old_root = last_verified_superproof.unwrap().previous_superproof_root.unwrap();
    let old_root: &str = "";
    update_previous_superproof_root(get_pool().await, &old_root, superproof_id).await?;

    let new_root = encode_keccak_hash(&new_superproof_root)?;
    update_superproof_root(get_pool().await, &new_root, superproof_id).await?;

    let aggregation_start = Instant::now();

    let input_data = form_bonsai_input_data(agg_input)?;

    //TODO: move it to DB;
    let agg_image = get_aggregate_circuit_bonsai_image(get_pool().await).await?;
    let (receipt, agg_session_id) = execute_aggregation(input_data, &agg_image.image_id, assumptions, superproof_id).await?;
    receipt.clone().unwrap().verify(agg_image.circuit_verifying_id).expect("agg receipt not verified");
    let snark_receipt = run_stark2snark(agg_session_id, superproof_id).await?.unwrap();

    let prove_result = snark_to_gnark_reduction(&snark_receipt, config, agg_image.circuit_verifying_id)?;

    let aggregation_time = aggregation_start.elapsed();
    Ok((receipt, snark_receipt, prove_result, aggregation_time))
}



fn snark_to_gnark_reduction(snark_receipt: &SnarkReceipt, config: &ConfigData, circuit_verifying_id: [u32;8]) -> AnyhowResult<ProveResult> {
    
    let inner_proof = form_circom_proof_from_snark_receipt(snark_receipt);

    let inner_vkey_path = get_inner_vkey_path(&config.storage_folder_path, &config.snark_reduction_data_path);
    let inner_v_key: CircomVKey = read_file(&inner_vkey_path)?;

    let cs_bytes = read_bytes_from_file(&get_cs_bytes_path(&config.storage_folder_path, &config.snark_reduction_data_path))?;

    let pk_bytes = read_bytes_from_file(&get_snark_reduction_pk_bytes_path(&config.storage_folder_path, &config.snark_reduction_data_path))?;

    let v_key: GnarkVKey = read_file(&get_snark_reduction_vk_path(&config.storage_folder_path, &config.snark_reduction_data_path))?;

    let mut circuit_verifying_id_bytes = vec![];
    for i in circuit_verifying_id {
        circuit_verifying_id_bytes.extend_from_slice(&i.to_be_bytes());
    }
    let args = Risc0SnarkProveArgs {
        cs_bytes,
        pk_bytes,
        v_key,
        inner_proof,
        inner_v_key,
        agg_verifier_id: circuit_verifying_id_bytes,
        journal: snark_receipt.journal.clone(),
    };
    let result = CircuitBuilderImpl::prove_risc0_snark(args);
    Ok(result)
}


fn form_circom_proof_from_snark_receipt(snark_receipt: &SnarkReceipt) -> CircomProof {
    let a0 = BigUint::from_bytes_be(&snark_receipt.snark.a[0]).to_string();
    let a1 = BigUint::from_bytes_be(&snark_receipt.snark.a[1]).to_string();
    
    let b00 = BigUint::from_bytes_be(&snark_receipt.snark.b[0][1]).to_string();
    let b01 = BigUint::from_bytes_be(&snark_receipt.snark.b[0][0]).to_string();

    let b10 = BigUint::from_bytes_be(&snark_receipt.snark.b[1][1]).to_string();
    let b11 = BigUint::from_bytes_be(&snark_receipt.snark.b[1][0]).to_string();

    let c1 = BigUint::from_bytes_be(&snark_receipt.snark.c[1]).to_string();
    let c0 = BigUint::from_bytes_be(&snark_receipt.snark.c[0]).to_string();

    CircomProof {
        A: vec![a0,a1],
        B: vec![vec![b00, b01], vec![b10, b11]],
        C: vec![c0,c1],
        Protocol: "groth16".to_string(),
        Curve: "bn128".to_string(),
    }
}


fn form_bonsai_input_data<H: Hasher + Serialize>(agg_input: AggInputs<H>) -> AnyhowResult<Vec<u8>> {
    let data = to_vec(&agg_input)?;
    let input_data_vec: Vec<u8> = bytemuck::cast_slice(&data).to_vec();
    Ok(input_data_vec)
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use dotenv::dotenv;
//     use quantum_db::repository::proof_repository::get_proofs_in_superproof_id;

//     #[tokio::test]
//     #[ignore]
//     pub async fn test_aggregate_proof_by_superproof_id() {
//         // NOTE: it connect to database mentioned in the env file, to connect to the test db use .env.test file
//         // dotenv::from_filename("../.env.test").ok();
//         // dotenv().ok();
//         let config_data = ConfigData::new("../../config.yaml"); // change the path
//         let superproof_id = 90; // insert your circuit hash
//         let superproof = get_superproof_by_id(get_pool().await, superproof_id).await.unwrap();
//         let proofs = get_proofs_in_superproof_id(get_pool().await,superproof_id).await.unwrap();
//         let (result, reduction_time) = handle_proof_aggregation(proofs, superproof_id, &config_data).await.unwrap();
//         println!("{:?}", result);
//         assert_eq!(result.success, true);
//     }
// }