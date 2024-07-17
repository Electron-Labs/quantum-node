// use aggregator::handle_aggregation;
use anyhow::{anyhow, Result as AnyhowResult};
use dotenv::dotenv;
// use imt::handle_imt;
use quantum_db::{
    error::error::CustomError,
    repository::{
        proof_repository::{
            get_n_reduced_proofs, update_proof_status, update_superproof_id_in_proof,
        },
        superproof_repository::{insert_new_superproof, update_superproof_status},
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
use tracing::{error, info};
use crate::connection::get_pool;
use crate::imt::handle_imt;
use crate::{proof_generator, registration};
use crate::aggregator::handle_aggregation;

pub async fn handle_register_circuit_task(
    registration_task: Task,
    config: &ConfigData,
) -> AnyhowResult<()> {
    let user_circuit_hash = registration_task.clone().user_circuit_hash;

    // Change Task status to InProgress
    update_task_status(get_pool().await, registration_task.id.unwrap(), TaskStatus::InProgress).await?;

    // Change user_circuit_data.circuit_reduction_status to InProgress
    update_user_circuit_data_reduction_status(get_pool().await, &user_circuit_hash, CircuitReductionStatus::InProgress).await?;

    let request = registration::handle_circuit_registration(registration_task.clone(), config).await;

    match request {
        Ok(_) => {
            // Change user_circuit_data.circuit_reduction_status to Completed
            update_user_circuit_data_reduction_status(get_pool().await, &user_circuit_hash, CircuitReductionStatus::SmartContractRgistrationPending).await?;

            // Set Task Status to Completed
            update_task_status(get_pool().await, registration_task.id.unwrap(), TaskStatus::Completed).await?;

            info!("Circuit registered successfully");
        }
        Err(e) => {
            // Update db task to failed and circuit reduction to failed too
            update_user_circuit_data_reduction_status(get_pool().await, &user_circuit_hash, CircuitReductionStatus::Failed).await?;

            // Set Task Status to failed
            update_task_status(get_pool().await, registration_task.id.unwrap(), TaskStatus::Failed).await?;
            error!(
                "Circuit registration failed : {:?}",
                e.root_cause().to_string()
            );
        }
    }
    Ok(())
}

pub async fn aggregate_proofs(
    proofs: Vec<Proof>,
    config: &ConfigData,
    superproof_id: u64,
) -> AnyhowResult<()>
{
    for proof in &proofs {
        update_proof_status(get_pool().await, &proof.proof_hash, ProofStatus::Aggregating).await?;
    }

    for proof in &proofs {
        update_superproof_id_in_proof(get_pool().await, &proof.proof_hash, superproof_id).await?;
    }
    // 4. superproof_status -> (0: Not Started, 1: IN_PROGRESS, 2: PROVING_DONE, 3: SUBMITTED_ONCHAIN, 4: FAILED)

    let aggregation_request = handle_aggregation(proofs.clone(), superproof_id, config).await;

    match aggregation_request {
        Ok(_) => {
            // Update Proof Status to aggregated for all the proofs
            for proof in proofs {
                update_proof_status(get_pool().await, &proof.proof_hash, ProofStatus::Aggregated).await?;
            }
            // Superproof status to PROVING_DONE
            info!("changing the superproof status to proving done");
            update_superproof_status(get_pool().await, SuperproofStatus::ProvingDone, superproof_id).await?;
        }
        Err(e) => {
            error!("aggregation_request error {:?}", e);

            // Change proof_generation status to FAILED
            for proof in proofs {
                update_proof_status(get_pool().await, &proof.proof_hash, ProofStatus::AggregationFailed).await?;
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
    let proof_hash = proof_generation_task.clone().proof_id.clone().unwrap();
    // Change Task status to InProgress
    update_task_status(get_pool().await, proof_generation_task.clone().id.unwrap(), TaskStatus::InProgress).await?;
    info!("Updated Task Status to InProgress");

    // Update Proof Status to Reducing
    update_proof_status(get_pool().await, &proof_hash, ProofStatus::Reducing).await?;
    info!("Update Proof Status to Reducing");

    let proof_hash = match proof_generation_task.proof_id {
        None => Err(anyhow!(error_line!("Proof generation task does not contain the proof id"))),
        Some(p) => Ok(p),
    }?;
    let request = proof_generator::handle_proof_generation_and_updation(&proof_hash, &proof_generation_task.user_circuit_hash, config).await;

    match request {
        Ok(_) => {
            // Change proof_generation status to REDUCED
            update_proof_status(get_pool().await, &proof_hash, ProofStatus::Reduced).await?;
            info!("Changed proof status to REDUCED");

            // Update task status to completed
            update_task_status(get_pool().await, proof_generation_task.clone().id.unwrap(), TaskStatus::Completed).await?;
            info!("Changed task status to Completed");

            info!("Proof Reduced Successfully");
        }
        Err(e) => {
            // Change proof_generation status to FAILED
            update_proof_status(get_pool().await, &proof_hash, ProofStatus::ReductionFailed).await?;
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

    handle_imt(aggregation_awaiting_proofs.clone(), superproof_id, config_data, ).await?;
    aggregate_proofs(aggregation_awaiting_proofs, config_data, superproof_id).await?;

    Ok(())
}

pub async fn worker(sleep_duration: Duration, config_data: &ConfigData) -> AnyhowResult<()> {
    loop {
        println!("Running worker loop");
        let aggregation_awaiting_proofs = get_n_reduced_proofs(get_pool().await, config_data.batch_size).await?;
        println!(
            "Aggregation awaiting proofs {:?}",
            aggregation_awaiting_proofs.len()
        );
        if aggregation_awaiting_proofs.len() == config_data.batch_size as usize {
            info!("Picked up Proofs aggregation");
            aggregate_and_generate_new_superproof(aggregation_awaiting_proofs.clone(), config_data).await?;
        }

        let unpicked_task = get_unpicked_task(get_pool().await).await?;
        if unpicked_task.is_some() {
            let task = unpicked_task.unwrap();
            if task.task_type == TaskType::CircuitReduction {
                info!("Picked up circuit reduction task --> {:?}", task);
                handle_register_circuit_task(task, config_data).await?;
            } else if task.task_type == TaskType::ProofGeneration {
                info!("Picked up proof generation task --> {:?}", task);
                handle_proof_generation_task(task, config_data).await?;
            }
        } else {
            println!("No task available to pick");
        }
        sleep(sleep_duration)
    }
}