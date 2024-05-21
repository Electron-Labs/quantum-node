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

use std::{thread::sleep, time::Duration};
use dotenv::dotenv;
use quantum_db::repository::task_repository::{get_aggregation_waiting_tasks_num, get_unpicked_circuit_reduction_task};
use anyhow::Result as AnyhowResult;
use quantum_types::enums::task_type::TaskType;

pub const BATCH_SIZE: u64 = 20; // Number of proofs to be included in 1 batch
pub const WORKER_SLEEP_SECS: u64 = 2;

pub async fn worker(sleep_duration: Duration) -> AnyhowResult<()> {
    println!(" --- Initialising DB connection pool ---");
    let pool = connection::get_pool().await;
    loop {
        println!("Running worker loop");
        let aggregation_awaiting_tasks = get_aggregation_waiting_tasks_num(pool).await?;
        if aggregation_awaiting_tasks >= BATCH_SIZE {
            // TODO: Do aggregation and submit on ethereum
        }

        let unpicked_task = get_unpicked_circuit_reduction_task(pool).await?;
        if unpicked_task.is_some() {
            let task = unpicked_task.unwrap();
            if task.task_type == TaskType::CircuitReduction {
                // TODO: Compile Reduction Circuit and do DB updations accordingly
            } else if task.task_type == TaskType::ProofGeneration {
                // TODO: Generate reduced proof and do DB updations accordingly
            }
        }
        sleep(sleep_duration)
    }
}

#[tokio::main]
async fn main() {
    println!(" --- Load env configuration ---");
    dotenv().ok();
    println!(" --- Starting worker --- ");
    let worker_sleep_duration = Duration::from_secs(WORKER_SLEEP_SECS);
    let _res = worker(worker_sleep_duration).await;
}
