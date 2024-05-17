use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum CircuitReductionStatus {
    NotPicked = 1,
    InProgress = 2,
    Completed = 3,
    Failed = 4,
}

impl CircuitReductionStatus {
    pub fn as_u8(&self) -> u8 {
        match self {
            CircuitReductionStatus::NotPicked => 1,
            CircuitReductionStatus::InProgress => 2,
            CircuitReductionStatus::Completed => 3,
            CircuitReductionStatus::Failed => 4
        }
    }

    #[allow(dead_code)]
    fn from(value: u8) -> Self {
        match value {
            1 => CircuitReductionStatus::NotPicked,
            2 => CircuitReductionStatus::InProgress,
            3 => CircuitReductionStatus::Completed,
            4 => CircuitReductionStatus::Failed,
            _ => panic!("Invalid enum value"),
        }
    }
}