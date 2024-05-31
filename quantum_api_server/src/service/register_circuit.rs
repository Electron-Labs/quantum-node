use quantum_db::repository::{reduction_circuit_repository::check_if_pis_len_compatible_reduction_circuit_exist, task_repository, user_circuit_data_repository::{get_user_circuit_data_by_circuit_hash, insert_user_circuit_data}};
use quantum_types::{enums::{circuit_reduction_status::CircuitReductionStatus, proving_schemes::ProvingSchemes, task_status::TaskStatus, task_type::TaskType}, traits::vkey::Vkey, types::{config::ConfigData, db::{protocol::Protocol, reduction_circuit::ReductionCircuit}}};
use rocket::State;

use anyhow::{anyhow, Result as AnyhowResult};
use tracing::info;

use crate::{connection::get_pool, error::error::CustomError, types::{circuit_registration_status::CircuitRegistrationStatusResponse, register_circuit::{RegisterCircuitRequest, RegisterCircuitResponse}}};

pub async fn register_circuit_exec<T: Vkey>(data: RegisterCircuitRequest, config_data: &State<ConfigData>, protocol: Protocol) -> AnyhowResult<RegisterCircuitResponse> {
    // Retreive verification key bytes
    let vkey_bytes: Vec<u8> = data.vkey.clone();

    // Borsh deserialise to corresponding vkey struct 
    let vkey: T = T::deserialize(&mut vkey_bytes.as_slice())?;
    let _ = match vkey.validate(data.num_public_inputs) {
        Ok(_) => Ok(()),
        Err(_) => {
            info!("vk is not valid");
            Err(anyhow!(CustomError::Internal("vk is invalid".to_string())))
        },
    }?;
    // Circuit Hash(str(Hash(vkey_bytes))) used to identify circuit 
    let circuit_hash_string = String::from_utf8(vkey.keccak_hash()?.to_vec())?;

    // Check if circuit is already registerd
    let is_circuit_already_registered = check_if_circuit_has_already_registered(circuit_hash_string.as_str()).await;
    if is_circuit_already_registered  {
        info!("circuit has alerady been registered");
        return Ok(
            RegisterCircuitResponse{circuit_hash: circuit_hash_string}
        );
    }

    // dump vkey
    let vkey_path = vkey.dump_vk(&circuit_hash_string, &config_data)?;

    // Get a reduction circuit id
    let reduction_circuit_id = handle_reduce_circuit(data.num_public_inputs, data.proof_type).await?;

    // Add user circuit data to DB
    insert_user_circuit_data(get_pool().await, &circuit_hash_string, &vkey_path, reduction_circuit_id.clone(), data.num_public_inputs, data.proof_type,CircuitReductionStatus::NotPicked, &protocol.protocol_name).await?;

    // Create a reduction task for Async worker to pick up later on
    create_circuit_reduction_task(reduction_circuit_id, &circuit_hash_string).await?;
    Ok(
        RegisterCircuitResponse{ circuit_hash: circuit_hash_string }
    )
}

pub async fn get_circuit_registration_status(circuit_hash: String) -> AnyhowResult<CircuitRegistrationStatusResponse> {
    let user_circuit = get_user_circuit_data_by_circuit_hash(get_pool().await, circuit_hash.as_str()).await?;
    let status = user_circuit.circuit_reduction_status;
    return Ok(CircuitRegistrationStatusResponse {
        circuit_registration_status: status.to_string()
    })
}

async fn handle_reduce_circuit(num_public_inputs: u8, proving_scheme: ProvingSchemes) -> AnyhowResult<Option<String>>{
    let reduction_circuit = get_existing_compatible_reduction_circuit(num_public_inputs, proving_scheme).await;
    let reduction_circuit_id = match reduction_circuit {
        Some(rc) => Some(rc.circuit_id),
        None => None
    };
    info!("reduction circuit id: {:?}", reduction_circuit_id );
    Ok(reduction_circuit_id)
}

async fn create_circuit_reduction_task(reduction_circuit_id: Option<String>, circuit_hash: &str) -> AnyhowResult<()> {
    if reduction_circuit_id.is_none() {
        task_repository::create_circuit_reduction_task(get_pool().await, circuit_hash, TaskType::CircuitReduction , TaskStatus::NotPicked).await?;
    }
    Ok(())
}

async fn get_existing_compatible_reduction_circuit(num_public_inputs: u8, proving_scheme: ProvingSchemes) -> Option<ReductionCircuit> {
    let mut reduction_circuit = None;
    if proving_scheme == ProvingSchemes::Groth16 || proving_scheme == ProvingSchemes::GnarkGroth16 {
        reduction_circuit =  check_if_pis_len_compatible_reduction_circuit_exist(get_pool().await, num_public_inputs, proving_scheme).await;
    }
    reduction_circuit
}



async fn check_if_circuit_has_already_registered(circuit_hash_string: &str) -> bool {
    let circuit_data = get_user_circuit_data_by_circuit_hash(get_pool().await, circuit_hash_string).await;
    let is_circuit_already_registered = match circuit_data {
        Ok(_) => true,
        Err(_) => false
    };
    is_circuit_already_registered
}