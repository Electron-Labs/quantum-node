use serde::{Deserialize, Serialize};

use crate::enums::{circuit_reduction_status::CircuitReductionStatus, proving_schemes::ProvingSchemes};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserCircuitData{
    pub circuit_hash: String,
    pub vk_path: String,
    pub reduction_circuit_id: Option<String>,
    pub pis_len: u8,
    pub proving_scheme: ProvingSchemes,
    pub circuit_reduction_status: CircuitReductionStatus
}