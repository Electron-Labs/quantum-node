use quantum_db::repository::protocol::{check_if_protocol_already_registered, insert_protocol_auth_token};
use quantum_utils::error_line;
use quantum_utils::keccak::get_keccak_hash_of_string;
use tracing::error;

use crate::{connection::get_pool, error::error::CustomError, types::generate_auth_token::{GenerateAuthTokenRequest, GenerateAuthTokenResponse}};

use anyhow::{anyhow, Result as AnyhowResult};

pub async fn generate_auth_token_for_protocol(data: &GenerateAuthTokenRequest) -> AnyhowResult<GenerateAuthTokenResponse> {
    // All protocol names are processed in upper case 
    let protocol_name = data.protocol_name.to_uppercase();

    // Check if protocol is already registered
    if check_if_protocol_already_registered(get_pool().await, &protocol_name).await? {
        error!("protocol has already been registered");
        return Err(anyhow!(CustomError::Internal(
            error_line!("protocol has already been registered")
        )));
    };

    // Generate protocol name hash and token
    let protocol_name_hash = get_keccak_hash_of_string(&protocol_name);
    let token = get_token_from_hash(protocol_name_hash);

    // Insert the token into the database
    insert_protocol_auth_token(get_pool().await, &protocol_name, &token).await?;
    
    // Return the generated token
    Ok(GenerateAuthTokenResponse {
        auth_token: token,
    })
}

// Gets hex auth token from HashOut
fn get_token_from_hash(hash: [u8; 32]) -> String {
    hex::encode(&hash[..24])
}