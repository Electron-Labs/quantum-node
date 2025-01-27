use quantum_db::repository::protocol::{check_if_protocol_already_registered, insert_protocol_auth_token};
use quantum_utils::error_line;
use quantum_utils::keccak::get_keccak_hash_of_string;
use tracing::error;

use crate::{connection::get_pool, error::error::CustomError, types::generate_auth_token::{GenerateAuthTokenRequest, GenerateAuthTokenResponse}};

use anyhow::{anyhow, Result as AnyhowResult};

pub async fn generate_auth_token_for_protocol(data: GenerateAuthTokenRequest) -> AnyhowResult<GenerateAuthTokenResponse> {
    let protocol_name = data.protocol_name.to_uppercase();
    let is_present = check_if_protocol_already_registered(get_pool().await, &protocol_name).await;
    let is_present = match is_present {
        Ok(t) => Ok(t) ,
        Err(e) => Err(anyhow!(CustomError::Internal(e.root_cause().to_string()))),
    };

    let is_present = is_present?;
    if is_present {
        error!("protocol has already been registered");
        return Err(anyhow!(CustomError::Internal(error_line!("protocol has already been registered".to_string()))));
    }

    let protocol_name_with_secret = std::env::var("auth_token_secret").expect("auth_token_secret must be set.") + &protocol_name;
    let protocol_name_hash = get_keccak_hash_of_string(&protocol_name_with_secret);
    let token = get_token_from_hash(protocol_name_hash);

    insert_protocol_auth_token(get_pool().await, &protocol_name, &token).await?;
    Ok(GenerateAuthTokenResponse {
        auth_token: token,
    })
}

fn get_token_from_hash(hash: [u8; 32]) -> String {
    let bytes = &hash[..24];
    let hex_string = hex::encode(&bytes);
    hex_string
}