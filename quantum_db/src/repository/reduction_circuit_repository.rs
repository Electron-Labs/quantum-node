use quantum_types::types::db::{proof::Proof, reduction_circuit::ReductionCircuit};
use sqlx::{mysql::MySqlRow, Error, MySql, Pool, Row};

use anyhow::{anyhow, Result as AnyhowResult};

use crate::error::error::CustomError;

pub async fn get_reduction_circuit_by_pis_len(pool: &Pool<MySql>, num_public_inputs: u8) -> AnyhowResult<ReductionCircuit>{
    let query  = sqlx::query("SELECT * from reduction_circuit where pis_len = ?")
                .bind(num_public_inputs);

    // info!("{}", query.sql());
    let reduction_circuit = match query.fetch_one(pool).await{
        Ok(t) => get_reduction_circuit_data_from_mysql_row(t),
        Err(e) => Err(anyhow!(CustomError::DB(e.to_string())))
    };
    reduction_circuit
}

pub async fn check_if_pis_len_compatible_reduction_circuit_exist(pool: &Pool<MySql>, num_public_inputs: u8) -> Option<ReductionCircuit>{
    let rc = get_reduction_circuit_by_pis_len(pool, num_public_inputs).await;
    match rc {
        Ok(rc) => Some(rc),
        Err(_) => None
    }
}

fn get_reduction_circuit_data_from_mysql_row(row: MySqlRow) -> AnyhowResult<ReductionCircuit>{
    let reduction_circuit = ReductionCircuit {
        circuit_id: row.try_get_unchecked("circuit_id")?,
        proving_key_path: row.try_get_unchecked("proving_key_path")?,
        vk_path: row.try_get_unchecked("vk_path")?,
        pis_len: row.try_get_unchecked("pis_len")?,
    };
    Ok(reduction_circuit)
}

// Sending ReductionCircuit type with reduction_circuit.id = None, return id
pub async fn add_reduction_circuit_row(pool: &Pool<MySql>, reduction_circuit: ReductionCircuit) -> AnyhowResult<u64, Error> {
    let query  = sqlx::query("INSERT into reduction_circuit(circuit_id, proving_key_path, vk_path, pis_len) VALUES(?,?,?,?)")
                .bind(reduction_circuit.circuit_id).bind(reduction_circuit.proving_key_path).bind(reduction_circuit.vk_path).bind(reduction_circuit.pis_len);

    // info!("{}", query.sql());
    let row_affected = match query.execute(pool).await {
        Ok(t) => Ok(t.rows_affected()),
        Err(e) => Err(e)
    };
    row_affected
}

// get ReductionCircuit data from reduction_circuit_id
pub async fn get_reduction_circuit_data_by_id(pool: &Pool<MySql>, id: &str) -> AnyhowResult<ReductionCircuit> {
    let query  = sqlx::query("SELECT * from reduction_circuit where circuit_id = ?")
                .bind(id);

    // info!("{}", query.sql());
    let reduction_circuit = match query.fetch_one(pool).await{
        Ok(t) => get_reduction_circuit_data_from_mysql_row(t),
        Err(e) => Err(anyhow!(CustomError::DB(e.to_string())))
    };
    reduction_circuit
}