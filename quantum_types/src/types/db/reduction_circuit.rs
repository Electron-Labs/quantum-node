use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReductionCircuit {
    pub id: Option<u64>,
    pub proving_key_path: String,
    pub vk_path: String,
    pub pis_len: String
}