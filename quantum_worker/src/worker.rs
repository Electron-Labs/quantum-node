use anyhow::{anyhow, Result as AnyhowResult};
use chrono::Utc;
use once_cell::sync::Lazy;
use quantum_db::
    repository::{
        proof_repository::{
            get_n_reduced_proofs, update_proof_status, update_superproof_id_in_proof,
        },
        superproof_repository::{get_last_verified_superproof, insert_new_superproof, update_superproof_status},
        task_repository::{get_all_unpicked_tasks, update_task_status}
    };
use quantum_types::{
    enums::{ proof_status::ProofStatus,
        superproof_status::SuperproofStatus, task_status::TaskStatus, task_type::TaskType,
    },
    types::{
        config::ConfigData,
        db::{proof::Proof, task::Task},
    },
};
use quantum_utils::error_line;
use tokio::{sync::{Mutex, Semaphore}, time::Instant};
use std::{sync::Arc, thread::sleep, time::Duration};
use tracing::{error, info};
use crate::{aggregator::handle_proof_aggregation_and_updation, connection::get_pool};
use crate::proof_generator;


pub static GLOBAL_CYCLE_COUNTER: Lazy<Arc<Mutex<i64>>> = Lazy::new(|| Arc::new(Mutex::new(0)));
pub async fn increment_cycle(cycle_used: i64) {
    let counter: Arc<Mutex<i64>> = GLOBAL_CYCLE_COUNTER.clone(); // Clone the Arc for shared ownership
    let mut num = counter.lock().await; // Lock the mutex to safely modify the counter
    *num += cycle_used;
}

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
    let semaphore = Arc::new(Semaphore::new(config_data.parallel_bonsai_session_limit as usize));
    loop {
        println!("Running worker loop");
        let last_verified_superproof = get_last_verified_superproof(get_pool().await).await?;
        let aggregation_awaiting_proofs = get_n_reduced_proofs(get_pool().await, config_data.batch_size).await?;
        println!(
            "Aggregation awaiting proofs {:?}",
            aggregation_awaiting_proofs.len()
        );
        if last_verified_superproof.is_some() && aggregation_awaiting_proofs.len() > 0 && false{
            let last_verified_superproof = last_verified_superproof.unwrap(); // safe to use unwrap here, already check 
            let last_superproof_onchain_time = match last_verified_superproof.onchain_submission_time {
                Some(t) => Ok(t),
                None => Err(anyhow!(error_line!("onchain verified time field missing in last verified superproof"))),
            }?;
            let next_agg_start_time = last_superproof_onchain_time + Duration::from_secs(config_data.aggregation_wait_time);
            let remaining_time = next_agg_start_time - Utc::now().naive_utc();
            println!("remaining time for agg : {:?} seconds", remaining_time.num_seconds());
            if next_agg_start_time <= Utc::now().naive_utc() {
                let permit: tokio::sync::OwnedSemaphorePermit = semaphore.clone().acquire_owned().await.map_err(|e| anyhow!(error_line!(format!("error in acquiring the semaphore: {:?}", e))))?;
                info!("Picked up Proofs aggregation");
                aggregate_and_generate_new_superproof(aggregation_awaiting_proofs.clone(), config_data).await?;
                increment_cycle(-1 as i64 * (config_data.pr_batch_max_cycle_count as i64)).await;
                drop(permit);
            }
        }
        
        let unpicked_tasks = get_all_unpicked_tasks(get_pool().await).await?;
        let start =  Instant::now();
        info!("unpicked task count: {:?}", unpicked_tasks.len());
        for t in unpicked_tasks {
            if t.task_type == TaskType::ProofGeneration {
                let permit: tokio::sync::OwnedSemaphorePermit = semaphore.clone().acquire_owned().await.map_err(|e| anyhow!(error_line!(format!("error in acquiring the semaphore: {:?}", e))))?;
                info!("Picked up proof generation task --> {:?}", t);
                let config_data_clone = config_data.clone();
                let task = t.clone();

                let final_value = *GLOBAL_CYCLE_COUNTER.lock().await;
                info!("current cycle used count: {:?}", final_value);
                if final_value >= 0  && final_value as u64 >= config_data_clone.pr_batch_max_cycle_count {
                    info!("cycle count for current batch exceeds the limit");
                    continue;
                }
                
                let handle = tokio::spawn(async move {
                    let result = handle_proof_generation_task(task, &config_data_clone).await;
                    // Release the permit when the task is done
                    drop(permit);
                    result
                });

                tokio::spawn(async move {
                    
                    match handle.await {
                        Ok(Ok(())) => {
                            info!("Task {:?} finished successfully.", t.id);
                            let total_time = start.elapsed().as_secs();
                            info!("total time taken for 5 proofs: {:?}", total_time);
                        }
                        Ok(Err(e)) => {
                            error!("Task {:?} failed with error: {:?}, error in task updation", t.id, e);
                        }
                        Err(join_err) => {
                            error!("Failed to join task {:?}: {:?}", t.id, join_err);
                        }
                    }
                });
            }
        }
        
        sleep(sleep_duration);
    }
}