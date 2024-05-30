use std::fs;

use quantum_db::repository::{reduction_circuit_repository::add_reduction_circuit_row, user_circuit_data_repository::{get_user_circuit_data_by_circuit_hash, update_user_circuit_data_redn_circuit, update_user_circuit_data_reduction_status}};
use quantum_types::{enums::{proving_schemes::ProvingSchemes, task_type::TaskType}, types::{config::ConfigData, db::{reduction_circuit::{self, ReductionCircuit}, task::Task}}};
use anyhow::{Ok, Result as AnyhowResult};
use sqlx::{MySql, Pool};
use quantum_circuits_ffi::circuit_builder::{BuildResult, CircomVK, GnarkVK};

use crate::utils::dump_reduction_circuit_data;

pub async fn handle_registration_task(pool: &Pool<MySql>, registration_task: Task, config: &ConfigData) -> AnyhowResult<()> {
    assert_eq!(registration_task.task_type, TaskType::CircuitReduction);
    let user_circuit_hash = registration_task.user_circuit_hash;
    
    // get user_circuit_data
    let user_circuit_data = get_user_circuit_data_by_circuit_hash(pool, &user_circuit_hash).await?;
    
    // Get vk_path
    let user_vk_path = user_circuit_data.vk_path;
    
    // Load User Vkey
    println!("Loading user vkey from path {:?}", user_vk_path);
    let user_vk_data = fs::read_to_string(user_vk_path)?;
    
    let build_result: BuildResult;

    // Call build_reduction_circuit from quantum_reduction_circuit
    println!("Calling gnark groth16 reduction circuit");
    if user_circuit_data.proving_scheme == ProvingSchemes::GnarkGroth16 {
        let gnark_vkey: GnarkVK = serde_json::from_str(&user_vk_data)?;
        build_result = gnark_vkey.build(user_circuit_data.pis_len as u8);
    } else if user_circuit_data.proving_scheme == ProvingSchemes::Groth16 {
        let snarkjs_vkey: CircomVK = serde_json::from_str(&user_vk_data)?;
        build_result = snarkjs_vkey.build();
    } else {
        return Ok(());
    }

    // Check if build was done successfully
    if !build_result.pass{
        return Err(anyhow::Error::msg(build_result.msg));
    }
    println!("Reduction circuit successfully built");
    // Dump reduction circuit proving key and verification key as raw bytes 
    let (circuit_id, pk_path, vk_path) = dump_reduction_circuit_data(config, &build_result.pk_raw_bytes, &build_result.vk_raw_bytes)?;

    println!("Dumped pk_bytes and vk_bytes for reduction circuit");
    // Add reduction circuit row (pk_path, vk_path, pis_len)
    let reduction_circuit = ReductionCircuit {
        circuit_id: circuit_id.clone(),
        proving_key_path: pk_path,
        vk_path: vk_path,
        pis_len: user_circuit_data.pis_len,
    };
    add_reduction_circuit_row(pool, reduction_circuit).await?;
    println!("Added reduction circuit data to DB");
    // Add reduction circuit id to user_circuit_data
    update_user_circuit_data_redn_circuit(pool, &user_circuit_hash, &circuit_id).await?;
    println!("Updated reduction_circuit_id to user circuit data");
    Ok(())
}


// #[cfg(test)]
// mod tests {
//     use std::fs;

//     use quantum_reduction_circuits_ffi::circuit_builder::GnarkVK;

//     #[test]
//     pub fn test_ffi() {
//         let json_data = fs::read_to_string("/Users/utsavjain/Desktop/electron_labs/quantum/quantum-node/dumps/gnark_vkey.json").expect("Failed to read file");
// 		let gnark_vkey: GnarkVK = serde_json::from_str(&json_data).expect("Failed to deserialize JSON data");
//         let x = gnark_vkey.build(1);
//     }
// }