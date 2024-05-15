use crate::types::{register_circuit::RegisterCircuitRequest, register_circuit::RegisterCircuitResponse};

pub fn register_circuit(data: RegisterCircuitRequest) -> RegisterCircuitResponse {
    // 1. Dump whatever you need in db/queue 
    // 2. Just return the keccak hash of the data.vkey at this point
    // 3. An async worker actually takes care of the registration request
    let hash = "0x00";
    RegisterCircuitResponse { 
        circuit_hash: String::from(hash)
    }
}