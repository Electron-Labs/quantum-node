use quantum_db::repository::user_circuit_data_repository::{get_user_circuit_data_by_circuit_hash, insert_user_circuit_data};
use quantum_types::{enums::circuit_reduction_status::CircuitReductionStatus, traits::vkey::Vkey, types::{config::ConfigData, db::protocol::Protocol}};
use quantum_utils::{keccak::encode_keccak_hash, paths::get_user_vk_path};
use rocket::State;

use anyhow::{anyhow, Result as AnyhowResult};
use tracing::error;
use quantum_db::repository::bonsai_image::get_bonsai_image_by_proving_scheme;
use crate::{connection::get_pool, error::error::CustomError, types::{circuit_registration_status::CircuitRegistrationStatusResponse, register_circuit::{RegisterCircuitRequest, RegisterCircuitResponse}}};


pub async fn register_circuit_exec<T: Vkey>(data: &RegisterCircuitRequest, config_data: &State<ConfigData>, protocol: &Protocol) -> AnyhowResult<RegisterCircuitResponse> {
    // Retreive verification key bytes
    let mut vkey_bytes = data.vkey.as_slice();

    // Borsh deserialise to corresponding vkey struct
    let vkey: T = T::deserialize_vkey(&mut vkey_bytes)
        .map_err(|e| anyhow!(CustomError::Internal(format!("Failed to deserialize vk: {}", e))))?;
    
    // Validate the vkey
    vkey.validate()
        .map_err(|e| {
            error!("vk is not valid");
            anyhow!(CustomError::Internal(format!("vk is invalid: {}", e)))
        })?;

    // Circuit Hash(str(Hash(vkey_bytes))) used to identify circuit
    let bonsai_image = get_bonsai_image_by_proving_scheme(get_pool().await, data.proof_type).await?;
    let circuit_hash = vkey.compute_circuit_hash(bonsai_image.circuit_verifying_id)
        .map_err(|e| anyhow!(CustomError::Internal(format!("Failed to compute circuit hash: {}", e))))?;
    let circuit_hash_string = encode_keccak_hash(&circuit_hash)
        .map_err(|e| anyhow!(CustomError::Internal(format!("Failed to encode circuit hash: {}", e))))?;
    println!("circuit_hash_string {:?}", circuit_hash_string);

    // Check if circuit is already registered
    if check_if_circuit_has_already_registered(circuit_hash_string.as_str()).await {
        error!("circuit has already been registered");
        return Ok(RegisterCircuitResponse { circuit_hash: circuit_hash_string });
    }
    println!("already registered check done");


    // dump vkey
    let vkey_path = get_user_vk_path(&config_data.storage_folder_path, &config_data.user_data_path, &circuit_hash_string);
    println!("User vkey path {:?}", vkey_path);
    vkey.dump_vk(&vkey_path).map_err(|e| anyhow!(CustomError::Internal(format!("Failed to dump vkey: {}", e))))?;
    println!("User vkey path dumped");

    // Add user circuit data to DB
    insert_user_circuit_data(get_pool().await, &circuit_hash_string, &vkey_path, data.proof_type, &protocol.protocol_name, &bonsai_image.image_id, CircuitReductionStatus::Completed).await?;
    println!("insert_user_circuit_data DONE");

    Ok(
        RegisterCircuitResponse{ circuit_hash: circuit_hash_string }
    )
}

pub async fn get_circuit_registration_status(circuit_hash: &str) -> AnyhowResult<CircuitRegistrationStatusResponse> {
    let user_circuit = get_user_circuit_data_by_circuit_hash(get_pool().await, circuit_hash).await?;
    let status = user_circuit.circuit_reduction_status;
    let bonsai_image = get_bonsai_image_by_proving_scheme(get_pool().await, user_circuit.proving_scheme).await?;
    
    let circuit_verifying_id_bytes: Vec<u8> = bonsai_image.circuit_verifying_id.iter().flat_map(|b| b.to_le_bytes()).collect();

     // Ensure the byte slice is exactly 32 bytes
     if circuit_verifying_id_bytes.len() != 32 {
        return Err(anyhow!(CustomError::Internal(format!("Invalid circuit verifying ID length"))));
    }

    // Convert the vector into a fixed-size array
    let circuit_verifying_id_bytes_array: [u8; 32] = circuit_verifying_id_bytes
        .try_into()
        .map_err(|_| anyhow!(CustomError::Internal(format!("Failed to convert to fixed-size array"))))?;

    // Compute the reduction circuit hash
    let reduction_circuit_hash = encode_keccak_hash(&circuit_verifying_id_bytes_array)
        .map_err(|_| anyhow!(CustomError::Internal(format!("Failed to encode circuit hash"))))?;

    Ok(CircuitRegistrationStatusResponse {
        circuit_registration_status: status.to_string(),
        reduction_circuit_hash,
    })
}

async fn check_if_circuit_has_already_registered(circuit_hash_string: &str) -> bool {
    get_user_circuit_data_by_circuit_hash(get_pool().await, circuit_hash_string).await.is_ok()
}

