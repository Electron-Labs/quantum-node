use serde::{Deserialize, Serialize};

use crate::enums::task_type::TaskType;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Task {
    pub id: Option<u64>,
    pub user_circuit_hash: Option<String>,
    pub task_type: TaskType,
    pub proof_id: Option<u64>,
    pub task_status: TaskType
}