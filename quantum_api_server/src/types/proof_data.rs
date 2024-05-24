use rocket::serde;
use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(crate = "rocket::serde")]

pub struct ProofDataResponse {
    pub status: String,
    pub superproof_id: i64,
    pub transaction_hash: Option<String>,
    pub verification_contract: String
}