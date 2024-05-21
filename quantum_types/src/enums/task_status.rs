use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum TaskStatus {
    NotPicked = 1,
    InProgress = 2,
    Completed = 3,
    Failed = 4,
}

impl TaskStatus {
    pub fn as_u8(&self) -> u8 {
        match self {
            TaskStatus::NotPicked => 1,
            TaskStatus::InProgress => 2,
            TaskStatus::Completed => 3,
            TaskStatus::Failed => 4
        }
    }
}

impl From<u8> for TaskStatus {
    fn from(value: u8) -> Self {
        match value {
            1 => TaskStatus::NotPicked,
            2 => TaskStatus::InProgress,
            3 => TaskStatus::Completed,
            4 => TaskStatus::Failed,
            // TODO: remove panic
            _ => panic!("Invalid enum value"),
        }
    }
}

impl ToString for TaskStatus {
    fn to_string(&self) -> String {
        match self {
            TaskStatus::NotPicked => String::from("NotPicked"),
            TaskStatus::InProgress => String::from("InProgress"),
            TaskStatus::Completed => String::from("Completed"),
            TaskStatus::Failed => String::from("Failed"),
        }
    }
}