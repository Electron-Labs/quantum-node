use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct UserCircuitData{
    pub circuit_hash: String,
    pub vk_path: String,
    pub cd_path: String,
    pub reduction_circuit_id: Option<u64>,
    pub pis_len: u64
}