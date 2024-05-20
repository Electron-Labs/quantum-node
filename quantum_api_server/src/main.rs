use config::load_config_data;
use config::ConfigData;
use connection::get_pool;
use dotenv::dotenv;
use error::error::CustomError;
use rocket::State;
use service::register_circuit::register_circuit_exec;
use types::gnark_groth16::GnarkGroth16Vkey;
use types::proving_schemes::ProvingSchemes;
use types::register_circuit::RegisterCircuitRequest;
use types::register_circuit::RegisterCircuitResponse;
use rocket::serde::json::Json;
mod types;
mod service;
pub mod enums;
pub mod repository;
pub mod connection;
pub mod config;
pub mod utils;
pub mod error;

use anyhow::Result as AnyhowResult;
use types::snarkjs_groth16::SnarkJSGroth16Vkey;

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

#[launch]
async fn rocket() -> _ {
    dotenv().ok();
    let config_data = load_config_data();
    let _db_initialize = get_pool().await;
    rocket::build().manage(config_data).mount("/", routes![index, ping, register_circuit])
}