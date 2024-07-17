use anyhow::{anyhow, Result as AnyhowResult};
use quantum_db::repository::{proof_repository::get_proof_by_proof_hash, user_circuit_data_repository::get_user_circuit_data_by_circuit_hash};
use quantum_types::{enums::{proof_status::ProofStatus, proving_schemes::ProvingSchemes}, types::{gnark_groth16::GnarkGroth16Pis, halo2_plonk::Halo2PlonkPis, snarkjs_groth16::SnarkJSGroth16Pis}};
use quantum_utils::error_line;
use rocket::{get, serde::json::Json};
use tracing::error;
use crate::{connection::get_pool, error::error::CustomError, service::proof::get_protocol_proof_exec, types::{auth::AuthToken, protocol_proof:: ProtocolProofResponse,}};

#[get["/protocol_proof/merkle/<proof_id>"]]
pub async fn get_protocol_proof(_auth_token: AuthToken, proof_id: String) -> AnyhowResult<Json<ProtocolProofResponse>, CustomError> {
    let response: AnyhowResult<ProtocolProofResponse, CustomError>;
    let pool = get_pool().await;
    let proof = get_proof_by_proof_hash(pool, &proof_id).await.map_err(|err| {
        CustomError::Internal(error_line!(format!("get_proof_by_proof_hash. Error: {}", err)))
    })?;
    if proof.proof_status != ProofStatus::Verified {
        return Err(CustomError::Internal(error_line!("proof is not verified".to_string())))
    }

    let user_circuit_hash = &proof.user_circuit_hash;
    let user_circuit_data = get_user_circuit_data_by_circuit_hash(pool, user_circuit_hash).await.map_err(|err| {
        CustomError::Internal(error_line!(format!("get_user_circuit_data_by_circuit_hash. Error: {}", err)))
    })?;

    match user_circuit_data.proving_scheme {
        ProvingSchemes::GnarkGroth16 => response = get_protocol_proof_exec::<GnarkGroth16Pis>(&proof).await,
        ProvingSchemes::Groth16 => response = get_protocol_proof_exec::<SnarkJSGroth16Pis>(&proof).await,
        ProvingSchemes::Halo2Plonk => response = get_protocol_proof_exec::<Halo2PlonkPis>(&proof).await,
        _ => return Err(CustomError::Internal(String::from("Unsupported Proving Scheme")))
    }

    match response {
        Ok(resp)  => Ok(Json(resp)),
        Err(err) => {
            match err {
                CustomError::NotFound(_) => {},
                _ => error!("Error in /protocol_proof/merkle/<proof_id>: {:?}", err)
            }
            Err(err)
        }
    }
}