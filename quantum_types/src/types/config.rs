use std::fs;

use serde::{Deserialize, Serialize};
use tracing::info;
use dotenv::dotenv;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConfigData {
    pub storage_folder_path: String,
    pub user_data_path: String,
    pub proof_path: String,
    pub public_inputs_path: String,
    pub reduced_proof_path: String,
    pub reduced_pis_path: String,
    pub reduced_proof_receipt_path: String,
    pub reduced_circuit_path: String,
    pub imt_circuit_data_path: String,
    pub aggregated_circuit_data: String,
    pub supperproof_path: String,
    pub verification_contract_address: String,
    pub imt_depth: u64,
    pub max_batch_size: u64,
    pub worker_sleep_secs: u64,
    pub aggregation_wait_time: u64,
    pub snark_reduction_data_path: String,
    pub parallel_bonsai_session_limit: u64,
    pub pr_batch_max_cycle_count: u64
}

impl ConfigData {
    pub fn new(path: &str) -> ConfigData {
        let config_contents_str = fs::read_to_string(path).expect("provide a valid path");
        let config_data = serde_yaml::from_str(&config_contents_str).unwrap();
        info!("config data loaded: {:?}", config_data);
        return config_data;
    }
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AMQPConfigData {
    pub proof_request_queue: String,
    pub proof_reply_queue: String,
    pub rabbitmq_endpoint: String,
}

impl AMQPConfigData {
    pub fn get_config() -> AMQPConfigData {
        dotenv().ok();

        let proof_request_queue = std::env::var("PROOF_REQUEST_QUEUE")
            .expect("`PROOF_REQUEST_QUEUE` env variable must be set");
        let proof_reply_queue = std::env::var("PROOF_REPLY_QUEUE")
            .expect("`PROOF_REPLY_QUEUE` env variable must be set");
        let rabbitmq_endpoint = std::env::var("RABBITMQ_ENDPOINT")
            .expect("`RABBITMQ_ENDPOINT` env variable must be set");

        AMQPConfigData {
            proof_request_queue,
            proof_reply_queue,
            rabbitmq_endpoint,
        }
    }
}
