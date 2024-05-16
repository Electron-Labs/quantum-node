use serde::{Deserialize, Serialize};

use crate::enums::{circuit_reduction_status::CircuitReductionStatus, proof_status::ProofStatus, task_type::TaskType};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Task {
    pub id: Option<u64>,
    pub user_circuit_hash: String,
    pub task_type: TaskType,
    pub proof_id: Option<u64>,
    pub proof_status: Option<ProofStatus>,
    pub circuit_reduction_status: Option<CircuitReductionStatus> 
}