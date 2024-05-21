use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum ProofStatus {
    NotFound = 1,
    Registered = 2,
    Reducing = 3,
    Reduced = 4,
    Aggregating = 5,
    Aggregated = 6,
    Verified = 7
}

impl ProofStatus {
    pub fn as_u8(&self) -> u8 {
        match self {
            ProofStatus::NotFound => 1,
            ProofStatus::Registered => 2,
            ProofStatus::Reducing => 3,
            ProofStatus::Reduced => 4,
            ProofStatus::Aggregating => 5,
            ProofStatus::Aggregated => 6,
            ProofStatus::Verified => 7
        }
    }
}

impl From<u8> for ProofStatus {
    fn from(value: u8) -> Self {
        match value {
            1 => ProofStatus::NotFound,
            2 => ProofStatus::Registered,
            3 => ProofStatus::Reducing,
            4 => ProofStatus::Reduced,
            5 => ProofStatus::Aggregating,
            6 => ProofStatus::Aggregated,
            7 => ProofStatus::Verified,
            // TODO: remove panic
            _ => panic!("Invalid enum value"),
        }
    }
}

impl ToString for ProofStatus {
    fn to_string(&self) -> String {
        match self {
            ProofStatus::NotFound => String::from("NotFound"),
            ProofStatus::Registered => String::from("Registered"),
            ProofStatus::Reducing => String::from("Reducing"),
            ProofStatus::Reduced => String::from("Reduced"),
            ProofStatus::Aggregating => String::from("Aggregating"),
            ProofStatus::Aggregated => String::from("Aggregated"),
            ProofStatus::Verified => String::from("Verified"),
        }
    }
}