use keccak_hash::keccak;

use crate::types::{register_circuit::RegisterCircuitRequest, register_circuit::RegisterCircuitResponse};

pub fn register_circuit_exec(data: RegisterCircuitRequest) -> RegisterCircuitResponse {
    // 1. Dump whatever you need in db/queue 
    // 2. Just return the keccak hash of the data.vkey at this point
    // Keccak hash borshified vkey for now
    let vkey = data.vkey.clone();
    let mut keccak_ip = vkey.as_slice();
    let hash = keccak(&mut keccak_ip);
    // 3. An async worker actually takes care of the registration request
    RegisterCircuitResponse { 
        circuit_hash: format!("{:?}", hash)
    }
}