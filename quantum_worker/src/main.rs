/*
    Quantum Worker: Responsibilities (Regsiters Circuit, Generate Proof, Aggregate Proofs + Submit on Ethereum)
    Task Table :  id, user_circuit_hash, task_type, proof_id, task_status
    User Circuit Table: id, vk_path, reduction_circuit_id, pis_len, proving_scheme, circuit_reduction_status
    Redn Circuit Table: id, proving_key_path, vk_path, pis_len
    Proof Table: id, user_circuit_id(FK), proof_hash, pis_path, proof_path, reduction_proof_path, reduction_proof_pis_path, superproof_id, reduction_time, proof_status

    Task worker keeps running in loop, as soon as it comes to top of the loop, does following stuff in same priority order:
    1. Checks if we have BATCH_SIZE number of proofs reduced, if yes run AGGREGATION and then submit on Ethereum
    2. Check if theres any registration or proof gen pending task available, if yes DO IT.
*/

pub mod connection;
pub mod aggregator;
pub mod registration;
pub mod proof_generator;
pub mod utils;

use std::{thread::sleep, time::Duration};
use dotenv::dotenv;
use quantum_db::repository::{proof_repository::{get_n_reduced_proofs, update_proof_status}, task_repository::{get_aggregation_waiting_tasks_num, get_unpicked_task, update_task_status}, user_circuit_data_repository::update_user_circuit_data_reduction_status};
use anyhow::Result as AnyhowResult;
use quantum_types::{enums::{circuit_reduction_status::CircuitReductionStatus, proof_status::ProofStatus, task_status::TaskStatus, task_type::TaskType}, types::{config::ConfigData, db::{proof::Proof, task::Task}}};
use sqlx::{MySql, Pool};
use quantum_utils::logger::initialize_logger;
use tracing::info;

pub const BATCH_SIZE: u64 = 20; // Number of proofs to be included in 1 batch
pub const WORKER_SLEEP_SECS: u64 = 10;

pub async fn regsiter_circuit(pool: &Pool<MySql>, registration_task: Task, config: &ConfigData) -> AnyhowResult<()> {
    let user_circuit_hash = registration_task.clone().user_circuit_hash;

    // Change Task status to InProgress
    update_task_status(pool, registration_task.id.unwrap(), TaskStatus::InProgress).await?;

    // Change user_circuit_data.circuit_reduction_status to InProgress
    update_user_circuit_data_reduction_status(pool, &user_circuit_hash, CircuitReductionStatus::InProgress).await?;

    let request = registration::handle_registration_task(pool, registration_task.clone(), config).await;

    match request {
        Ok(_) => {
            // Change user_circuit_data.circuit_reduction_status to Completed
            update_user_circuit_data_reduction_status(pool, &user_circuit_hash, CircuitReductionStatus::Completed).await?;

            // Set Task Status to Completed
            update_task_status(pool, registration_task.id.unwrap(), TaskStatus::Completed).await?;

            println!("Circuit registered successfully");
        },
        Err(e) => {
            // Update db task to failed and circuit reduction to failed too
            update_user_circuit_data_reduction_status(pool, &user_circuit_hash, CircuitReductionStatus::Failed).await?;

            // Set Task Status to failed
            update_task_status(pool, registration_task.id.unwrap(), TaskStatus::Failed).await?;
            println!("Circuit registration failed : {:?}", e.to_string());
        },
    }
    Ok(())
}

pub async fn aggregate_proofs(pool: &Pool<MySql>, proofs: Vec<Proof>) -> AnyhowResult<()> {

    // Update Proof Status to aggregating for all the proofs
    for proof in proofs {
        update_proof_status(pool, &proof.proof_hash, ProofStatus::Aggregating).await?;
    }
    Ok(())   
}


pub async fn generate_reduced_proof(pool: &Pool<MySql>, proof_generation_task: Task, config: &ConfigData) -> AnyhowResult<()> {
    let proof_hash = proof_generation_task.clone().proof_id.clone().unwrap();
    // Change Task status to InProgress
    update_task_status(pool, proof_generation_task.clone().id.unwrap(), TaskStatus::InProgress).await?;
    println!("Updated Task Status to InProgress");

    // Update Proof Status to Reducing
    update_proof_status(pool, &proof_hash, ProofStatus::Reducing).await?;
    println!("Update Proof Status to Reducing");

    let request = proof_generator::handle_proof_generation_task(pool, proof_generation_task.clone(), config).await;

    match request {
        Ok(_) => {
            // Change proof_generation status to REDUCED
            update_proof_status(pool, &proof_hash, ProofStatus::Reduced).await?;
            println!("Changed proof status to REDUCED");

            // Update task status to completed
            update_task_status(pool, proof_generation_task.clone().id.unwrap(), TaskStatus::Completed).await?;
            println!("Changed task status to Completed");

            println!("Proof Reduced Successfully");
        },  
        Err(e) => {
            // Change proof_generation status to FAILED
            update_proof_status(pool, &proof_hash, ProofStatus::ReductionFailed).await?;
            println!("Changed Proof Status to FAILED");

            // Update task status to failed
            update_task_status(pool, proof_generation_task.clone().id.unwrap(), TaskStatus::Failed).await?;
            println!("Changed Task Status to FAILED");

            println!("Proof Reduction Failed: {:?}", e.to_string());
        }
    }

    Ok(())
}

pub async fn worker(sleep_duration: Duration, config_data: &ConfigData) -> AnyhowResult<()> {
    println!(" --- Initialising DB connection pool ---");
    let pool = connection::get_pool().await;
    loop {
        println!("Running worker loop");
        // let aggregation_awaiting_tasks = get_aggregation_waiting_tasks_num(pool).await?;
        let aggregation_awaiting_proofs = get_n_reduced_proofs(pool, BATCH_SIZE).await?;
        println!("Aggregation awaiting proofs {:?}", aggregation_awaiting_proofs.len());
        if aggregation_awaiting_proofs.len() == BATCH_SIZE as usize {
            // TODO: Do aggregation and submit on ethereum
        }

        let unpicked_task = get_unpicked_task(pool).await?;
        if unpicked_task.is_some() {
            let task = unpicked_task.unwrap();
            if task.task_type == TaskType::CircuitReduction {
                println!("Picked up circuit reduction task --> {:?}", task);
                regsiter_circuit(pool, task, config_data).await?;
            } else if task.task_type == TaskType::ProofGeneration {
                println!("Picked up proof generation task --> {:?}", task);
                generate_reduced_proof(pool, task, config_data).await?;
            }
        } else {
            println!("No task available to pick");
        }
        sleep(sleep_duration)
    }
}

#[tokio::main]
async fn main() {
    println!(" --- Load env configuration ---");
    dotenv().ok();
    println!(" --- Starting worker --- ");
    let _guard = initialize_logger("qunatum_node_worker.log");
    let worker_sleep_duration = Duration::from_secs(WORKER_SLEEP_SECS);
    let config_data = ConfigData::new("./config.yaml");
    let _res = worker(worker_sleep_duration, &config_data).await;
}
