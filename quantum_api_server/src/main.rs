use connection::get_pool;
use dotenv::dotenv;
use error::error::CustomError;
use quantum_db::repository::proof_repository::get_proof_by_proof_hash;
use quantum_db::repository::protocol::get_protocol_by_auth_token;
use quantum_db::repository::reduction_circuit_repository::get_reduction_circuit_for_user_circuit;
use quantum_db::repository::superproof_repository::get_last_verified_superproof;
use quantum_types::enums::proving_schemes::ProvingSchemes;
use quantum_types::types::gnark_groth16::GnarkGroth16Pis;
use quantum_types::types::gnark_groth16::GnarkGroth16Proof;
use quantum_types::types::gnark_groth16::GnarkGroth16Vkey;
use quantum_types::types::snarkjs_groth16::SnarkJSGroth16Pis;
use quantum_types::types::snarkjs_groth16::SnarkJSGroth16Proof;
use quantum_types::types::snarkjs_groth16::SnarkJSGroth16Vkey;
use quantum_types::types::config::ConfigData;
use quantum_utils::logger::initialize_logger;
use rocket::State;
use service::proof::get_protocol_proof_exec;
use service::protocol::generate_auth_token_for_protocol;
use service::proof::get_proof_data_exec;
use service::proof::submit_proof_exec;
use service::register_circuit::get_circuit_registration_status;
use service::register_circuit::register_circuit_exec;
use rocket::serde::json::Json;
mod types;
mod service;
pub mod connection;
pub mod error;

use anyhow::Result as AnyhowResult;
use tracing::info;
use types::auth::AuthToken;
use types::circuit_registration_status::CircuitRegistrationStatusResponse;
use types::generate_auth_token::GenerateAuthTokenRequest;
use types::generate_auth_token::GenerateAuthTokenResponse;
use types::proof_data::ProofDataResponse;
use types::register_circuit::RegisterCircuitRequest;
use types::register_circuit::RegisterCircuitResponse;
use types::protocol_proof::ProtocolProofResponse;

use quantum_types;
use types::submit_proof::SubmitProofRequest;
use types::submit_proof::SubmitProofResponse;

#[macro_use] extern crate rocket;


#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/ping")]
fn ping(_auth_token: AuthToken) -> &'static str {
    service::ping::ping()
}

#[post("/register_circuit", data = "<data>")]
async fn register_circuit(auth_token: AuthToken, data: RegisterCircuitRequest, config_data: &State<ConfigData>) -> AnyhowResult<Json<RegisterCircuitResponse>, CustomError> {
    let response: AnyhowResult<RegisterCircuitResponse>; 
    let protocol = match get_protocol_by_auth_token(get_pool().await, &auth_token.0).await {
        Ok(p) => Ok(p),
        Err(_) => {
            info!("error in db while fetching protocol");
            Err(CustomError::Internal("error in db while fetching protocol".to_string()))
        },
    };
    let protocol = protocol?;
    info!("{:?}", protocol);
    let protocol = match protocol {
        Some(p) => Ok(p),
        None => {
            info!("No protocol against this auth token");
            Err(CustomError::Internal("No protocol against this auth token".to_string()))
        },
    };
    let protocol = protocol?;

    info!("{:?}", protocol);

    if data.proof_type == ProvingSchemes::GnarkGroth16 {
        response = register_circuit_exec::<GnarkGroth16Vkey>(data, config_data, protocol).await;
    } else if data.proof_type == ProvingSchemes::Groth16 {
        response = register_circuit_exec::<SnarkJSGroth16Vkey>(data, config_data, protocol).await;
    } else {
        return Err(CustomError::Internal(String::from("Unsupported Proving Scheme")))
    }
    match response {
        Ok(resp)  => Ok(Json(resp)),
        Err(e) => Err(CustomError::Internal(e.to_string()))
    }
}

#[get("/circuit/<circuit_id>/status")]
async fn get_circuit_reduction_status(_auth_token: AuthToken, circuit_id: String) -> AnyhowResult<Json<CircuitRegistrationStatusResponse>, CustomError>{
    let status = get_circuit_registration_status(circuit_id).await;
    match status {
        Ok(s) => Ok(Json(s)),
        Err(_) => Err(CustomError::Internal(String::from("error in db call")))
    }
}

#[post("/proof", data = "<data>")]
async fn submit_proof(_auth_token: AuthToken, data: SubmitProofRequest, config_data: &State<ConfigData>) -> AnyhowResult<Json<SubmitProofResponse>, CustomError>{
    let response: AnyhowResult<SubmitProofResponse>; 
    if data.proof_type == ProvingSchemes::GnarkGroth16 {
        response = submit_proof_exec::<GnarkGroth16Proof, GnarkGroth16Pis>(data, config_data).await;
    } else if data.proof_type == ProvingSchemes::Groth16 {
        response = submit_proof_exec::<SnarkJSGroth16Proof, SnarkJSGroth16Pis>(data, config_data).await;
    } else {
        info!("unspoorted proving scheme");
        return Err(CustomError::Internal(String::from("Unsupported Proving Scheme")))
    }
    match response {
        Ok(resp)  => Ok(Json(resp)),
        Err(e) => Err(CustomError::Internal(e.to_string()))
    }
}

#[get("/proof/<proof_id>")]
async fn get_proof_status(_auth_token: AuthToken, proof_id: String, config_data: &State<ConfigData>) -> AnyhowResult<Json<ProofDataResponse>, CustomError> {
    let response = get_proof_data_exec(proof_id, config_data).await;
    match response{
        Ok(r) => Ok(Json(r)),
        Err(e) => Err(CustomError::Internal(e.to_string()))
    }
}

#[post["/auth/protocol", data = "<data>"]]
async fn generate_auth_token(_auth_token: AuthToken, data: GenerateAuthTokenRequest) -> AnyhowResult<Json<GenerateAuthTokenResponse>, CustomError> {
    let response = generate_auth_token_for_protocol(data).await;
    match response{
        Ok(r) => Ok(Json(r)),
        Err(e) => Err(CustomError::Internal(e.to_string()))
    }
}

#[get["/protocol_proof/merkle/<proof_id>"]]
async fn get_protocol_proof(_auth_token: AuthToken, proof_id: String) -> AnyhowResult<Json<ProtocolProofResponse>, CustomError> {
    // Hash(reduction_circuit_hash||proof_id)
    // latest_verified_superproof --> Leaves Path

    let response = match get_protocol_proof_exec(&proof_id).await {
        Ok(r) => Ok(Json(r)),
        Err(e) => {
            let erro_string  = format!("some error occured in getting protocol proof : {:?}", e);
            Err(CustomError::Internal(erro_string))
        },
    };

    return response
}

#[launch]
async fn rocket() -> _ {
    dotenv().ok();

    let cors = rocket_cors::CorsOptions {
        ..Default::default()
    }.to_cors().unwrap();

    let _guard = initialize_logger("qunatum_node_api.log");
    let config_data = ConfigData::new("./config.yaml");
    let _db_initialize = get_pool().await;
    
    let t = rocket::Config::figment();
    rocket::custom(t).manage(config_data).mount("/", routes![index, ping, register_circuit, get_circuit_reduction_status, submit_proof, get_proof_status, generate_auth_token, get_protocol_proof]).attach(cors)
}