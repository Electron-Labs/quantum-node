/*
    Quantum Worker: Responsibilities (Regsiters Circuit, Generate Proof, Aggregate Proofs + Submit on Ethereum)
    Task Table :  id, user_circuit_hash, task_type, proof_id, task_status
    User Circuit Table: id, vk_path, reduction_circuit_id, n_pis, n_commitments, proving_scheme, circuit_reduction_status
    Redn Circuit Table: id, proving_key_path, vk_path, n_inner_pis, n_inner_commitments
    Proof Table: id, user_circuit_id(FK), proof_hash, pis_path, proof_path, reduction_proof_path, reduction_proof_pis_path, superproof_id, reduction_time, proof_status

    Task worker keeps running in loop, as soon as it comes to top of the loop, does following stuff in same priority order:
    1. Checks if we have BATCH_SIZE number of proofs reduced, if yes run AGGREGATION and then submit on Ethereum
    2. Check if theres any registration or proof gen pending task available, if yes DO IT.
*/
use std::time::Duration;
use dotenv::dotenv;
use tracing::{error, info};
use quantum_types::types::config::ConfigData;
use quantum_utils::logger::initialize_logger;
use quantum_worker::connection::get_pool;
use quantum_worker::worker::worker;

#[tokio::main]
async fn main() {
    info!(" --- Load env configuration ---");
    dotenv().ok();
    info!(" --- Starting worker --- ");
    let _guard = initialize_logger("qunatum_node_worker.log");
    let config_data = ConfigData::new("./config.yaml");
    let _pool = get_pool().await;
    let worker_sleep_duration = Duration::from_secs(config_data.worker_sleep_secs);
    match worker(worker_sleep_duration, &config_data).await {
        Ok(_) => {
            info!("stopping worker");
        }
        Err(e) => {
            error!("error encountered in worker: {}", e);
            error!("worker stopped");
        }
    };
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn () {
//
//     }
// }
