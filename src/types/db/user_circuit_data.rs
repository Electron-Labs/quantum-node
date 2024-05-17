use serde::{Deserialize, Serialize};

use crate::types::proving_schemes::ProvingSchemes;


#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct UserCircuitData{
    pub circuit_hash: String,
    pub vk_path: String,
    pub reduction_circuit_id: Option<u64>,
    pub pis_len: u64,
    pub proving_scheme: ProvingSchemes
}