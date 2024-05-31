use quantum_db::repository::{proof_repository::{get_proof_by_proof_hash, insert_proof}, superproof_repository::get_superproof_by_id, task_repository::create_proof_task, user_circuit_data_repository::get_user_circuit_data_by_circuit_hash};
use quantum_types::{enums::{circuit_reduction_status::CircuitReductionStatus, proof_status::ProofStatus, task_status::TaskStatus, task_type::TaskType}, traits::{pis::Pis, proof::Proof}, types::config::ConfigData};
use rocket::State;
use anyhow::{anyhow, Result as AnyhowResult};
use tracing::info;
use crate::{connection::get_pool, error::error::CustomError, types::{proof_data::ProofDataResponse, submit_proof::{SubmitProofRequest, SubmitProofResponse}}};

pub async fn submit_proof_exec<T: Proof, F: Pis>(data: SubmitProofRequest, config_data: &State<ConfigData>) -> AnyhowResult<SubmitProofResponse>{
    validate_circuit_data_in_submit_proof_request(&data).await?;
    
    let proof: T = T::deserialize(&mut data.proof.as_slice())?;
    let proof_id = String::from_utf8(proof.keccak_hash()?.to_vec())?;

    check_if_proof_already_exist(&proof_id).await?;

    let pis: F = F::deserialize(&mut data.pis.as_slice())?;
    
    let proof_full_path = proof.dump_proof(&data.circuit_hash, config_data, &proof_id)?;
    let pis_full_path = pis.dump_pis(&data.circuit_hash, config_data, &proof_id)?;

    insert_proof(get_pool().await, &proof_id, &pis_full_path, &proof_full_path, ProofStatus::Registered, &data.circuit_hash).await?;
    create_proof_task(get_pool().await, &data.circuit_hash, TaskType::ProofGeneration, TaskStatus::NotPicked, &proof_id).await?;

    Ok(SubmitProofResponse {
        proof_id
    })
}

pub async fn get_proof_data_exec(proof_id: String, config_data: &ConfigData) -> AnyhowResult<ProofDataResponse> {
    let mut response = ProofDataResponse {
        status: ProofStatus::NotFound.to_string(),
        superproof_id: -1,
        transaction_hash: None,
        verification_contract: config_data.verification_contract_address.clone()
    };
    let proof = get_proof_by_proof_hash(get_pool().await, &proof_id).await;
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
                let error_msg = format!("superproof not found in db: {}", e.to_string());
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
        Err(_) => {
            info!("circuit has not been registered");
            Err(anyhow!(CustomError::BadRequest("circuit hash not found".to_string())))
        }
    };

    let circuit_data = circuit_data?;
    if circuit_data.circuit_reduction_status != CircuitReductionStatus::Completed {
        info!("circuit reduction not completed");
        return Err(anyhow!(CustomError::BadRequest("circuit reduction not completed".to_string())));
    }
    if data.proof_type != circuit_data.proving_scheme {
        info!("prove type is not correct");
        return Err(anyhow!(CustomError::BadRequest("prove type is not correct".to_string())));
    }
    
    Ok(())
}

pub async fn check_if_proof_already_exist(proof_id: &str) -> AnyhowResult<()> {
    let proof = get_proof_by_proof_hash(get_pool().await, proof_id).await;
    let is_proof_already_registered = match proof {
        Ok(_) => true,
        Err(_) => false
    };
    if is_proof_already_registered {
        info!("proof already exist");
        return Err(anyhow!(CustomError::BadRequest("proof already exist".to_string())));
    }
    Ok(())
}