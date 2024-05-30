use serde::{Deserialize, Serialize};

use crate::enums::proving_schemes::ProvingSchemes;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReductionCircuit {
    pub circuit_id: String,
    pub proving_key_path: String,
    pub vk_path: String,
    pub pis_len: u8,
    pub proving_scheme: ProvingSchemes
}