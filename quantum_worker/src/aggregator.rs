use std::time::Instant;

use anyhow::{Ok, Result as AnyhowResult};
use quantum_circuits_ffi::interactor::QuantumV2CircuitInteractor;
use quantum_db::repository::{
    proof_repository::add_aggregation_hardware_cost_to_proofs, reduction_circuit_repository::get_reduction_circuit_for_user_circuit, superproof_repository::{update_superproof_agg_time, update_superproof_proof_path, update_superproof_total_proving_time}, user_circuit_data_repository::get_user_circuit_data_by_circuit_hash
};
use quantum_types::{
    enums::proving_schemes::ProvingSchemes,
    traits::{circuit_interactor::CircuitInteractor, pis::Pis, proof::Proof, vkey::Vkey},
    types::{
        config::ConfigData,
        db::proof::Proof as DBProof,
        gnark_groth16::{GnarkGroth16Pis, GnarkGroth16Proof, GnarkGroth16Vkey},
        halo2_plonk::{Halo2PlonkPis, Halo2PlonkVkey},
        snarkjs_groth16::{SnarkJSGroth16Pis, SnarkJSGroth16Vkey},
    },
};
use quantum_utils::{
    error_line,
    file::read_bytes_from_file,
    paths::{
        get_aggregation_circuit_constraint_system_path, get_aggregation_circuit_proving_key_path,
        get_aggregation_circuit_vkey_path, get_superproof_proof_path,
    },
};
use sqlx::{MySql, Pool};
use tracing::info;

pub async fn handle_aggregation(
    pool: &Pool<MySql>,
    proofs: Vec<DBProof>,
    superproof_id: u64,
    config: &ConfigData,
) -> AnyhowResult<()> {
    let mut reduced_proofs = Vec::<GnarkGroth16Proof>::new();
    let mut reduced_pis_vec = Vec::<GnarkGroth16Pis>::new();
    let mut reduced_circuit_vkeys = Vec::<GnarkGroth16Vkey>::new();
    let mut protocol_vkey_hashes: Vec<Vec<u8>> = vec![];
    let mut protocol_pis_hashes: Vec<Vec<u8>> = vec![];

    for proof in &proofs {
        let reduced_proof_path = proof.reduction_proof_path.clone().unwrap();
        let reduced_proof = GnarkGroth16Proof::read_proof(&reduced_proof_path)?;
        reduced_proofs.push(reduced_proof);

        let reduced_pis_path = proof.reduction_proof_pis_path.clone().unwrap();
        let reduced_pis = GnarkGroth16Pis::read_pis(&reduced_pis_path)?;
        reduced_pis_vec.push(reduced_pis);

        let reduced_circuit_vkey_path =
            get_reduction_circuit_for_user_circuit(pool, &proof.user_circuit_hash)
                .await?
                .vk_path;
        let reduced_vkey = GnarkGroth16Vkey::read_vk(&reduced_circuit_vkey_path)?;
        reduced_circuit_vkeys.push(reduced_vkey);

        let user_circuit_data =
            get_user_circuit_data_by_circuit_hash(pool, &proof.user_circuit_hash).await?;
        let protocol_circuit_vkey_path =
            get_user_circuit_data_by_circuit_hash(pool, &proof.user_circuit_hash)
                .await?
                .vk_path;
        let protocol_pis_path = proof.pis_path.clone();

        match user_circuit_data.proving_scheme {
            ProvingSchemes::GnarkGroth16 => {
                let protocol_vkey = GnarkGroth16Vkey::read_vk(&protocol_circuit_vkey_path)?;
                protocol_vkey_hashes.push(protocol_vkey.keccak_hash()?.to_vec());

                let protocol_pis = GnarkGroth16Pis::read_pis(&protocol_pis_path)?;
                protocol_pis_hashes.push(protocol_pis.keccak_hash()?.to_vec());
            }
            ProvingSchemes::Groth16 => {
                let protocol_vkey = SnarkJSGroth16Vkey::read_vk(&protocol_circuit_vkey_path)?;
                protocol_vkey_hashes.push(protocol_vkey.keccak_hash()?.to_vec());

                let protocol_pis = SnarkJSGroth16Pis::read_pis(&protocol_pis_path)?;
                protocol_pis_hashes.push(protocol_pis.keccak_hash()?.to_vec());
            }
            ProvingSchemes::Halo2Plonk => {
                let protocol_vkey = Halo2PlonkVkey::read_vk(&protocol_circuit_vkey_path)?;
                protocol_vkey_hashes.push(protocol_vkey.keccak_hash()?.to_vec());

                let protocol_pis = Halo2PlonkPis::read_pis(&protocol_pis_path)?;
                protocol_pis_hashes.push(protocol_pis.keccak_hash()?.to_vec());
            }
            _ => todo!(),
        }
    }
    println!("superproof_id {:?}", superproof_id);

    // Read aggregator_circuit_pkey and aggregator_circuit_vkey from file
    let aggregator_cs_path =
        get_aggregation_circuit_constraint_system_path(&config.aggregated_circuit_data);
    let aggregator_pkey_path =
        get_aggregation_circuit_proving_key_path(&config.aggregated_circuit_data);
    let aggregator_vkey_path = get_aggregation_circuit_vkey_path(&config.aggregated_circuit_data);
    let aggregator_circuit_cs = read_bytes_from_file(&aggregator_cs_path)?;
    let aggregator_circuit_pkey = read_bytes_from_file(&aggregator_pkey_path)?;
    let aggregator_circuit_vkey = GnarkGroth16Vkey::read_vk(&aggregator_vkey_path)?;

    let aggregation_start = Instant::now();

    let aggregation_result = QuantumV2CircuitInteractor::generate_aggregated_proof(
        reduced_proofs,
        reduced_pis_vec,
        reduced_circuit_vkeys,
        protocol_vkey_hashes,
        protocol_pis_hashes,
        aggregator_circuit_cs,
        aggregator_circuit_pkey,
        aggregator_circuit_vkey,
    );

    let aggregation_time = aggregation_start.elapsed();
    info!(
        "aggregation_result {:?} in {:?}",
        aggregation_result.msg, aggregation_time
    );

    if !aggregation_result.success {
        return Err(anyhow::Error::msg(error_line!(aggregation_result.msg)));
    }

    // Dump superproof_proof and add to the DB
    let superproof_proof = aggregation_result.reduced_proof;
    let superproof_proof_path = get_superproof_proof_path(
        &config.storage_folder_path,
        &config.supperproof_path,
        superproof_id,
    );
    superproof_proof.dump_proof(&superproof_proof_path)?;
    update_superproof_proof_path(pool, &superproof_proof_path, superproof_id).await?;

    // Add agg_time to the db
    update_superproof_agg_time(pool, aggregation_time.as_secs(), superproof_id).await?;
    
    let proof_with_max_reduction_time = proofs.iter().min_by_key(|proof| proof.reduction_time);
    let total_proving_time = proof_with_max_reduction_time.unwrap().reduction_time.unwrap() + aggregation_time.as_secs();
    update_superproof_total_proving_time(pool, total_proving_time, superproof_id);
    
    let agg_hardware_cost_pr_proof: f32 = (aggregation_time.as_secs() * config.proof_aggregation_pr_sec_machine_cost) / proofs.len();
    add_aggregation_hardware_cost_to_proofs(pool, agg_hardware_cost_pr_proof, superproof_id).await?;
    Ok(())
}
