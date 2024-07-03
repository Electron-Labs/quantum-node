use std::fs;

use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConfigData {
    pub storage_folder_path: String,
    pub user_data_path: String,
    pub proof_path: String,
    pub public_inputs_path: String,
    pub reduced_proof_path: String,
    pub reduced_pis_path: String,
    pub reduced_circuit_path: String,
    pub aggregated_circuit_data: String,
    pub supperproof_path: String,
    pub verification_contract_address: String,
}

impl ConfigData {
    pub fn new(path: &str) -> ConfigData {
        let config_contents_str = fs::read_to_string(path).expect("provide a valid path");
        let config_data = serde_yaml::from_str(&config_contents_str).unwrap();
        info!("config data loaded: {:?}", config_data);
        return config_data;
    }
}