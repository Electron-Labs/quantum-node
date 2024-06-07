use rocket::serde;
use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(crate = "rocket::serde")]

pub struct CircuitRegistrationStatusResponse {
    pub circuit_registration_status: String,
    pub reduction_circuit_hash: Option<String>
}
