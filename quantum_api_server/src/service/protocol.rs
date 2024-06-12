use quantum_db::repository::protocol::{check_if_protocol_already_registered, insert_protocol_auth_token};
use quantum_utils::keccak::get_keccak_hash_of_string;

use crate::{connection::get_pool, error::error::CustomError, types::generate_auth_token::{GenerateAuthTokenRequest, GenerateAuthTokenResponse}};

use anyhow::{anyhow, Context, Result as AnyhowResult};

pub async fn generate_auth_token_for_protocol(data: GenerateAuthTokenRequest) -> AnyhowResult<GenerateAuthTokenResponse> {
    let is_present = check_if_protocol_already_registered(get_pool().await, &data.protocol_name).await.with_context(|| format!("Cannot verify if protocol is already registered in file: {} on line: {}", file!(), line!()));
    let is_present = match is_present {
        Ok(t) => Ok(t) ,
        Err(_) => Err(anyhow!(CustomError::Internal("some internal error".to_string()))),
    };

    let is_present = is_present?;
    if is_present {
        println!("protocol has already been registered");
        return Err(anyhow!(CustomError::Internal("protocol has already been registered".to_string()))).with_context(|| format!("protocol has already been registered in file: {} on line: {}", file!(), line!()));
    }
    
    let protocol_name_hash = get_keccak_hash_of_string(&data.protocol_name);
    let token = get_token_from_hash(protocol_name_hash);

    insert_protocol_auth_token(get_pool().await, &data.protocol_name, &token).await.with_context(|| format!("Cannot insert protocol in db in file: {} on line: {}", file!(), line!()))?;
    Ok(GenerateAuthTokenResponse {
        auth_token: token,
    })
}

fn get_token_from_hash(hash: [u8; 32]) -> String {
    let bytes = &hash[..24];
    let hex_string = hex::encode(&bytes);
    hex_string
}