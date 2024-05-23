use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReductionCircuit {
    pub circuit_id: String,
    pub proving_key_path: String,
    pub vk_path: String,
    pub pis_len: u8
}