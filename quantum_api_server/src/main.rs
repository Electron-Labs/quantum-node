use config::load_config_data;
use config::ConfigData;
use connection::get_pool;
use dotenv::dotenv;
use error::error::CustomError;
use quantum_types::enums::proving_schemes::ProvingSchemes;
use quantum_types::types::gnark_groth16::GnarkGroth16Vkey;
use quantum_types::types::snarkjs_groth16::SnarkJSGroth16Vkey;
use rocket::State;
use service::register_circuit::get_circuit_registration_status;
use service::register_circuit::register_circuit_exec;
use rocket::serde::json::Json;
mod types;
mod service;
pub mod connection;
pub mod config;
pub mod utils;
pub mod error;

use anyhow::Result as AnyhowResult;
use types::circuit_registration_status::CircuitRegistrationStatusResponse;
use types::register_circuit::RegisterCircuitRequest;
use types::register_circuit::RegisterCircuitResponse;
// use types::snarkjs_groth16::SnarkJSGroth16Vkey;

use quantum_types;

#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/ping")]
fn ping() -> &'static str {
    service::ping::ping()
}

#[post("/register_circuit", data = "<data>")]
async fn register_circuit(data: RegisterCircuitRequest, config_data: &State<ConfigData>) -> AnyhowResult<Json<RegisterCircuitResponse>, CustomError> {
    let response: AnyhowResult<RegisterCircuitResponse>; 
    if data.proof_type == ProvingSchemes::GnarkGroth16 {
        response = register_circuit_exec::<GnarkGroth16Vkey>(data, config_data).await;
    } else if data.proof_type == ProvingSchemes::Groth16 {
        response = register_circuit_exec::<SnarkJSGroth16Vkey>(data, config_data).await;
    } else {
        return Err(CustomError::Internal(String::from("Unsupported Proving Scheme")))
    }
    match response {
        Ok(resp)  => Ok(Json(resp)),
        Err(e) => Err(CustomError::Internal(e.to_string()))
    }
}

#[get("/circuit/<circuit_id>/status")]
async fn get_circuit_reduction_status(circuit_id: String) -> AnyhowResult<Json<CircuitRegistrationStatusResponse>, CustomError>{
    let status = get_circuit_registration_status(circuit_id).await;
    match status {
        Ok(s) => Ok(Json(s)),
        Err(_) => Err(CustomError::Internal(String::from("error in db call")))
    }
}

#[launch]
async fn rocket() -> _ {
    dotenv().ok();
    let config_data = load_config_data();
    let _db_initialize = get_pool().await;
    rocket::build().manage(config_data).mount("/", routes![index, ping, register_circuit, get_circuit_reduction_status])
}