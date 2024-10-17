use serde::Serialize;


#[derive(Serialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct ProtocolProofResponse {
    pub merkle_proof_position: u64,
    pub merkle_proof: Vec<String>,
}