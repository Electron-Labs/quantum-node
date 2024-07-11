use anyhow::Result as AnyhowResult;
use rocket::{get, serde::json::Json};
use tracing::error;
use crate::{error::error::CustomError, service::proof::get_protocol_proof_exec, types::{auth::AuthToken, protocol_proof:: ProtocolProofResponse}};

#[get["/protocol_proof/merkle/<proof_id>"]]
pub async fn get_protocol_proof(_auth_token: AuthToken, proof_id: String) -> AnyhowResult<Json<ProtocolProofResponse>, CustomError> {
    // Hash(reduction_circuit_hash||proof_id)
    // latest_verified_superproof --> Leaves Path

    let response = match get_protocol_proof_exec(&proof_id).await {
        Ok(r) => Ok(Json(r)),
        Err(e) => {
            error!("Error in /protocol_proof/merkle/<proof_id>: {:?}", e);
            Err(CustomError::Internal(e.root_cause().to_string()))
        },
    };

    return response
}