use serde::Serialize;


#[derive(Serialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct ProtocolProofResponse {
    pub protocol_vkey_hash: String, //"0x" hex
    pub reduction_vkey_hash: String,
    pub merkle_proof_position: u64,
    pub merkle_proof: Vec<String>,
    pub leaf_next_value: String, // hex
    pub leaf_next_index: String,
    pub superproof_root: String
}