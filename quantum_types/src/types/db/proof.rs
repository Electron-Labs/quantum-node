use serde::{Deserialize, Serialize};
use crate::enums::proof_status::ProofStatus;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Proof {
    pub id: Option<u64>,
    pub proof_hash: String,
    pub pis_path: String,
    pub proof_path: String,
    // pub reduction_proof_path: Option<String>,
    // pub reduction_proof_pis_path: Option<String>,
    pub input_id: Option<String>,
    pub session_id: Option<String>,
    pub superproof_id: Option<u64>,
    pub reduction_time: Option<u64>,
    pub proof_status: ProofStatus,
    pub user_circuit_hash: String
}