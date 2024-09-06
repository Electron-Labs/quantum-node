use rocket::serde;
use ::serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Debug, Deserialize)]
#[serde(crate = "rocket::serde")]

pub struct CircuitRegistrationStatusResponse {
    pub circuit_registration_status: String
}
