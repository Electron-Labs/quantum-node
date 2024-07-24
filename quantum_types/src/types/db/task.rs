use serde::{Deserialize, Serialize};

use crate::enums::{task_status::TaskStatus, task_type::TaskType};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Task {
    pub id: Option<u64>,
    pub user_circuit_hash: String,
    pub task_type: TaskType,
    pub proof_hash: Option<String>,
    pub proof_id: Option<u64>,
    pub task_status: TaskStatus
}