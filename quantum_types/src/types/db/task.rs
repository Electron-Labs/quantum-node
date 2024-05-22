use serde::{Deserialize, Serialize};

use crate::enums::{task_status::TaskStatus, task_type::TaskType};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Task {
    pub id: Option<u64>,
    pub user_circuit_hash: String,
    pub task_type: TaskType,
    pub proof_id: Option<String>,
    pub task_status: TaskStatus
}