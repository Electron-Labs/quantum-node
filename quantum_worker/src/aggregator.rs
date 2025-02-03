use std::{sync::Arc,
    time::{Duration, Instant}};

use crate::{
    bonsai::{execute_aggregation_with_retry, run_stark2snark_with_retry},
    connection::get_pool,
};
use agg_core::{inputs::get_agg_inputs, types::AggInputs};
use anyhow::{anyhow, Result as AnyhowResult};
use num_bigint::BigUint;
use quantum_circuits_interface::ffi::circuit_builder::{
    self, CircomProof, CircomVKey, CircuitBuilder, CircuitBuilderImpl, GnarkProveArgs, ProveResult,
    Risc0Data, Sp1Data, G1, G1A, G2,
};
use quantum_db::repository::{
    bonsai_image::get_aggregate_circuit_bonsai_image,
    superproof_repository::{
        update_cycles_in_superproof, update_r0_leaves_path,
        update_r0_receipts_path, update_r0_root, update_sp1_leaves_path, update_sp1_root,
        update_sp1_snark_receipt_path, update_superproof_agg_time, update_superproof_pis_path,
        update_superproof_proof_path, update_superproof_root, update_superproof_total_proving_time,
    },
    user_circuit_data_repository::get_user_circuit_data_by_circuit_hash,
};
use quantum_types::{
    enums::proving_schemes::ProvingSchemes,
    traits::{pis::Pis, proof::Proof, vkey::Vkey},
    types::{
        config::ConfigData,
        db::proof::Proof as DBProof,
        gnark_groth16::{GnarkGroth16Pis, GnarkGroth16Vkey, SuperproofGnarkGroth16Proof},
        gnark_plonk::{GnarkPlonkPis, GnarkPlonkVkey},
        halo2_plonk::{Halo2PlonkPis, Halo2PlonkVkey},
        halo2_poseidon::{Halo2PoseidonPis, Halo2PoseidonVkey},
        plonk2::{Plonky2Pis, Plonky2Vkey},
        riscs0::{Risc0Pis, Risc0Vkey},
        snarkjs_groth16::{SnarkJSGroth16Pis, SnarkJSGroth16Vkey},
        sp1::{Sp1Proof, Sp1Vkey},
    },
};
use quantum_utils::{
    error_line,
    file::{dump_object, read_bytes_from_file, read_file, write_bytes_to_file},
    keccak::encode_keccak_hash,
    paths::{
        get_aggregated_r0_proof_receipt_path, get_aggregated_r0_snark_receipt_path, get_aggregated_sp1_snark_receipt_path, get_cs_bytes_path, get_inner_vkey_path, get_r0_aggregate_leaves_path, get_snark_reduction_pk_bytes_path, get_snark_reduction_vk_path, get_sp1_agg_pk_bytes_path, get_sp1_agg_vk_hash_bytes_path, get_sp1_aggregate_leaves_path, get_sp1_empty_proof_path, get_superproof_pis_path, get_superproof_proof_path
    },
};
use risc0_zkvm::{serde::to_vec, Receipt};
use serde::Serialize;
use sp1_sdk::{ProverClient, SP1ProofWithPublicValues, SP1ProvingKey, SP1VerifyingKey};
use tracing::info;
use utils::hash::{Hasher, KeccakHasher};

// superroot = digest( risc0_root || sp1_root )
pub fn get_superroot(risc0_root: &Vec<u8>, sp1_root: &Vec<u8>) -> [u8; 32] {
    KeccakHasher::combine_hash(risc0_root, sp1_root)
}

pub async fn handle_proof_aggregation_and_updation(
    proofs_r0: Vec<DBProof>,
    proofs_sp1: Vec<DBProof>,
    superproof_id: u64,
    config: &ConfigData,
) -> AnyhowResult<()> {
    info!("superproof_id {:?}", superproof_id);

    let config_clone = config.clone();
    let config_clone_sp1 = config.clone();
    let proof_r0_clone = proofs_r0.clone();
    let risc0_handle = tokio::spawn(  async move  {handle_proof_aggregation_r0(proof_r0_clone, superproof_id, &config_clone).await});
    let sp1_handle = tokio::spawn( async move {handle_proof_aggregation_sp1(proofs_sp1.clone(), superproof_id, &config_clone_sp1).await});
    let risc0_aggregation = risc0_handle.await??;
    let r0_receipt = risc0_aggregation.0;
    let r0_snark_receipt = risc0_aggregation.1;
    let r0_root_bytes = risc0_aggregation.2;
    let r0_aggregation_time = risc0_aggregation.3;

    let sp1_aggregation = sp1_handle.await??;
    let sp1_snark_proof = sp1_aggregation.0;
    let sp1_root_bytes = sp1_aggregation.1;
    let sp1_aggregation_time = sp1_aggregation.2;

    // sp1_snark_proof.save(&aggregated_sp1_snark_receipt_path)?;
    info!(
        "individual aggregations done in time : {:?}",
        r0_aggregation_time + sp1_aggregation_time
    );

    let agg_image = get_aggregate_circuit_bonsai_image(get_pool().await).await?;

    let r0_root = encode_keccak_hash(&r0_root_bytes)?;
    let sp1_root = encode_keccak_hash(&sp1_root_bytes)?;
    info!("r0_root {:?}", r0_root);
    info!("sp1_root {:?}", sp1_root);

    update_r0_root(get_pool().await, &r0_root, superproof_id).await?;
    update_sp1_root(get_pool().await, &sp1_root, superproof_id).await?;

    let gnark_combination_start = Instant::now();

    let prove_result = snark_to_gnark_reduction(
        &r0_snark_receipt,
        &sp1_snark_proof,
        config,
        agg_image.circuit_verifying_id,
    )?;

    let superroot = encode_keccak_hash(&get_superroot(
        &r0_root_bytes.to_vec(),
        &sp1_root_bytes.to_vec(),
    ))?;
    info!("superoot {:?}", superroot);
    update_superproof_root(get_pool().await, &superroot, superproof_id).await?;

    let total_aggregation_time =
        gnark_combination_start.elapsed() + r0_aggregation_time + sp1_aggregation_time;

    if !prove_result.pass {
        return Err(anyhow::Error::msg(error_line!(prove_result.msg)));
    }

    let aggregated_r0_receipt_path = get_aggregated_r0_proof_receipt_path(
        &config.storage_folder_path,
        &config.supperproof_path,
        superproof_id,
    );
    dump_object(r0_receipt.unwrap(), &aggregated_r0_receipt_path)?;

    let aggregated_r0_snark_receipt_path = get_aggregated_r0_snark_receipt_path(
        &config.storage_folder_path,
        &config.supperproof_path,
        superproof_id,
    );
    dump_object(r0_snark_receipt, &aggregated_r0_snark_receipt_path)?;

    let aggregated_sp1_snark_receipt_path = get_aggregated_sp1_snark_receipt_path(
        &config.storage_folder_path,
        &config.supperproof_path,
        superproof_id,
    );
    sp1_snark_proof.save(&aggregated_sp1_snark_receipt_path)?;

    let superproof_proof = SuperproofGnarkGroth16Proof::from_gnark_proof_result(prove_result.proof);
    let superproof_pis = GnarkGroth16Pis(prove_result.pub_inputs);

    let superproof_proof_path = get_superproof_proof_path(
        &config.storage_folder_path,
        &config.supperproof_path,
        superproof_id,
    );
    let superproof_pis_path = get_superproof_pis_path(
        &config.storage_folder_path,
        &config.supperproof_path,
        superproof_id,
    );

    superproof_proof.dump_proof(&superproof_proof_path)?;
    superproof_pis.dump_pis(&superproof_pis_path)?;

    // TODO: Add new field superproof receipt path
    update_superproof_proof_path(get_pool().await, &superproof_proof_path, superproof_id).await?;

    update_superproof_pis_path(get_pool().await, &superproof_pis_path, superproof_id).await?;
    // Add agg_time to the db
    update_superproof_agg_time(
        get_pool().await,
        total_aggregation_time.as_secs(),
        superproof_id,
    )
    .await?;

    update_r0_receipts_path(
        get_pool().await,
        &aggregated_r0_receipt_path,
        &aggregated_r0_snark_receipt_path,
        superproof_id,
    )
    .await?;

    // update sp1 snark receipt path
    update_sp1_snark_receipt_path(
        get_pool().await,
        &aggregated_sp1_snark_receipt_path,
        superproof_id,
    )
    .await?;

    // We only do reduction for r0 proofs
    let proof_with_max_reduction_time = proofs_r0.iter().max_by_key(|proof| proof.reduction_time);
    let max_reduction_time = proof_with_max_reduction_time.map(|proof| proof.reduction_time).unwrap_or(Some(0)).unwrap_or(0);

    // TODO: remove unwrap , check reduction time is not getting update in db
    let total_proving_time = max_reduction_time + total_aggregation_time.as_secs();
    update_superproof_total_proving_time(get_pool().await, total_proving_time, superproof_id)
        .await?;
    Ok(())
}

async fn handle_proof_aggregation_sp1(
    proofs: Vec<DBProof>,
    superproof_id: u64,
    config: &ConfigData,
) -> AnyhowResult<(SP1ProofWithPublicValues, [u8; 32], Duration)> {
    if proofs.len() == 0 {
        return handle_no_sp1_proof_aggregation(config);
    }

    println!("inside the sp1 aggregation");
    let mut protocol_vkeys: Vec<SP1VerifyingKey> = vec![];
    let mut deserialised_proofs: Vec<SP1ProofWithPublicValues> = vec![];
    for proof in &proofs {
        let user_circuit_data =
            get_user_circuit_data_by_circuit_hash(get_pool().await, &proof.user_circuit_hash)
                .await?;
        let protocol_circuit_vkey_path = user_circuit_data.vk_path;
        let protocol_proof_path = proof.proof_path.clone();
        // Proving Scheme for these will always be sp1
        let protocol_vkey = Sp1Vkey::read_vk(&protocol_circuit_vkey_path)?.get_verifying_key()?;
        let deserialised_proof =
            Sp1Proof::read_proof(&protocol_proof_path)?.get_proof_with_public_inputs()?;
        protocol_vkeys.push(protocol_vkey);
        deserialised_proofs.push(deserialised_proof);
    }

    println!("before sp1 agg input");
    let (stdin, leaves, root) =
        crate::utils::get_agg_inputs_sp1::<KeccakHasher>(protocol_vkeys, deserialised_proofs)?;
    println!("after sp1 agg input");
    let sp1_aggregate_leaves_path = get_sp1_aggregate_leaves_path(
        &config.storage_folder_path,
        &config.supperproof_path,
        superproof_id,
    );

    let leaves_serialized = bincode::serialize(&leaves)?;
    println!("after sp1 leave serailise");
    write_bytes_to_file(&leaves_serialized, &sp1_aggregate_leaves_path)?;
    update_sp1_leaves_path(get_pool().await, &sp1_aggregate_leaves_path, superproof_id).await?;

    let aggregation_start = Instant::now();

    // Execute Aggregation for sp1
    let proving_key_path = get_sp1_agg_pk_bytes_path(
        &config.storage_folder_path,
        &config.sp1_snark_reduction_data_path,
    ); // TODO: Add this path to config and read from there
    println!("agg_pk deserialise");
    let aggregation_pk: SP1ProvingKey =
        bincode::deserialize_from(std::fs::File::open(proving_key_path)?)?;

    let client = ProverClient::new();
    let aggregated_proof = client.prove(&aggregation_pk, &stdin).groth16().run()?;
    println!("Received SP1 proof");

    let aggregation_time = aggregation_start.elapsed();
    // TODO: dump sp1 leaves like r0 too here
    // return sp1 root bytes [u8;32] from here too
    Ok((aggregated_proof, root, aggregation_time))
}

fn handle_no_sp1_proof_aggregation(config: &ConfigData) -> AnyhowResult<(SP1ProofWithPublicValues, [u8; 32], Duration)>{
    info!("No new sp1 proofs, using old aggregated_sp1_snark_receipt");
    // use hardocoded aggregated_sp1_snark_receipt_path
    let aggregated_sp1_snark_receipt_path = get_sp1_empty_proof_path(&config.storage_folder_path, &config.sp1_folder_path);
    let sp1_snark_proof = SP1ProofWithPublicValues::load(aggregated_sp1_snark_receipt_path)?;
    let sp1_aggregation_time = Duration::ZERO;
    let mut sp1_root_bytes = [0u8; 32];
    sp1_root_bytes.copy_from_slice(&sp1_snark_proof.public_values.as_slice());
    Ok((sp1_snark_proof, sp1_root_bytes, sp1_aggregation_time))
}

async fn handle_proof_aggregation_r0(
    proofs: Vec<DBProof>,
    superproof_id: u64,
    config: &ConfigData,
) -> AnyhowResult<(Option<Receipt>, Receipt, [u8; 32], Duration)> {
    info!("superproof_id {:?}", superproof_id);
    info!("inside risc0 proof aggregation");

    // let last_verified_superproof = get_last_verified_superproof(get_pool().await).await?;
    let mut protocol_ids: Vec<u8> = vec![];
    let mut protocol_vkey_hashes: Vec<[u8; 32]> = vec![];
    let mut protocol_pis_hashes: Vec<[u8; 32]> = vec![];
    let mut assumptions = vec![];
    for proof in &proofs {
        assumptions.push(proof.session_id.clone().unwrap());

        let user_circuit_data =
            get_user_circuit_data_by_circuit_hash(get_pool().await, &proof.user_circuit_hash)
                .await?;
        let protocol_circuit_vkey_path = user_circuit_data.vk_path;
        let protocol_pis_path = proof.pis_path.clone();

        match user_circuit_data.proving_scheme {
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
            }
            ProvingSchemes::Halo2Poseidon => {
                let protocol_vkey = Halo2PoseidonVkey::read_vk(&protocol_circuit_vkey_path)?;
                protocol_vkey_hashes.push(protocol_vkey.keccak_hash()?);

                let protocol_pis = Halo2PoseidonPis::read_pis(&protocol_pis_path)?;
                protocol_pis_hashes.push(protocol_pis.keccak_hash()?);
                protocol_ids.push(5);
            }
            ProvingSchemes::Risc0 => {
                let protocol_vkey = Risc0Vkey::read_vk(&protocol_circuit_vkey_path)?;
                protocol_vkey_hashes.push(protocol_vkey.keccak_hash()?);

                let protocol_pis = Risc0Pis::read_pis(&protocol_pis_path)?;
                protocol_pis_hashes.push(protocol_pis.keccak_hash()?);
                protocol_ids.push(6);
            }
            _ => {
                panic!("Shouldnt happen!!")
            }
        }
    }

    let (agg_input, leaves, batch_root_bytes) = get_agg_inputs::<KeccakHasher>(
        protocol_ids,
        protocol_vkey_hashes,
        protocol_pis_hashes,
        proofs.len(),
    )?;

    let r0_aggregate_leaves_path = get_r0_aggregate_leaves_path(
        &config.storage_folder_path,
        &config.supperproof_path,
        superproof_id,
    );

    let leaves_serialized = bincode::serialize(&leaves)?;
    write_bytes_to_file(&leaves_serialized, &r0_aggregate_leaves_path)?;
    update_r0_leaves_path(get_pool().await, &r0_aggregate_leaves_path, superproof_id).await?;

    // Update r0 root
    let aggregation_start = Instant::now();

    let input_data = form_bonsai_input_data(agg_input)?;

    //TODO: move it to DB;
    let agg_image = get_aggregate_circuit_bonsai_image(get_pool().await).await?;
    let (receipt, agg_session_id, agg_cycle_used) =
    execute_aggregation_with_retry(&input_data, &agg_image.image_id, &assumptions, superproof_id).await?;

    let total_cycle_used = calc_total_cycle_used(agg_cycle_used, &proofs);
    update_cycles_in_superproof(
        get_pool().await,
        agg_cycle_used,
        total_cycle_used,
        superproof_id,
    )
    .await?;

    let snark_receipt = run_stark2snark_with_retry(&agg_session_id, superproof_id)
        .await?
        .unwrap();
    println!("snark receipt: {:?}", snark_receipt);

    let aggregation_time = aggregation_start.elapsed();
    Ok((receipt, snark_receipt, batch_root_bytes, aggregation_time))
}

fn calc_total_cycle_used(agg_cycle_used: u64, proofs: &[DBProof]) -> u64 {
    let proof_cycles: u64 = proofs.iter().map(|item| item.cycle_used.unwrap_or(0)).sum();
    agg_cycle_used + proof_cycles
}

fn snark_to_gnark_reduction(
    risc0_snark: &Receipt,
    sp1_snark: &SP1ProofWithPublicValues,
    config: &ConfigData,
    risc0_agg_image_id: [u32; 8],
) -> AnyhowResult<ProveResult> {
    let cs_bytes = read_bytes_from_file(&get_cs_bytes_path(
        &config.storage_folder_path,
        &config.risc0_snark_reduction_data_path,
    ))?;
    let pk_bytes = read_bytes_from_file(&get_snark_reduction_pk_bytes_path(
        &config.storage_folder_path,
        &config.risc0_snark_reduction_data_path,
    ))?;
    let v_key: circuit_builder::GnarkGroth16VKey = read_file(&get_snark_reduction_vk_path(
        &config.storage_folder_path,
        &config.risc0_snark_reduction_data_path,
    ))?;

    let risc0_inner_proof = form_circom_proof_from_snark_receipt(risc0_snark)?;
    let risc0_inner_vkey_path = get_inner_vkey_path(
        &config.storage_folder_path,
        &config.risc0_snark_reduction_data_path,
    );
    let risc0_inner_v_key: CircomVKey = read_file(&risc0_inner_vkey_path)?;

    let sp1_snark_proof = sp1_snark
        .proof
        .clone()
        .try_as_groth_16()
        .ok_or(anyhow!("try_as_groth_16 failed"))?;
    let sp1_inner_proof = sp1_snark_proof_to_ffi_type(sp1_snark_proof.encoded_proof)?;
    let sp1_inner_vkey_path = get_inner_vkey_path(
        &config.storage_folder_path,
        &config.sp1_snark_reduction_data_path,
    );
    let sp1_inner_v_key: circuit_builder::GnarkGroth16VKey = read_file(&sp1_inner_vkey_path)?;

    let sp1_agg_program_v_key = read_bytes_from_file(&get_sp1_agg_vk_hash_bytes_path(
        &config.storage_folder_path,
        &config.sp1_snark_reduction_data_path,
    ))?;

    let mut risc0_agg_image_id_bytes = vec![];
    for x in risc0_agg_image_id {
        risc0_agg_image_id_bytes.extend_from_slice(&x.to_le_bytes());
    }

    println!("risc0_snark.journal {:?}", risc0_snark.journal.bytes);
    println!("risc0_agg_image_id_bytes {:?}", risc0_agg_image_id_bytes.clone());
    println!("sp1_snark.public_values {:?}", sp1_snark.public_values.to_vec());
    println!("sp1_agg_program_v_key {:?}", sp1_agg_program_v_key.clone());

    let risc0_data = Risc0Data {
        inner_proof: risc0_inner_proof,
        inner_v_key: risc0_inner_v_key,
        journal: risc0_snark.journal.bytes.clone(),
        agg_image_id: risc0_agg_image_id_bytes,
    };
    let sp1_data = Sp1Data {
        inner_proof: sp1_inner_proof,
        inner_v_key: sp1_inner_v_key,
        public_values: sp1_snark.public_values.to_vec(),
        agg_program_v_key: sp1_agg_program_v_key,
    };


    let args = GnarkProveArgs {
        cs_bytes,
        pk_bytes,
        v_key,
        risc0_data,
        sp1_data,
    };
    let result = CircuitBuilderImpl::prove_gnark(args);
    println!("prove_gnark result {:?}", result.proof);
    println!("prove_gnark pub_inputs {:?}", result.pub_inputs);
    Ok(result)
}

fn form_circom_proof_from_snark_receipt(snark_receipt: &Receipt) -> AnyhowResult<CircomProof> {
    let mut ptr = 0;
    let bytes = snark_receipt.inner.groth16()?.seal.clone();
    let a0 = BigUint::from_bytes_be(&bytes[ptr..ptr + 32]).to_string();
    ptr += 32;
    let a1 = BigUint::from_bytes_be(&bytes[ptr..ptr + 32]).to_string();
    ptr += 32;

    let b01 = BigUint::from_bytes_be(&bytes[ptr..ptr + 32]).to_string();
    ptr += 32;
    let b00 = BigUint::from_bytes_be(&bytes[ptr..ptr + 32]).to_string();
    ptr += 32;

    let b11 = BigUint::from_bytes_be(&bytes[ptr..ptr + 32]).to_string();
    ptr += 32;
    let b10 = BigUint::from_bytes_be(&bytes[ptr..ptr + 32]).to_string();
    ptr += 32;

    let c0 = BigUint::from_bytes_be(&bytes[ptr..ptr + 32]).to_string();
    ptr += 32;
    let c1 = BigUint::from_bytes_be(&bytes[ptr..ptr + 32]).to_string();
    ptr += 32;

    let proof = CircomProof {
        A: vec![a0, a1, "1".to_string()],
        B: vec![
            vec![b00, b01],
            vec![b10, b11],
            vec!["1".to_string(), "0".to_string()],
        ],
        C: vec![c0, c1, "1".to_string()],
        Protocol: "groth16".to_string(),
        Curve: "bn128".to_string(),
    };

    Ok(proof)
}

fn sp1_snark_proof_to_ffi_type(
    encoded_proof: String,
) -> AnyhowResult<circuit_builder::GnarkGroth16Proof> {
    let mut proof_bytes = [0u8; 256];
    hex::decode_to_slice(encoded_proof, &mut proof_bytes)?;
    let mut offset = 0;

    // a
    let elm_bytes = &proof_bytes[offset..offset + 32];
    let a_x = BigUint::from_bytes_be(&elm_bytes);
    offset += 32;
    let elm_bytes = &proof_bytes[offset..offset + 32];
    let a_y = BigUint::from_bytes_be(&elm_bytes);
    offset += 32;

    // b
    let elm_bytes = &proof_bytes[offset..offset + 32];
    let b_x_a1 = BigUint::from_bytes_be(&elm_bytes);
    offset += 32;
    let elm_bytes = &proof_bytes[offset..offset + 32];
    let b_x_a0 = BigUint::from_bytes_be(&elm_bytes);
    offset += 32;
    let elm_bytes = &proof_bytes[offset..offset + 32];
    let b_y_a1 = BigUint::from_bytes_be(&elm_bytes);
    offset += 32;
    let elm_bytes = &proof_bytes[offset..offset + 32];
    let b_y_a0 = BigUint::from_bytes_be(&elm_bytes);
    offset += 32;

    // b
    let elm_bytes = &proof_bytes[offset..offset + 32];
    let c_x = BigUint::from_bytes_be(&elm_bytes);
    offset += 32;
    let elm_bytes = &proof_bytes[offset..offset + 32];
    let c_y = BigUint::from_bytes_be(&elm_bytes);
    offset += 32;

    let a = G1 {
        X: a_x.to_string(),
        Y: a_y.to_string(),
    };

    let b = G2 {
        X: G1A {
            A0: b_x_a0.to_string(),
            A1: b_x_a1.to_string(),
        },
        Y: G1A {
            A0: b_y_a0.to_string(),
            A1: b_y_a1.to_string(),
        },
    };

    let c = G1 {
        X: c_x.to_string(),
        Y: c_y.to_string(),
    };

    Ok(circuit_builder::GnarkGroth16Proof {
        Ar: a,
        Bs: b,
        Krs: c,
        CommitmentPok: G1 {
            X: "0".to_string(),
            Y: "0".to_string(),
        },
        Commitments: vec![],
    })
}

fn form_bonsai_input_data<H: Hasher + Serialize>(agg_input: AggInputs<H>) -> AnyhowResult<Vec<u8>> {
    let data = to_vec(&agg_input)?;
    let input_data_vec: Vec<u8> = bytemuck::cast_slice(&data).to_vec();
    Ok(input_data_vec)
}

#[cfg(test)]
mod tests {
    use std::fs;

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

    // #[tokio::test]
    // #[ignore]
    // pub async fn test_form_circom_proof_from_snark_receipt() {
    //     let s = SnarkReceipt {
    //         snark: Groth16Seal {
    //             a: [[7, 134, 10, 194, 220, 228, 10, 147, 40, 206, 193, 121, 215, 185, 233, 37, 12, 210, 197, 63, 197, 65, 179, 2, 91, 139, 217, 89, 45, 49, 216, 29].to_vec(), [17, 91, 2, 136, 175, 111, 16, 178, 68, 251, 57, 84, 14, 106, 249, 16, 70, 141, 148, 178, 98, 231, 105, 18, 3, 70, 155, 94, 117, 116, 145, 150].to_vec()].to_vec()

    //             , b: [[[0, 228, 185, 81, 10, 211, 129, 33, 230, 143, 33, 138, 100, 223, 124, 208, 91, 54, 72, 154, 78, 241, 253, 229, 79, 199, 121, 123, 131, 100, 130, 125].to_vec(), [18, 97, 110, 247, 50, 52, 74, 11, 230, 212, 47, 180, 154, 109, 146, 178, 184, 153, 159, 140, 84, 172, 18, 4, 31, 8, 213, 11, 220, 232, 153, 107].to_vec()].to_vec(), [[19, 166, 95, 171, 8, 120, 66, 22, 239, 59, 122, 217, 8, 65, 243, 108, 103, 223, 183, 243, 183, 237, 6, 223, 51, 179, 216, 39, 146, 84, 8, 25].to_vec(), [15, 17, 171, 251, 180, 143, 74, 130, 103, 137, 117, 45, 217, 193, 67, 134, 6, 181, 245, 87, 196, 106, 14, 248, 47, 118, 207, 148, 109, 12, 37, 87].to_vec()].to_vec()].to_vec()

    //             , c:[[17, 175, 26, 146, 194, 183, 124, 180, 212, 2, 35, 15, 247, 235, 250, 183, 162, 197, 198, 39, 76, 1, 121, 3, 107, 105, 141, 31, 134, 76, 72, 248].to_vec(), [20, 11, 194, 53, 122, 101, 218, 241, 130, 184, 225, 18, 220, 224, 69, 91, 181, 95, 47, 123, 173, 16, 241, 244, 61, 136, 245, 165, 236, 160, 177, 194].to_vec()].to_vec()

    //         },

    //         post_state_digest:  [163, 172, 194, 113, 23, 65, 137, 150, 52, 11, 132, 229, 169, 15, 62, 244, 196, 157, 34, 199, 158, 68, 170, 216, 34, 236, 156, 49, 62, 30, 184, 226].to_vec(),
    //         journal: [90, 18, 196, 95, 51, 152, 136, 218, 195, 64, 81, 98, 111, 194, 77, 226, 56, 208, 183, 55, 189, 132, 182, 44, 38, 183, 233, 171, 12, 192, 231, 112].to_vec() ,
    //     };

    //     let proof = form_circom_proof_from_snark_receipt(&s);
    //     println!("circom_proof: {:?}", proof);
    // }

    // #[tokio::test]
    // #[ignore]
    // pub async fn leaf_deserialize() {
    //     let path = "/home/ubuntu/aditya-risc0-test/quantum-node/storage/superproofs/45/leaves.bin";
    //     let bytes = fs::read(path).unwrap();
    //     let leaves: Vec<[u8; 32]> = bincode::deserialize(&bytes).unwrap();

    //     println!("leaves: {:?}", leaves);
    // }

    // #[test]
    // pub fn test_hash() {
    //     let hash = read_bytes_from_file(&get_sp1_agg_vk_hash_bytes_path(
    //         "/home/ubuntu/quantum/quantum-node/storage",
    //         "/sp1_snark_reduction",
    //     )).unwrap();
    //     println!("hash: {:?}", hash);
    // }

    // #[test]
    // pub fn test_write_bytes_to_file() {
    //     let bytes = [123, 2, 228, 20, 207, 223, 208, 234, 150, 60, 208, 248, 48, 50, 180, 200, 28, 142, 9, 12, 156, 157, 25, 39, 234, 135, 78, 232, 224, 192, 137, 216].to_vec();
    //     let path = "/home/ubuntu/quantum/quantum-node/storage/sp1/sp1_empty_root_bytes.bin";
    //     write_bytes_to_file(&bytes, path).unwrap();
    // }
}
