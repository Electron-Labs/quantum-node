use crate::types::{request::register_circuit::RegisterCircuitRequest, response::regsiter_circuit::RegisterCircuitResponse};

pub fn register_circuit(data: RegisterCircuitRequest) -> RegisterCircuitResponse {
    RegisterCircuitResponse { 
        circuit_hash: data.name.clone() ,
        circuit: data.name
    }
}