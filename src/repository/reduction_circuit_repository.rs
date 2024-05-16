use sqlx::mysql::MySqlRow;
use sqlx::{Execute, MySql, Row};

use crate::connection::get_pool;
use crate::types::reduction_circuit::ReductionCircuit;
// use crate::{connection::get_pool, types::reduction_circuit::ReductionCircuit};

pub async fn get_reduction_circuit_by_pis_len(num_public_inputs: u8) -> Option<ReductionCircuit>{
    let pool = get_pool().await;
    let query  = sqlx::query("SELECT * from reduction_circuit where pis_len = ?")
                .bind(num_public_inputs);

    // info!("{}", query.sql());
    let reduction_circuit = match query.fetch_optional(pool).await.unwrap() {
        Some(t) => Some(get_reduction_circuit_data_from_mysql_row(t)),
        None => None,
    };
    reduction_circuit
}

fn get_reduction_circuit_data_from_mysql_row(row: MySqlRow) -> ReductionCircuit{
    ReductionCircuit {
        id: row.try_get_unchecked("id").unwrap(),
        proving_key_path: row.try_get_unchecked("proving_key_path").unwrap(),
        vk_path: row.try_get_unchecked("vk_path").unwrap(),
        pis_len: row.try_get_unchecked("pis_len").unwrap(),
    }
}