use quantum_db::repository::{proof_repository::{get_proof_by_proof_hash, insert_proof}, task_repository::create_proof_task, user_circuit_data_repository::get_user_circuit_data_by_circuit_hash};
use quantum_types::{enums::{circuit_reduction_status::CircuitReductionStatus, proof_status::ProofStatus, task_status::TaskStatus, task_type::TaskType}, traits::{pis::Pis, proof::Proof}, types::config::ConfigData};
use quantum_utils::keccak::get_keccal_hash_from_bytes;
use rocket::State;
use anyhow::{anyhow, Result as AnyhowResult};
use crate::{connection::get_pool, error::error::CustomError, types::submit_proof::{SubmitProofRequest, SubmitProofResponse}};

pub async fn submit_proof_exec<T: Proof, F: Pis>(data: SubmitProofRequest, config_data: &State<ConfigData>) -> AnyhowResult<SubmitProofResponse>{
    check_if_circuit_reduction_completed(&data.circuit_hash).await?;
    
    let proof_encoded: Vec<u8> = data.proof.clone();
    let proof: T = T::deserialize(&mut proof_encoded.as_slice())?;
    let proof_id = get_keccal_hash_from_bytes(proof_encoded.clone());

    check_if_proof_already_exist(&proof_id).await?;
    println!("{:?}", proof_id);
    let pis_encoded: Vec<u8> = data.pis.clone();
    let pis: F = F::deserialize(&mut pis_encoded.as_slice())?;
    
    let proof_full_path = proof.dump_proof(&data.circuit_hash, config_data, &proof_id)?;
    let pis_full_path = pis.dump_pis(&data.circuit_hash, config_data, &proof_id)?;

    insert_proof(get_pool().await, &proof_id, &pis_full_path, &proof_full_path, ProofStatus::Registered, &data.circuit_hash).await?;
    create_proof_task(get_pool().await, &data.circuit_hash, TaskType::ProofGeneration, TaskStatus::NotPicked, &proof_id).await?;

    Ok(SubmitProofResponse {
        proof_id
    })
}

pub async fn check_if_circuit_reduction_completed(circuit_hash: &str) -> AnyhowResult<()>{
    let circuit_data = get_user_circuit_data_by_circuit_hash(get_pool().await, circuit_hash).await;
    let circuit_data = match circuit_data {
        Ok(cd) => Ok(cd),
        Err(_) => {
            println!("circuit has not been registered");
            Err(anyhow!(CustomError::BadRequest("circuit hash not found".to_string())))
        }
    };

    let cd = circuit_data?;
    if cd.circuit_reduction_status != CircuitReductionStatus::Completed {
        println!("circuit reduction not completed");
        return Err(anyhow!(CustomError::BadRequest("circuit reduction not completed".to_string())));
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
        println!("proof already exist");
        return Err(anyhow!(CustomError::BadRequest("proof already exist".to_string())));
    }
    Ok(())
}