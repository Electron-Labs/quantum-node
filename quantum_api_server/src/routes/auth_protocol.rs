use anyhow::Result as AnyhowResult;
use rocket::post;
use rocket::serde::json::Json;
use tracing::error;
use crate::{error::error::CustomError, service::protocol::generate_auth_token_for_protocol, types::{auth::AuthToken, generate_auth_token::{GenerateAuthTokenRequest, GenerateAuthTokenResponse}}};

#[post["/auth/protocol", data = "<data>"]]
pub async fn generate_auth_token(data: GenerateAuthTokenRequest) -> AnyhowResult<Json<GenerateAuthTokenResponse>, CustomError> {
    let response = generate_auth_token_for_protocol(data).await;
    match response{
        Ok(r) => Ok(Json(r)),
        Err(e) => {
            error!("Error in auth/protocol: {:?}", e);
            Err(CustomError::Internal(e.root_cause().to_string()))
        }
    }
}