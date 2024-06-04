use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum SuperproofStatus {
    NotStarted = 0,
    InProgress = 1,
    ProvingDone = 2,
    SubmittedOnchain = 3,
    Failed = 4,
}

impl SuperproofStatus {
    pub fn as_u8(&self) -> u8 {
        match self {
            SuperproofStatus::NotStarted => 0,
            SuperproofStatus::InProgress => 1,
            SuperproofStatus::ProvingDone => 2,
            SuperproofStatus::SubmittedOnchain => 3,
            SuperproofStatus::Failed => 4,
        }
    }
}

impl From<u8> for SuperproofStatus {
    fn from(value: u8) -> Self {
        match value {
            0 => SuperproofStatus::NotStarted,
            1 => SuperproofStatus::InProgress,
            2 => SuperproofStatus::ProvingDone,
            3 => SuperproofStatus::SubmittedOnchain,
            4 => SuperproofStatus::Failed,
            // TODO: remove panic
            _ => panic!("Invalid enum value"),
        }
    }
}

impl ToString for SuperproofStatus {
    fn to_string(&self) -> String {
        match self {
            SuperproofStatus::NotStarted => String::from("NotStarted"),
            SuperproofStatus::InProgress => String::from("InProgress"),
            SuperproofStatus::ProvingDone => String::from("ProvingDone"),
            SuperproofStatus::SubmittedOnchain => String::from("SubmittedOnchain"),
            SuperproofStatus::Failed => String::from("Failed"),
        }
    }
}