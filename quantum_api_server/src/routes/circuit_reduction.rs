use anyhow::Result as AnyhowResult;
use crate::{error::error::CustomError, service::register_circuit::get_circuit_registration_status, types::{auth::AuthToken, circuit_registration_status::CircuitRegistrationStatusResponse}};
use rocket::{get, serde::json::Json};
use tracing::error;

#[get("/circuit/<circuit_id>/status")]
pub async fn get_circuit_reduction_status(_auth_token: AuthToken, circuit_id: String) -> AnyhowResult<Json<CircuitRegistrationStatusResponse>, CustomError>{
    let status = get_circuit_registration_status(circuit_id).await;
    match status {
        Ok(s) => Ok(Json(s)),
        Err(e) => {
            error!("Error in /circuit/<circuit_id>/status: {:?}",e);
            Err(CustomError::Internal(e.root_cause().to_string()))
        }
    }
}