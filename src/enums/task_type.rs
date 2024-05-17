use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum TaskType {
    CircuitReduction = 1,
    ProofGeneration = 2
}

impl TaskType {
    pub fn as_u8(&self) -> u8 {
        match self {
            TaskType::CircuitReduction => 1,
            TaskType::ProofGeneration => 2
        }
    }

    fn from(value: u8) -> Self {
        match value {
            1 => TaskType::CircuitReduction,
            2 => TaskType::ProofGeneration,
            _ => panic!("Invalid enum value"),
        }
    }
}