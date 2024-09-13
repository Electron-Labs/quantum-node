use agg_core::inputs::{compute_combined_vkey_hash, compute_leaf_value};
use quantum_db::repository::{bonsai_image::get_bonsai_image_by_image_id, proof_repository::{get_latest_proof_by_circuit_hash, insert_proof}, superproof_repository::{get_last_verified_superproof, get_superproof_by_id}, task_repository::create_proof_task, user_circuit_data_repository::get_user_circuit_data_by_circuit_hash};
use quantum_types::{enums::{circuit_reduction_status::CircuitReductionStatus, proof_status::ProofStatus, task_status::TaskStatus, task_type::TaskType}, traits::{pis::Pis, proof::Proof, vkey::Vkey}, types::{config::ConfigData, hash::KeccakHashOut, imt::ImtTree}};
use quantum_types::types::db::proof::Proof as DbProof;
use quantum_utils::{keccak::encode_keccak_hash, paths::{get_user_pis_path, get_user_proof_path},error_line};
use rocket::State;
use anyhow::{anyhow, Result as AnyhowResult};
use tracing::{error, info};
use utils::hash::{Hasher, KeccakHasher};
use crate::{connection::get_pool, error::error::CustomError, types::{proof_data::ProofDataResponse, protocol_proof::ProtocolProofResponse, submit_proof::{SubmitProofRequest, SubmitProofResponse}}};
use quantum_db::repository::proof_repository::get_proof_by_proof_hash;
use quantum_db::repository::protocol::get_protocol_by_protocol_name;

pub async fn submit_proof_exec<T: Proof, F: Pis, V: Vkey>(data: SubmitProofRequest, config_data: &State<ConfigData>) -> AnyhowResult<SubmitProofResponse>{
    validate_circuit_data_in_submit_proof_request(&data).await?;

    let proof: T = T::deserialize_proof(&mut data.proof.as_slice())?;

    let pis: F = F::deserialize_pis(&mut data.pis.as_slice())?;

    let user_circuit_data = get_user_circuit_data_by_circuit_hash(get_pool().await, &data.circuit_hash).await?;
    let user_vk = V::read_vk(&user_circuit_data.vk_path)?;

    let proof_id_hash =  KeccakHasher::combine_hash(&user_vk.keccak_hash()?,&pis.keccak_hash()?);

    let proof_hash = encode_keccak_hash(&proof_id_hash)?;

    check_if_proof_already_exist(&proof_hash, &data.circuit_hash).await?;

    // Dump proof and pis binaries
    let proof_full_path = get_user_proof_path(&config_data.storage_folder_path, &config_data.proof_path, &data.circuit_hash, &proof_hash);
    let pis_full_path = get_user_pis_path(&config_data.storage_folder_path, &config_data.public_inputs_path, &data.circuit_hash, &proof_hash);
    proof.dump_proof(&proof_full_path)?;
    pis.dump_pis(&pis_full_path)?;

    let public_inputs_json_string =  serde_json::to_string(&pis.get_data()?).unwrap();
    let proof_id = insert_proof(get_pool().await, &proof_hash, &pis_full_path, &proof_full_path, ProofStatus::Registered, &data.circuit_hash, &public_inputs_json_string).await?;
    create_proof_task(get_pool().await, &data.circuit_hash, TaskType::ProofGeneration, TaskStatus::NotPicked, &proof_hash, proof_id).await?;

    Ok(SubmitProofResponse {
        proof_id: proof_hash
    })
}

pub async fn get_proof_data_exec(proof_hash: String, config_data: &ConfigData) -> AnyhowResult<ProofDataResponse> {
    let mut response = ProofDataResponse {
        status: ProofStatus::NotFound.to_string(),
        superproof_id: -1,
        transaction_hash: None,
        verification_contract: config_data.verification_contract_address.clone()
    };
    let proof = get_proof_by_proof_hash(get_pool().await, &proof_hash).await;
    if proof.is_err() {
        return Ok(response);
    }

    let proof = proof?;
    response.status = proof.proof_status.to_string();
    if proof.superproof_id.is_some() {
        let superproof_id = proof.superproof_id.unwrap_or(0);
        let superproof = get_superproof_by_id(get_pool().await, superproof_id).await;
        let superproof = match superproof {
            Ok(sp) => Ok(sp),
            Err(e) => {
                info!("err in superproof fetch");
                let error_msg = format!("superproof not found in db: {}", e.root_cause().to_string());
                Err(anyhow!(CustomError::Internal(error_msg)))
            }
        };
        let superproof = superproof?;

        response.superproof_id = superproof_id as i64;
        response.transaction_hash = superproof.transaction_hash;
    }

    return Ok(response);
}

async fn validate_circuit_data_in_submit_proof_request(data: &SubmitProofRequest) -> AnyhowResult<()>{
    let circuit_data = get_user_circuit_data_by_circuit_hash(get_pool().await, &data.circuit_hash).await;
    let circuit_data = match circuit_data {
        Ok(cd) => Ok(cd),
        Err(e) => {
            info!("circuit has not been registered");
            Err(anyhow!(CustomError::BadRequest(error_line!(format!("circuit hash not found. {}", e.root_cause().to_string())))))
        }
    };

    let circuit_data = circuit_data?;
    if circuit_data.circuit_reduction_status != CircuitReductionStatus::Completed {
        info!("circuit reduction not completed");
        return Err(anyhow!(CustomError::BadRequest(error_line!("circuit reduction not completed".to_string()))));
    }

    if data.proof_type != circuit_data.proving_scheme {
        info!("prove type is not correct");
        return Err(anyhow!(CustomError::BadRequest(error_line!("prove type is not correct".to_string()))));
    }

    validate_on_ongoing_proof_with_same_circuit_hash(&data.circuit_hash).await?;
    Ok(())
}

pub async fn validate_on_ongoing_proof_with_same_circuit_hash(circuit_hash: &str) -> AnyhowResult<()> {
    let proof = match get_latest_proof_by_circuit_hash(get_pool().await, circuit_hash).await {
        Ok(p) => Ok(p),
        Err(e) => {
            error!("error in finding the last proof for circuit hash {:?}: {:?}", circuit_hash, error_line!(e));
            Err(e)
        },
    };

    if proof.is_err() {
        return Ok(())
    }

    let proof = proof?;
    if proof.proof_status == ProofStatus::Registered || proof.proof_status == ProofStatus::Reducing || proof.proof_status == ProofStatus::Reduced {
        return Err(anyhow!(CustomError::BadRequest(error_line!(format!("last proof for circuit id {:?} hasn't been verified, rejecting proof submission request", circuit_hash)))))
    }
    Ok(())
}

// TODO: need to change
pub async fn check_if_proof_already_exist(proof_hash: &str, circuit_hash: &str) -> AnyhowResult<()> {
    let proof = get_proof_by_proof_hash(get_pool().await, proof_hash).await;
    if proof.is_ok() {
        let user_circuit = get_user_circuit_data_by_circuit_hash(get_pool().await, proof?.user_circuit_hash.as_str()).await?;
        let protocol_name = user_circuit.protocol_name;

        let protocol = get_protocol_by_protocol_name(get_pool().await, &protocol_name).await?;
        if protocol.is_proof_repeat_allowed == 0 {
            info!("proof already exist");
            return Err(anyhow!(CustomError::BadRequest(error_line!("proof already exist".to_string()))));
        }
        // else {
        //     let user_circuit_hash = get_user_
        //     let proof = get_proof_by_proof_hash_within_limits(get_pool().await, proof_hash, circuit_hash, 51).await;
        //     if proof.is_ok() {
        //         info!("repeat proof present in the latest 51 proofs");
        //         return Err(anyhow!(CustomError::BadRequest(error_line!("repeat proof present in the latest 51 proofs".to_string()))));
        //     }
        // }
    }
    Ok(())
}

pub async fn get_protocol_proof_exec<T: Pis, V: Vkey>(proof: &DbProof) -> AnyhowResult<ProtocolProofResponse, CustomError> {
    let user_circuit_hash = proof.user_circuit_hash.clone();
    let user_circuit_data = get_user_circuit_data_by_circuit_hash(get_pool().await, &user_circuit_hash).await?;
    let bonsai_image = get_bonsai_image_by_image_id(get_pool().await, &user_circuit_data.bonsai_image_id).await?;

    let user_circuit_vk = V::read_vk(&user_circuit_data.vk_path)?;

    let combined_vk_hash = compute_combined_vkey_hash::<KeccakHasher>(&user_circuit_vk.keccak_hash()?, &bonsai_image.circuit_verifying_id)?;

    let pis: T = T::read_pis(&proof.pis_path)?;

    let protocol_pis_hash = pis.keccak_hash()?;

    let latest_verififed_superproof = match get_last_verified_superproof(get_pool().await).await? {
        Some(superproof) => Ok(superproof),
        None => Err(anyhow!(CustomError::Internal(error_line!("last super proof verified not found".to_string())))),
    }?;
    let leaf_path = latest_verififed_superproof.superproof_leaves_path.unwrap();
    let imt_tree = ImtTree::read_tree(&leaf_path)?;

    let leaf_value = compute_leaf_value::<KeccakHasher>(&combined_vk_hash, &protocol_pis_hash);

    let mt_proof = imt_tree
        .get_imt_proof(KeccakHashOut(
            leaf_value[..32]
                .try_into()
                .map_err(|e: std::array::TryFromSliceError| anyhow!(e))?,
        ))
        .map_err(|err| CustomError::NotFound(error_line!(format!("proof not found in the tree::{}", err.to_string()))))?;
    let mt_proof_encoded = mt_proof.0.iter().map(|x| encode_keccak_hash(x.as_slice()[0..32].try_into().unwrap()).unwrap()).collect::<Vec<String>>();

    let mut merkle_proof_position: u64 = 0;
    for i in 0..mt_proof.1.len() {
        merkle_proof_position += (mt_proof.1[i] as u64) * 2u64.pow(i as u32);
    }

    let leaf_next_index_str = format!("0x{}", hex::encode(&mt_proof.2.next_idx));

    Ok(ProtocolProofResponse {
        merkle_proof_position,
        merkle_proof: mt_proof_encoded,
        leaf_next_value: encode_keccak_hash(&mt_proof.2.next_value.0)?,
        leaf_next_index: leaf_next_index_str,
    })
}