// use aggregator::handle_aggregation;
use anyhow::{anyhow, Result as AnyhowResult};
use chrono::Utc;
use dotenv::dotenv;
// use imt::handle_imt;
// use bonsai_sdk::

use quantum_db::{
    error::error::CustomError,
    repository::{
        proof_repository::{
            get_n_reduced_proofs, update_proof_status, update_superproof_id_in_proof,
        },
        superproof_repository::{get_last_verified_superproof, insert_new_superproof, update_superproof_status},
        task_repository::{
            get_aggregation_waiting_tasks_num, get_unpicked_task, update_task_status,
        },
        user_circuit_data_repository::update_user_circuit_data_reduction_status,
    },
};
use quantum_types::{
    enums::{
        circuit_reduction_status::CircuitReductionStatus, proof_status::ProofStatus,
        superproof_status::SuperproofStatus, task_status::TaskStatus, task_type::TaskType,
    },
    types::{
        config::ConfigData,
        db::{proof::Proof, superproof::Superproof, task::Task},
    },
};
use quantum_utils::{error_line, logger::initialize_logger};
use sqlx::{MySql, Pool};
use std::{thread::sleep, time::Duration};
// use risc0_zkvm::{ExecutorEnv, Receipt};
use tracing::{error, info};
use crate::{aggregator::handle_proof_aggregation_and_updation, connection::get_pool};
// use crate::imt::handle_imt_proof_generation_and_updation;
use crate::{proof_generator, registration};
// use crate::aggregator::handle_proof_aggregation_and_updation;

pub async fn handle_aggregate_proof_task(
    proofs: Vec<Proof>,
    config: &ConfigData,
    superproof_id: u64,
) -> AnyhowResult<()>
{
    let mut proof_ids: Vec<u64> = vec![];
    for proof in &proofs {
        let proof_id = match proof.id {
            Some(id) => Ok(id),
            None => Err(anyhow!(error_line!("not able to find proofId"))),
        };
        let proof_id = proof_id?;
        proof_ids.push(proof_id);
    }

    let aggregation_request = handle_proof_aggregation_and_updation(proofs.clone(), superproof_id, config).await;

    match aggregation_request {
        Ok(_) => {
            // Update Proof Status to aggregated for all the proofs
            for proof_id in proof_ids {
                update_proof_status(get_pool().await, proof_id, ProofStatus::Aggregated).await?;
            }
            // Superproof status to PROVING_DONE
            info!("changing the superproof status to proving done");
            update_superproof_status(get_pool().await, SuperproofStatus::ProvingDone, superproof_id).await?;
        }
        Err(e) => {
            error!("aggregation_request error {:?}", e);

            // Change proof_generation status to FAILED
            for proof_id in proof_ids {
                update_proof_status(get_pool().await, proof_id, ProofStatus::AggregationFailed).await?;
            }

            error!("changing the superproof status to failed");
            update_superproof_status(get_pool().await, SuperproofStatus::Failed, superproof_id).await?;
            return Err(e);
        }
    }
    Ok(())
}

pub async fn handle_proof_generation_task(
    proof_generation_task: Task,
    config: &ConfigData,
) -> AnyhowResult<()> {
    let proof_id = proof_generation_task.clone().proof_id.clone().unwrap();
    // Change Task status to InProgress
    update_task_status(get_pool().await, proof_generation_task.clone().id.unwrap(), TaskStatus::InProgress).await?;
    info!("Updated Task Status to InProgress");

    // Update Proof Status to Reducing
    update_proof_status(get_pool().await, proof_id, ProofStatus::Reducing).await?;
    info!("Update Proof Status to Reducing");

    let proof_id = match proof_generation_task.proof_id {
        None => Err(anyhow!(error_line!("Proof generation task does not contain the proof id"))),
        Some(p) => Ok(p),
    }?;

    let proof_hash = match proof_generation_task.clone().proof_hash {
        None => Err(anyhow!(error_line!("Proof generation task does not contain the proof hash"))),
        Some(p) => Ok(p),
    }?;

    let request = proof_generator::handle_proof_generation_and_updation(proof_id, &proof_hash, &proof_generation_task.user_circuit_hash, config).await;

    match request {
        Ok(_) => {
            // Change proof_generation status to REDUCED
            update_proof_status(get_pool().await, proof_id, ProofStatus::Reduced).await?;
            info!("Changed proof status to REDUCED");

            // Update task status to completed
            update_task_status(get_pool().await, proof_generation_task.clone().id.unwrap(), TaskStatus::Completed).await?;
            info!("Changed task status to Completed");

            info!("Proof Reduced Successfully");
        }
        Err(e) => {
            // Change proof_generation status to FAILED
            update_proof_status(get_pool().await, proof_id, ProofStatus::ReductionFailed).await?;
            info!("Changed Proof Status to FAILED");

            // Update task status to failed
            update_task_status(get_pool().await, proof_generation_task.clone().id.unwrap(), TaskStatus::Failed).await?;
            info!("Changed Task Status to FAILED");

            error!("Proof Reduction Failed: {:?}", e.root_cause().to_string());
        }
    }

    Ok(())
}

pub async fn aggregate_and_generate_new_superproof(aggregation_awaiting_proofs: Vec<Proof>, config_data: &ConfigData) -> AnyhowResult<()>
{
    // INSERT NEW SUPERPROOF RECORD
    let mut proof_ids: Vec<u64> = vec![];
    for proof in &aggregation_awaiting_proofs {
        let proof_id = match proof.id {
            Some(id) => Ok(id),
            None => Err(anyhow!(error_line!("not able to find proofId"))),
        };
        let proof_id = proof_id?;
        proof_ids.push(proof_id);
    }
    let proof_json_string = serde_json::to_string(&proof_ids)?;
    let superproof_id = insert_new_superproof(get_pool().await, &proof_json_string, SuperproofStatus::InProgress).await?;
    info!("added new superproof record => superproof_id={}",superproof_id);


    for proof_id in proof_ids.clone() {
        update_proof_status(get_pool().await, proof_id, ProofStatus::Aggregating).await?;
    }

    for proof_id in proof_ids {
        update_superproof_id_in_proof(get_pool().await, proof_id, superproof_id).await?;
    }

    // handle_imt_proof_generation_and_updation(aggregation_awaiting_proofs.clone(), superproof_id, config_data, ).await?;
    handle_aggregate_proof_task(aggregation_awaiting_proofs, config_data, superproof_id).await?;

    Ok(())
}

pub async fn worker(sleep_duration: Duration, config_data: &ConfigData) -> AnyhowResult<()> {
    loop {
        // TODO: will have variable batch size for aggregation.
        // println!("Running worker loop");
        let last_verified_superproof = get_last_verified_superproof(get_pool().await).await?;
        let aggregation_awaiting_proofs = get_n_reduced_proofs(get_pool().await, config_data.batch_size).await?;
        println!(
            "Aggregation awaiting proofs {:?}",
            aggregation_awaiting_proofs.len()
        );
        if (aggregation_awaiting_proofs.len() !=0 && 
        last_verified_superproof.is_some() && 
        last_verified_superproof.unwrap().onchain_submission_time.unwrap() + Duration::from_secs(30*60) >= Utc::now().naive_utc()){
            info!("Picked up Proofs aggregation");
            aggregate_and_generate_new_superproof(aggregation_awaiting_proofs.clone(), config_data).await?;
        }


        let unpicked_task = get_unpicked_task(get_pool().await).await?;
        if unpicked_task.is_some() {
            let task = unpicked_task.unwrap();
           if task.task_type == TaskType::ProofGeneration {
                info!("Picked up proof generation task --> {:?}", task);
                handle_proof_generation_task(task, config_data).await?;
            }
        } else {
            println!("No task available to pick");
        }
        sleep(sleep_duration);
    }
}