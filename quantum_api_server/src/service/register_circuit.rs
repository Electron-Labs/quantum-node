use quantum_db::repository::{reduction_circuit_repository::{check_if_n_inner_commitments_compatible_reduction_circuit_id_exist, get_reduction_circuit_for_user_circuit}, task_repository, user_circuit_data_repository::{get_user_circuit_data_by_circuit_hash, insert_user_circuit_data}};
use quantum_types::{enums::{circuit_reduction_status::CircuitReductionStatus, proving_schemes::ProvingSchemes, task_status::TaskStatus, task_type::TaskType}, traits::{pis::Pis, vkey::Vkey}, types::{config::ConfigData, db::{protocol::Protocol, reduction_circuit::ReductionCircuit}, gnark_groth16::GnarkGroth16Vkey, halo2_plonk::{Halo2PlonkPis, Halo2PlonkVkey}}};
use quantum_utils::{keccak::encode_keccak_hash, paths::get_user_vk_path};
use rocket::State;

use anyhow::{anyhow, Result as AnyhowResult};
use tracing::info;
use quantum_db::repository::bonsai_image::get_bonsai_image_by_proving_scheme;
use crate::{connection::get_pool, error::error::CustomError, types::{circuit_registration_status::CircuitRegistrationStatusResponse, register_circuit::{RegisterCircuitRequest, RegisterCircuitResponse}}};


// TODO: handle the circuit registration on smart contract. Need to have a some kind of status
pub async fn register_circuit_exec<T: Vkey>(data: RegisterCircuitRequest, config_data: &State<ConfigData>, protocol: Protocol) -> AnyhowResult<RegisterCircuitResponse> {
    // Retreive verification key bytes
    let vkey_bytes: Vec<u8> = data.vkey.clone();

    // Borsh deserialise to corresponding vkey struct
    let vkey: T = T::deserialize_vkey(&mut vkey_bytes.as_slice())?;
    let _ = match vkey.validate(data.num_public_inputs) {
        Ok(_) => Ok(()),
        Err(e) => {
            info!("vk is not valid");
            Err(anyhow!(CustomError::Internal(format!("vk is invalid. {}",e))))
        },
    }?;
    println!("validated");
    // Circuit Hash(str(Hash(vkey_bytes))) used to identify circuit

    let bonsai_image = get_bonsai_image_by_proving_scheme(get_pool().await, data.proof_type).await?;

    // TODO: need to change this
    let circuit_hash = vkey.compute_circuit_hash(bonsai_image.circuit_verifying_id)?;
    let circuit_hash_string = encode_keccak_hash(&circuit_hash)?;
    println!("circuit_hash_string {:?}", circuit_hash_string);

    // Check if circuit is already registered
    let is_circuit_already_registered = check_if_circuit_has_already_registered(circuit_hash_string.as_str()).await;
    if is_circuit_already_registered  {
        info!("circuit has already been registered");
        return Ok(
            RegisterCircuitResponse{circuit_hash: circuit_hash_string}
        );
    }
    println!("already registered {:?}", is_circuit_already_registered);


    // dump vkey
    let vkey_path = get_user_vk_path(&config_data.storage_folder_path, &config_data.user_data_path, &circuit_hash_string);
    println!("User vkey path {:?}", vkey_path);
    vkey.dump_vk(&vkey_path)?;
    println!("User vkey path dumped");

    // Add user circuit data to DB
    insert_user_circuit_data(get_pool().await, &circuit_hash_string, &vkey_path, data.proof_type, &protocol.protocol_name, &bonsai_image.image_id, CircuitReductionStatus::SmartContractRgistrationPending).await?;
    println!("insert_user_circuit_data DONE");

    Ok(
        RegisterCircuitResponse{ circuit_hash: circuit_hash_string }
    )
}

// TODO: should this api be removed? or should I maintain it for backward compatibility with sdk
pub async fn get_circuit_registration_status(circuit_hash: String) -> AnyhowResult<CircuitRegistrationStatusResponse> {
    let user_circuit = get_user_circuit_data_by_circuit_hash(get_pool().await, circuit_hash.as_str()).await?;
    let status = user_circuit.circuit_reduction_status;
    return Ok(CircuitRegistrationStatusResponse {
        circuit_registration_status: status.to_string(),
    })
}

async fn check_if_circuit_has_already_registered(circuit_hash_string: &str) -> bool {
    let circuit_data = get_user_circuit_data_by_circuit_hash(get_pool().await, circuit_hash_string).await;
    let is_circuit_already_registered = match circuit_data {
        Ok(_) => true,
        Err(_) => false
    };
    is_circuit_already_registered
}

