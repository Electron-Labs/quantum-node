use quantum_types::types::db::reduction_circuit::ReductionCircuit;
use sqlx::mysql::MySqlRow;
use sqlx::Row;

use crate::{connection::get_pool, error::error::CustomError};
// use crate::error::error::QuantumError;
use anyhow::{anyhow, Result as AnyhowResult};



pub async fn get_reduction_circuit_by_pis_len(num_public_inputs: u8) -> AnyhowResult<ReductionCircuit>{
    let pool = get_pool().await;
    let query  = sqlx::query("SELECT * from reduction_circuit where pis_len = ?")
                .bind(num_public_inputs);

    // info!("{}", query.sql());
    let reduction_circuit = match query.fetch_one(pool).await{
        Ok(t) => get_reduction_circuit_data_from_mysql_row(t),
        Err(e) => Err(anyhow!(CustomError::Internal(e.to_string())))
    };
    reduction_circuit
}

pub async fn check_if_pis_len_compatible_reduction_circuit_exist(num_public_inputs: u8) -> Option<ReductionCircuit>{
    let rc = get_reduction_circuit_by_pis_len(num_public_inputs).await;
    match rc {
        Ok(rc) => Some(rc),
        Err(_) => None
    }
}

fn get_reduction_circuit_data_from_mysql_row(row: MySqlRow) -> AnyhowResult<ReductionCircuit>{
    let reduction_circuit = ReductionCircuit {
        id: row.try_get_unchecked("id")?,
        proving_key_path: row.try_get_unchecked("proving_key_path")?,
        vk_path: row.try_get_unchecked("vk_path")?,
        pis_len: row.try_get_unchecked("pis_len")?,
    };
    Ok(reduction_circuit)
}