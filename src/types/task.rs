use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Task {
    pub id: Option<u64>,
    pub user_circuit_hash: String,
    pub task_type: u64,
    pub proof_id: Option<u64>,
    pub proof_status: Option<u64>,
    pub circuit_reduction_status: Option<u64> 
}