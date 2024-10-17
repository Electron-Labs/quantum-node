use std::{fs::{self, File}, io::{BufWriter, Read}, path::PathBuf, time::{Duration, Instant}};

use agg_core::{inputs::get_agg_inputs, types::AggInputs};
use anyhow::Result as AnyhowResult;
use bonsai_sdk::responses::SnarkReceipt;
use num_bigint::BigUint;
use quantum_circuits_interface::ffi::circuit_builder::{CircomProof, CircomVKey, CircuitBuilder, CircuitBuilderImpl, GnarkVKey, ProveResult, Risc0SnarkProveArgs};
use quantum_db::repository::{
    bonsai_image::get_aggregate_circuit_bonsai_image, superproof_repository::{
        get_last_verified_superproof, update_previous_superproof_root, update_superproof_agg_time, update_superproof_leaves_path, update_superproof_pis_path, update_superproof_proof_path, update_superproof_receipts_path, update_superproof_root, update_superproof_total_proving_time
    }, user_circuit_data_repository::get_user_circuit_data_by_circuit_hash
};
use quantum_types::{
    enums::proving_schemes::ProvingSchemes,
    traits::{pis::Pis, proof::Proof, vkey::Vkey},
    types::{
        config::ConfigData, db::proof::Proof as DBProof, gnark_groth16::{GnarkGroth16Pis, GnarkGroth16Proof, GnarkGroth16Vkey, SuperproofGnarkGroth16Proof}, gnark_plonk::{GnarkPlonkPis, GnarkPlonkVkey}, halo2_plonk::{Halo2PlonkPis, Halo2PlonkVkey}, halo2_poseidon::{Halo2PoseidonPis, Halo2PoseidonVkey}, plonk2::{Plonky2Pis, Plonky2Vkey}, riscs0::{Risc0Pis, Risc0Vkey}, snarkjs_groth16::{SnarkJSGroth16Pis, SnarkJSGroth16Vkey}, sp1::{Sp1Pis, Sp1Vkey}
    },
};
use quantum_utils::{
    error_line, file::{dump_object, read_bytes_from_file, read_file, write_bytes_to_file}, keccak::encode_keccak_hash, paths::{get_cs_bytes_path, get_inner_vkey_path, get_snark_reduction_pk_bytes_path, get_snark_reduction_vk_path, get_superproof_leaves_path, get_superproof_pis_path, get_superproof_proof_path, get_superproof_proof_receipt_path, get_superproof_snark_receipt_path, get_user_vk_path}
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

    // TODO: add all object inside a single object
    let (receipt, snark_receipt, aggregation_result, aggregation_time) = handle_proof_aggregation(proofs.clone(), superproof_id, config).await?;
    info!("aggregation done in time : {:?}", aggregation_time);

    if !aggregation_result.pass {
        return Err(anyhow::Error::msg(error_line!(aggregation_result.msg)));
    }
    // TODO: Dump superproof receipt and add to the DB
    let superproof_receipt_path = get_superproof_proof_receipt_path(
        &config.storage_folder_path,
        &config.supperproof_path,
        superproof_id,
    );
    dump_object(receipt.unwrap(), &superproof_receipt_path)?;

    let superproof_snark_receipt_path = get_superproof_snark_receipt_path(
        &config.storage_folder_path,
        &config.supperproof_path,
        superproof_id,
    );
    dump_object(snark_receipt, &superproof_snark_receipt_path)?;
    // superproof_proof.dump_proof(&superproof_proof_path)?;

    let superproof_proof = SuperproofGnarkGroth16Proof::from_risc0_gnark_proof_result(aggregation_result.proof);
    let superproof_pis = GnarkGroth16Pis(aggregation_result.pub_inputs);

    let superproof_proof_path = get_superproof_proof_path(&config.storage_folder_path, &config.supperproof_path, superproof_id);
    let superproof_pis_path = get_superproof_pis_path(&config.storage_folder_path, &config.supperproof_path, superproof_id);

    superproof_proof.dump_proof(&superproof_proof_path)?;
    superproof_pis.dump_pis(&superproof_pis_path)?;

    // TODO: Add new field superproof receipt path
    update_superproof_proof_path(get_pool().await, &superproof_proof_path, superproof_id).await?;

    update_superproof_pis_path(get_pool().await, &superproof_pis_path, superproof_id).await?;
    // Add agg_time to the db
    update_superproof_agg_time(get_pool().await, aggregation_time.as_secs(), superproof_id).await?;

    update_superproof_receipts_path(get_pool().await, &superproof_receipt_path, &superproof_snark_receipt_path, superproof_id).await?;
    

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
            ProvingSchemes::GnarkGroth16 => {
                let protocol_vkey = GnarkGroth16Vkey::read_vk(&protocol_circuit_vkey_path)?;
                protocol_vkey_hashes.push(protocol_vkey.keccak_hash()?);

                let protocol_pis = GnarkGroth16Pis::read_pis(&protocol_pis_path)?;
                protocol_pis_hashes.push(protocol_pis.keccak_hash()?);
                protocol_ids.push(3);
            }
            ProvingSchemes::GnarkPlonk => {
                let protocol_vkey = GnarkPlonkVkey::read_vk(&protocol_circuit_vkey_path)?;
                protocol_vkey_hashes.push(protocol_vkey.keccak_hash()?);

                let protocol_pis = GnarkPlonkPis::read_pis(&protocol_pis_path)?;
                protocol_pis_hashes.push(protocol_pis.keccak_hash()?);
                protocol_ids.push(4);
            }
            ProvingSchemes::Plonky2 => {
                let protocol_vkey = Plonky2Vkey::read_vk(&protocol_circuit_vkey_path)?;
                protocol_vkey_hashes.push(protocol_vkey.keccak_hash()?);

                let protocol_pis = Plonky2Pis::read_pis(&protocol_pis_path)?;
                protocol_pis_hashes.push(protocol_pis.keccak_hash()?);
                protocol_ids.push(2);
            },
            ProvingSchemes::Halo2Poseidon => {
                let protocol_vkey = Halo2PoseidonVkey::read_vk(&protocol_circuit_vkey_path)?;
                protocol_vkey_hashes.push(protocol_vkey.keccak_hash()?);

                let protocol_pis = Halo2PoseidonPis::read_pis(&protocol_pis_path)?;
                protocol_pis_hashes.push(protocol_pis.keccak_hash()?);
                protocol_ids.push(5);
            },
            ProvingSchemes::Risc0 => {
                let protocol_vkey = Risc0Vkey::read_vk(&protocol_circuit_vkey_path)?;
                protocol_vkey_hashes.push(protocol_vkey.keccak_hash()?);

                let protocol_pis = Risc0Pis::read_pis(&protocol_pis_path)?;
                protocol_pis_hashes.push(protocol_pis.keccak_hash()?);
                protocol_ids.push(6);
            },
            ProvingSchemes::Sp1 => {
                let protocol_vkey = Sp1Vkey::read_vk(&protocol_circuit_vkey_path)?;
                protocol_vkey_hashes.push(protocol_vkey.keccak_hash()?);

                let protocol_pis = Sp1Pis::read_pis(&protocol_pis_path)?;
                protocol_pis_hashes.push(protocol_pis.keccak_hash()?);
                protocol_ids.push(7);
            },
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
    let old_root = last_verified_superproof.unwrap().superproof_root.unwrap();
    // let old_root: &str = "";
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

    println!("inner_proof: {:?}", inner_proof);
    let inner_vkey_path = get_inner_vkey_path(&config.storage_folder_path, &config.snark_reduction_data_path);
    let inner_v_key: CircomVKey = read_file(&inner_vkey_path)?;

    let cs_bytes = read_bytes_from_file(&get_cs_bytes_path(&config.storage_folder_path, &config.snark_reduction_data_path))?;

    let pk_bytes = read_bytes_from_file(&get_snark_reduction_pk_bytes_path(&config.storage_folder_path, &config.snark_reduction_data_path))?;

    let v_key: GnarkVKey = read_file(&get_snark_reduction_vk_path(&config.storage_folder_path, &config.snark_reduction_data_path))?;

    let mut circuit_verifying_id_bytes = vec![];
    for i in circuit_verifying_id {
        circuit_verifying_id_bytes.extend_from_slice(&i.to_le_bytes());
    }

    println!("circuit_verifying_id_bytes: {:?}", circuit_verifying_id_bytes);
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
        A: vec![a0,a1,"1".to_string()],
        B: vec![vec![b00, b01], vec![b10, b11], vec!["1".to_string(), "0".to_string()]],
        C: vec![c0,c1,"1".to_string()],
        Protocol: "groth16".to_string(),
        Curve: "bn128".to_string(),
    }
}


fn form_bonsai_input_data<H: Hasher + Serialize>(agg_input: AggInputs<H>) -> AnyhowResult<Vec<u8>> {
    let data = to_vec(&agg_input)?;
    let input_data_vec: Vec<u8> = bytemuck::cast_slice(&data).to_vec();
    Ok(input_data_vec)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use quantum_db::repository::proof_repository::get_proofs_in_superproof_id;
    use risc0_zkvm::Groth16Seal;
    // use risc0_zkvm::Groth16Seal;

    // #[tokio::test]
    // #[ignore]
    // pub async fn test_aggregate_proof_by_superproof_id() {
    //     // NOTE: it connect to database mentioned in the env file, to connect to the test db use .env.test file
    //     // dotenv::from_filename("../.env.test").ok();
    //     // dotenv().ok();
    //     let config_data = ConfigData::new("../../config.yaml"); // change the path
    //     let superproof_id = 90; // insert your circuit hash
    //     let superproof = get_superproof_by_id(get_pool().await, superproof_id).await.unwrap();
    //     let proofs = get_proofs_in_superproof_id(get_pool().await,superproof_id).await.unwrap();
    //     let (result, reduction_time) = handle_proof_aggregation(proofs, superproof_id, &config_data).await.unwrap();
    //     println!("{:?}", result);
    //     assert_eq!(result.success, true);
    // }

    #[tokio::test]
    #[ignore]
    pub async fn test_form_circom_proof_from_snark_receipt() {
        let s = SnarkReceipt {
            snark: Groth16Seal { 
                a: [[7, 134, 10, 194, 220, 228, 10, 147, 40, 206, 193, 121, 215, 185, 233, 37, 12, 210, 197, 63, 197, 65, 179, 2, 91, 139, 217, 89, 45, 49, 216, 29].to_vec(), [17, 91, 2, 136, 175, 111, 16, 178, 68, 251, 57, 84, 14, 106, 249, 16, 70, 141, 148, 178, 98, 231, 105, 18, 3, 70, 155, 94, 117, 116, 145, 150].to_vec()].to_vec()
                
                , b: [[[0, 228, 185, 81, 10, 211, 129, 33, 230, 143, 33, 138, 100, 223, 124, 208, 91, 54, 72, 154, 78, 241, 253, 229, 79, 199, 121, 123, 131, 100, 130, 125].to_vec(), [18, 97, 110, 247, 50, 52, 74, 11, 230, 212, 47, 180, 154, 109, 146, 178, 184, 153, 159, 140, 84, 172, 18, 4, 31, 8, 213, 11, 220, 232, 153, 107].to_vec()].to_vec(), [[19, 166, 95, 171, 8, 120, 66, 22, 239, 59, 122, 217, 8, 65, 243, 108, 103, 223, 183, 243, 183, 237, 6, 223, 51, 179, 216, 39, 146, 84, 8, 25].to_vec(), [15, 17, 171, 251, 180, 143, 74, 130, 103, 137, 117, 45, 217, 193, 67, 134, 6, 181, 245, 87, 196, 106, 14, 248, 47, 118, 207, 148, 109, 12, 37, 87].to_vec()].to_vec()].to_vec()
                
                , c:[[17, 175, 26, 146, 194, 183, 124, 180, 212, 2, 35, 15, 247, 235, 250, 183, 162, 197, 198, 39, 76, 1, 121, 3, 107, 105, 141, 31, 134, 76, 72, 248].to_vec(), [20, 11, 194, 53, 122, 101, 218, 241, 130, 184, 225, 18, 220, 224, 69, 91, 181, 95, 47, 123, 173, 16, 241, 244, 61, 136, 245, 165, 236, 160, 177, 194].to_vec()].to_vec()
            
            },


            post_state_digest:  [163, 172, 194, 113, 23, 65, 137, 150, 52, 11, 132, 229, 169, 15, 62, 244, 196, 157, 34, 199, 158, 68, 170, 216, 34, 236, 156, 49, 62, 30, 184, 226].to_vec(),
            journal: [90, 18, 196, 95, 51, 152, 136, 218, 195, 64, 81, 98, 111, 194, 77, 226, 56, 208, 183, 55, 189, 132, 182, 44, 38, 183, 233, 171, 12, 192, 231, 112].to_vec() ,
        };

        let proof = form_circom_proof_from_snark_receipt(&s);
        println!("circom_proof: {:?}", proof);
    }
}