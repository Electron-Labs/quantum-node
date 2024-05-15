use rocket::serde::Serialize;


#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct RegisterCircuitResponse {
    pub circuit_hash: String,
}

