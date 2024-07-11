use rocket::serde;
use ::serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Debug, Deserialize)]
#[serde(crate = "rocket::serde")]

pub struct ProofDataResponse {
    pub status: String,
    pub superproof_id: i64,
    pub transaction_hash: Option<String>,
    pub verification_contract: String
}