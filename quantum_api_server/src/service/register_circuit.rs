use keccak_hash::keccak;
use borsh::BorshDeserialize;
use rocket::State;

use anyhow::Result as AnyhowResult;
use serde::Serialize;

use crate::{ config::ConfigData,
    enums::{circuit_reduction_status::CircuitReductionStatus, task_type::TaskType},
    repository::{reduction_circuit_repository::check_if_pis_len_compatible_reduction_circuit_exist, task_repository,
        user_circuit_data_repository::{get_user_circuit_data_by_circuit_hash, insert_user_circuit_data}},
    types::{ db::reduction_circuit::ReductionCircuit, proving_schemes::ProvingSchemes, 
        register_circuit::{RegisterCircuitRequest, RegisterCircuitResponse}}, 
    utils::file::{create_dir, dump_json_file}};

pub async fn register_circuit_exec<T: BorshDeserialize + Serialize>(data: RegisterCircuitRequest, config_data: &State<ConfigData>) -> AnyhowResult<RegisterCircuitResponse> {
    // Retreive verification key bytes
    let vkey_bytes: Vec<u8> = data.vkey.clone();

    // Borsh deserialise to corresponding vkey struct 
    let vkey: T = T::deserialize(&mut vkey_bytes.as_slice())?;

    // Circuit Hash(str(Hash(vkey_bytes))) used to identify circuit 
    let circuit_hash_string = get_circuit_hash_from_vkey_bytes(vkey_bytes);

    // Check if circuit is already registerd
    let is_circuit_already_registered = check_if_circuit_has_already_registered(circuit_hash_string.as_str()).await;
    if is_circuit_already_registered  {
        println!("circuit has alerady been registered");
        return Ok(
            RegisterCircuitResponse{circuit_hash: circuit_hash_string}
        );
    }

    // Dump vkey
    let vk_path = format!("{}/{}", config_data.user_circuit_vk_path, circuit_hash_string );
    let vk_key_full_path = format!("{}/vk.json", vk_path.as_str() );
    dump_vkey(vkey, vk_path.as_str())?;

    // Get a reduction circuit id
    let reduction_circuit_id = handle_reduce_circuit(data.num_public_inputs, data.proof_type).await?;

    // Add user circuit data to DB
    insert_user_circuit_data(&circuit_hash_string, &vk_key_full_path, reduction_circuit_id, data.num_public_inputs, data.proof_type ).await?;

    // Create a reduction task for Async worker to pick up later on
    create_circuit_reduction_task(reduction_circuit_id, &circuit_hash_string).await?;
    Ok(
        RegisterCircuitResponse{ circuit_hash: circuit_hash_string }
    )
}

async fn handle_reduce_circuit(num_public_inputs: u8, proving_scheme: ProvingSchemes) -> AnyhowResult<Option<u64>>{
    let reduction_circuit = get_existing_compatible_reduction_circuit(num_public_inputs, proving_scheme).await;
    let reduction_circuit_id = match reduction_circuit {
        Some(rc) => rc.id,
        None => None
    };
    println!("reduction circuit id: {:?}", reduction_circuit_id );
    Ok(reduction_circuit_id)
}

async fn create_circuit_reduction_task(reduction_circuit_id: Option<u64>, circuit_hash: &str) -> AnyhowResult<()> {
    if reduction_circuit_id.is_none() {
        task_repository::create_circuit_reduction_task(circuit_hash, TaskType::CircuitReduction , CircuitReductionStatus::NotPicked).await?;
    }
    Ok(())
}

async fn get_existing_compatible_reduction_circuit(num_public_inputs: u8, proving_scheme: ProvingSchemes) -> Option<ReductionCircuit> {
    let mut reduction_circuit = None;
    if proving_scheme == ProvingSchemes::Groth16 || proving_scheme == ProvingSchemes::GnarkGroth16 {
        reduction_circuit =  check_if_pis_len_compatible_reduction_circuit_exist(num_public_inputs).await;
    }
    reduction_circuit
}

fn dump_vkey<T: Serialize>(vkey: T, vk_path: &str) -> AnyhowResult<()> {
    create_dir(vk_path)?;
    dump_json_file(vk_path, "vk.json", vkey)?;
    Ok(())
}

async fn check_if_circuit_has_already_registered(circuit_hash_string: &str) -> bool {
    let circuit_data = get_user_circuit_data_by_circuit_hash(circuit_hash_string).await;
    let is_circuit_already_registered = match circuit_data {
        Ok(_) => true,
        Err(_) => false
    };
    is_circuit_already_registered
}

fn get_circuit_hash_from_vkey_bytes(vkey_bytes: Vec<u8>) -> String {
    let mut keccak_ip = vkey_bytes.as_slice();
    let hash = keccak(&mut keccak_ip);
    let circuit_hash_string = format!("{:?}", hash);
    circuit_hash_string
}