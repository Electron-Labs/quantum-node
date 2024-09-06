use serde::{Deserialize, Serialize};

use crate::enums::{circuit_reduction_status::CircuitReductionStatus, proving_schemes::ProvingSchemes};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserCircuitData{
    pub circuit_hash: String,
    pub vk_path: String,
    pub proving_scheme: ProvingSchemes,
    pub protocol_name: String,
    pub bonsai_image_id: String,
    pub circuit_reduction_status: CircuitReductionStatus,
}