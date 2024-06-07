use serde::Serialize;


#[derive(Serialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct ProtocolProofResponse {
    pub proof: Vec<Vec<u8>>,
    pub proof_helper: Vec<u8>
}