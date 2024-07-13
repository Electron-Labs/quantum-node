use std::fs;

use quantum_db::repository::{reduction_circuit_repository::{add_reduction_circuit_row, check_if_n_inner_commitments_compatible_reduction_circuit_id_exist}, user_circuit_data_repository::{get_user_circuit_data_by_circuit_hash, update_user_circuit_data_redn_circuit, update_user_circuit_data_reduction_status}};
use quantum_types::{enums::{proving_schemes::ProvingSchemes, task_type::TaskType}, traits::{circuit_interactor::ReductionCircuitBuildResult, pis::Pis, vkey::Vkey}, types::{config::ConfigData, db::{reduction_circuit::{self, ReductionCircuit}, task::Task}, gnark_groth16::{GnarkGroth16Pis, GnarkGroth16Vkey}, halo2_plonk::Halo2PlonkVkey, snarkjs_groth16::SnarkJSGroth16Vkey}};
use anyhow::{anyhow, Ok, Result as AnyhowResult};
use quantum_utils::error_line;
use sqlx::{MySql, Pool};
use quantum_circuits_interface::ffi::interactor::QuantumV2CircuitInteractor;
use quantum_types::traits::circuit_interactor::CircuitInteractorFFI;
use tracing::{info, error};

use crate::{connection::get_pool, utils::dump_reduction_circuit_data};

pub async fn handle_registration_task(pool: &Pool<MySql>, registration_task: Task, config: &ConfigData) -> AnyhowResult<()> {
    assert_eq!(registration_task.task_type, TaskType::CircuitReduction);
    let user_circuit_hash = registration_task.user_circuit_hash;

    // get user_circuit_data
    let user_circuit_data = get_user_circuit_data_by_circuit_hash(pool, &user_circuit_hash).await?;

    // Get vk_path
    let user_vk_path = user_circuit_data.vk_path;

    // Load User Vkey
    let circuit_build_result: ReductionCircuitBuildResult;

    let mut reduction_circuit_id = None;
    // try to get some existing reduction circuit id
    match user_circuit_data.n_commitments {
        Some(some_n_commitments) => reduction_circuit_id = get_reduction_circuit_id(some_n_commitments).await?,
        _=> {}
    }

    if reduction_circuit_id.is_none() {
        // Build reduction circuit
        info!("Calling gnark groth16 reduction circuit");
        if user_circuit_data.proving_scheme == ProvingSchemes::GnarkGroth16 {
            let inner_circuit_gnark_vkey = GnarkGroth16Vkey::read_vk(&user_vk_path)?;
            info!("vkey :: {:?}", inner_circuit_gnark_vkey);
            circuit_build_result = QuantumV2CircuitInteractor::build_gnark_groth16_circuit(inner_circuit_gnark_vkey, user_circuit_data.n_pis as usize);
        } else if user_circuit_data.proving_scheme == ProvingSchemes::Groth16 {
            circuit_build_result = QuantumV2CircuitInteractor::build_snarkjs_groth16_circuit();
        } else if user_circuit_data.proving_scheme == ProvingSchemes::Halo2Plonk {
            let vkey = Halo2PlonkVkey::read_vk(&user_vk_path)?;
            circuit_build_result = QuantumV2CircuitInteractor::build_halo2_plonk_circuit(vkey);
        } else {
            error!("Unsupported Proving scheme");
            return Err(anyhow::Error::msg(error_line!("Proving scheme unsupported")));
        }

        // Check if circuit build was successful
        if !circuit_build_result.success {
            return Err(anyhow::Error::msg(error_line!(circuit_build_result.msg)));
        }
        info!("Reduction Circuit successfully built");

        // Dump reduction circuit proving key and verification key as raw bytes
        let (circuit_id, pk_path, vk_path) = dump_reduction_circuit_data(config, &circuit_build_result.proving_key_bytes, &circuit_build_result.verification_key)?;
        reduction_circuit_id = Some(circuit_id.clone());

        info!("Dumped pk_bytes and vk_bytes for reduction circuit");
        // Add reduction circuit row
        let reduction_circuit = ReductionCircuit {
            circuit_id: circuit_id.clone(),
            proving_key_path: pk_path,
            vk_path,
            n_inner_pis: user_circuit_data.n_pis,
            n_inner_commitments: user_circuit_data.n_commitments,
            proving_scheme: user_circuit_data.proving_scheme

        };
        add_reduction_circuit_row(pool, reduction_circuit).await?;
        info!("Added reduction circuit data to DB");
    }

    // Add reduction circuit id to user_circuit_data
    update_user_circuit_data_redn_circuit(pool, &user_circuit_hash, &reduction_circuit_id.ok_or(anyhow!(error_line!("missing reduction_circuit_id")))?).await?;
    info!("Updated reduction_circuit_id to user circuit data");
    Ok(())
}

async fn get_reduction_circuit_id(n_commitments: u8) -> AnyhowResult<Option<String>>{
    let reduction_circuit = check_if_n_inner_commitments_compatible_reduction_circuit_id_exist(get_pool().await, n_commitments).await;
    let reduction_circuit_id = match reduction_circuit {
        Some(rc) => Some(rc.circuit_id),
        None => None
    };
    info!("reduction circuit id: {:?}", reduction_circuit_id );
    Ok(reduction_circuit_id)
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