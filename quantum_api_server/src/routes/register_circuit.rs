use anyhow::Result as AnyhowResult;
use quantum_db::repository::protocol::get_protocol_by_auth_token;
use quantum_types::{enums::proving_schemes::ProvingSchemes, types::{config::ConfigData, gnark_groth16::GnarkGroth16Vkey, gnark_plonk::GnarkPlonkVkey, halo2_plonk::Halo2PlonkVkey, snarkjs_groth16::SnarkJSGroth16Vkey}};
use quantum_utils::error_line;
use rocket::post;
use rocket::serde::json::Json;
use rocket::State;
use tracing::{error, info};

use crate::{connection::get_pool, error::error::CustomError, service::register_circuit::register_circuit_exec, types::{auth::AuthToken, register_circuit::{RegisterCircuitRequest, RegisterCircuitResponse}}};

#[post("/register_circuit", data = "<data>")]
pub async fn register_circuit(auth_token: AuthToken, data: RegisterCircuitRequest, config_data: &State<ConfigData>) -> AnyhowResult<Json<RegisterCircuitResponse>, CustomError> {
    let response: AnyhowResult<RegisterCircuitResponse>;
    let protocol = match get_protocol_by_auth_token(get_pool().await, &auth_token.0).await {
        Ok(p) => Ok(p),
        Err(e) => {
            error!("error in db while fetching protocol");
            Err(CustomError::Internal(error_line!(format!("error in db while fetching protocol. Error: {}", e))))
        },
    };
    let protocol = protocol?;
    info!("{:?}", protocol);
    let protocol = match protocol {
        Some(p) => Ok(p),
        None => {
            error!("No protocol against this auth token");
            Err(CustomError::Internal(error_line!("/register_circuit No protocol against this auth token".to_string())))
        },
    };
    let protocol = protocol?;

    info!("{:?}", protocol);

    if data.proof_type == ProvingSchemes::GnarkGroth16 {
        response = register_circuit_exec::<GnarkGroth16Vkey>(data, config_data, protocol).await;
    } else if data.proof_type == ProvingSchemes::Groth16 {
        response = register_circuit_exec::<SnarkJSGroth16Vkey>(data, config_data, protocol).await;
    } else if data.proof_type == ProvingSchemes::Halo2Plonk {
        response = register_circuit_exec::<Halo2PlonkVkey>(data, config_data, protocol).await;
    } else if data.proof_type == ProvingSchemes::GnarkPlonk {
        response = register_circuit_exec::<GnarkPlonkVkey>(data, config_data, protocol).await;
    } else {
        return Err(CustomError::Internal(String::from("Unsupported Proving Scheme")))
    }
    match response {
        Ok(resp)  => Ok(Json(resp)),
        Err(e) => {
            error!("Error in /register_circuit: {:?}", e);
            Err(CustomError::Internal(e.root_cause().to_string()))
        }
    }
}