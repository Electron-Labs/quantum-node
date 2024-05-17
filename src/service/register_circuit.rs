use keccak_hash::keccak;
use borsh::BorshDeserialize;
use rocket::State;

use anyhow::Result as AnyhowResult;

use crate::{ config::ConfigData,
    enums::{circuit_reduction_status::CircuitReductionStatus, task_type::TaskType},
    repository::{reduction_circuit_repository::check_if_pis_len_compatible_reduction_circuit_exist, 
        task_repository::create_circuit_reduction_task, 
        user_circuit_data_repository::{get_user_circuit_data_by_circuit_hash, insert_user_circuit_data}},
    types::{ db::reduction_circuit::ReductionCircuit, gnark_groth16::GnarkGroth16Vkey, proving_schemes::ProvingSchemes, 
        register_circuit::{RegisterCircuitRequest, RegisterCircuitResponse}}, 
    utils::file::{create_dir, dump_json_file}};

pub async fn register_circuit_exec(data: RegisterCircuitRequest, config_data: &State<ConfigData>) -> AnyhowResult<RegisterCircuitResponse> {
    // 1. Dump whatever you need in db/queue 
    // 2. Just return the keccak hash of the data.vkey at this point
    // Keccak hash borshified vkey for now
    
    let vkey_bytes: Vec<u8> = data.vkey.clone();
    let gnark_vkey = GnarkGroth16Vkey::deserialize(&mut vkey_bytes.as_slice())?;
    let circuit_hash_string = get_circuit_hash_from_vkey_bytes(vkey_bytes);
    let is_circuit_already_registered = check_if_circuit_has_already_registered(circuit_hash_string.as_str()).await;
    
    if is_circuit_already_registered  {
        println!("circuit has alerady been registered");
        return Ok(
            RegisterCircuitResponse{circuit_hash: circuit_hash_string}
        );
    }

    let vk_path = format!("{}/{}", config_data.user_circuit_vk_path, circuit_hash_string );
    let vk_key_full_path = format!("{}/vk.json", vk_path.as_str() );
    dump_vkey(gnark_vkey, vk_path.as_str())?;

    let reduction_circuit_id = handle_reduce_circuit(&circuit_hash_string, data.num_public_inputs, data.proof_type).await?;
    insert_user_circuit_data(&circuit_hash_string, &vk_key_full_path, reduction_circuit_id, data.num_public_inputs, data.proof_type ).await?;

    // 3. An async worker actually takes care of the registration request
    Ok(
        RegisterCircuitResponse{ circuit_hash: circuit_hash_string }
    )
}

async fn handle_reduce_circuit(circuit_hash: &str, num_public_inputs: u8, proving_scheme: ProvingSchemes) -> AnyhowResult<Option<u64>>{
    let reduction_circuit = get_existing_compatible_reduction_circuit(num_public_inputs, proving_scheme).await;
    let reduction_circuit_id = match reduction_circuit {
        Some(rc) => rc.id,
        None => None
    };
    println!("reduction circuit id: {:?}", reduction_circuit_id );
    if reduction_circuit_id.is_none() {
        create_circuit_reduction_task(&circuit_hash, TaskType::CircuitReduction , CircuitReductionStatus::NotPicked).await?;
    }
    Ok(reduction_circuit_id)
}

async fn get_existing_compatible_reduction_circuit(num_public_inputs: u8, proving_scheme: ProvingSchemes) -> Option<ReductionCircuit> {
    let mut reduction_circuit = None;
    if proving_scheme == ProvingSchemes::Groth16 || proving_scheme == ProvingSchemes::GnarkGroth16 {
        reduction_circuit =  check_if_pis_len_compatible_reduction_circuit_exist(num_public_inputs).await;
    }
    reduction_circuit
}

fn dump_vkey(gnark_vkey: GnarkGroth16Vkey, vk_path: &str) -> AnyhowResult<()> {
    create_dir(vk_path)?;
    dump_json_file(vk_path, "vk.json", gnark_vkey)?;
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