use std::time::Instant;

use anyhow::{anyhow, Ok, Result as AnyhowResult};
use quantum_circuits_interface::amqp::interactor::QuantumV2CircuitInteractor;
use quantum_db::repository::{
    reduction_circuit_repository::get_reduction_circuit_for_user_circuit,
    superproof_repository::{
        get_last_verified_superproof, get_superproof_by_id, update_superproof_agg_time,
        update_superproof_pis_path, update_superproof_proof_path,
        update_superproof_total_proving_time,
    },
    user_circuit_data_repository::get_user_circuit_data_by_circuit_hash,
};
use quantum_types::{
    enums::proving_schemes::ProvingSchemes,
    traits::{circuit_interactor::CircuitInteractorAMQP, pis::Pis, proof::Proof, vkey::Vkey},
    types::{
        config::{AMQPConfigData, ConfigData},
        db::proof::Proof as DBProof,
        gnark_groth16::{GnarkGroth16Pis, GnarkGroth16Proof, GnarkGroth16Vkey, GnarkVerifier},
        halo2_plonk::{Halo2PlonkPis, Halo2PlonkVkey},
        snarkjs_groth16::{SnarkJSGroth16Pis, SnarkJSGroth16Vkey},
    },
};
use quantum_utils::{
    error_line,
    paths::{get_imt_vkey_path, get_superproof_pis_path, get_superproof_proof_path},
};
use sqlx::{MySql, Pool};
use tracing::info;
use crate::connection::get_pool;
use crate::utils::get_last_superproof_leaves;

pub async fn handle_aggregation(
    proofs: Vec<DBProof>,
    superproof_id: u64,
    config: &ConfigData,
) -> AnyhowResult<()> {
    info!("superproof_id {:?}", superproof_id);

    let amqp_config = AMQPConfigData::get_config();

    // prepare reduction_circuit_data_vec
    let mut reduction_circuit_data_vec = Vec::<GnarkVerifier>::new();
    let mut protocol_vkey_hashes: Vec<Vec<u8>> = vec![];
    let mut protocol_pis_hashes: Vec<Vec<u8>> = vec![];
    for proof in &proofs {
        let reduced_proof_path = proof.reduction_proof_path.clone().unwrap();
        let reduced_proof = GnarkGroth16Proof::read_proof(&reduced_proof_path)?;

        let reduced_pis_path = proof.reduction_proof_pis_path.clone().unwrap();
        let reduced_pis = GnarkGroth16Pis::read_pis(&reduced_pis_path)?;

        let reduced_circuit_vkey_path = get_reduction_circuit_for_user_circuit(get_pool().await, &proof.user_circuit_hash).await?.vk_path;
        let reduced_vkey = GnarkGroth16Vkey::read_vk(&reduced_circuit_vkey_path)?;

        let gnark_verifier = GnarkVerifier {
            Proof: reduced_proof,
            VK: reduced_vkey,
            PubInputs: reduced_pis.0,
        };
        reduction_circuit_data_vec.push(gnark_verifier);

        let user_circuit_data = get_user_circuit_data_by_circuit_hash(get_pool().await, &proof.user_circuit_hash).await?;
        let protocol_circuit_vkey_path = get_user_circuit_data_by_circuit_hash(get_pool().await, &proof.user_circuit_hash).await?.vk_path;
        let protocol_pis_path = proof.pis_path.clone();

        match user_circuit_data.proving_scheme {
            ProvingSchemes::GnarkGroth16 => {
                let protocol_vkey = GnarkGroth16Vkey::read_vk(&protocol_circuit_vkey_path)?;
                protocol_vkey_hashes.push(protocol_vkey.extended_keccak_hash(user_circuit_data.n_commitments)?.to_vec());

                let protocol_pis = GnarkGroth16Pis::read_pis(&protocol_pis_path)?;
                protocol_pis_hashes.push(protocol_pis.extended_keccak_hash()?.to_vec());
            }
            ProvingSchemes::Groth16 => {
                let protocol_vkey = SnarkJSGroth16Vkey::read_vk(&protocol_circuit_vkey_path)?;
                protocol_vkey_hashes.push(protocol_vkey.extended_keccak_hash(user_circuit_data.n_commitments)?.to_vec());

                let protocol_pis = SnarkJSGroth16Pis::read_pis(&protocol_pis_path)?;
                protocol_pis_hashes.push(protocol_pis.extended_keccak_hash()?.to_vec());
            }
            ProvingSchemes::Halo2Plonk => {
                let protocol_vkey = Halo2PlonkVkey::read_vk(&protocol_circuit_vkey_path)?;
                protocol_vkey_hashes.push(protocol_vkey.extended_keccak_hash(user_circuit_data.n_commitments)?.to_vec());

                let protocol_pis = Halo2PlonkPis::read_pis(&protocol_pis_path)?;
                protocol_pis_hashes.push(protocol_pis.extended_keccak_hash()?.to_vec());
            }
            _ => todo!(),
        }
    }

    let last_leaves = get_last_superproof_leaves(config).await?;

    // prepare imt_reduction_circuit_data
    let superproof = get_superproof_by_id(get_pool().await, superproof_id).await?;
    let imt_proof_path = superproof.imt_proof_path.ok_or(anyhow!("missing imt proof path"))?;
    let imt_pis_path = superproof.imt_pis_path.ok_or(anyhow!("missing imt pis path"))?;
    let imt_vkey_path = get_imt_vkey_path(&config.aggregated_circuit_data);
    let imt_proof = GnarkGroth16Proof::read_proof(&imt_proof_path)?;
    let imt_pis = GnarkGroth16Pis::read_pis(&imt_pis_path)?;
    let imt_vkey = GnarkGroth16Vkey::read_vk(&imt_vkey_path)?;
    let imt_reduction_circuit_data = GnarkVerifier {
        Proof: imt_proof,
        PubInputs: imt_pis.0,
        VK: imt_vkey,
    };

    let aggregation_start = Instant::now();
    let aggregation_result = QuantumV2CircuitInteractor::generate_aggregated_proof(
        &amqp_config,
        config.batch_size,
        last_leaves.leaves,
        reduction_circuit_data_vec,
        imt_reduction_circuit_data,
        protocol_vkey_hashes,
        protocol_pis_hashes,
        superproof_id,
    )?;

    let aggregation_time = aggregation_start.elapsed();
    info!(
        "aggregation_result {:?} in {:?}",
        aggregation_result.msg, aggregation_time
    );
    if !aggregation_result.success {
        return Err(anyhow::Error::msg(error_line!(aggregation_result.msg)));
    }

    // Dump superproof proof and add to the DB
    let superproof_proof = aggregation_result.aggregated_proof;
    let superproof_proof_path = get_superproof_proof_path(
        &config.storage_folder_path,
        &config.supperproof_path,
        superproof_id,
    );
    superproof_proof.dump_proof(&superproof_proof_path)?;
    update_superproof_proof_path(get_pool().await, &superproof_proof_path, superproof_id).await?;

    // Dump superproof pis and add to the DB
    let superproof_pis = GnarkGroth16Pis(aggregation_result.pub_inputs);
    let superproof_pis_path = get_superproof_pis_path(
        &config.storage_folder_path,
        &config.supperproof_path,
        superproof_id,
    );
    superproof_pis.dump_pis(&superproof_pis_path)?;
    update_superproof_pis_path(get_pool().await, &superproof_pis_path, superproof_id).await?;

    // Add agg_time to the db
    update_superproof_agg_time(get_pool().await, aggregation_time.as_secs(), superproof_id).await?;

    let proof_with_max_reduction_time = proofs.iter().max_by_key(|proof| proof.reduction_time);
    let total_proving_time = proof_with_max_reduction_time
        .unwrap()
        .reduction_time
        .unwrap()
        + aggregation_time.as_secs();
    update_superproof_total_proving_time(get_pool().await, total_proving_time, superproof_id).await?;

    Ok(())
}
