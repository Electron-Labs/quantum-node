use std::str::FromStr;

use sqlx::mysql::MySqlRow;
use sqlx::{Error, Row};

use crate::connection::get_pool;
use crate::error::error::CustomError;
use crate::types::db::user_circuit_data::UserCircuitData;
use crate::types::proving_schemes::{self, ProvingSchemes};
use anyhow::{anyhow, Result as AnyhowResult};

pub async fn get_user_circuit_data_by_circuit_hash(circuit_hash: &str) -> AnyhowResult<UserCircuitData>{
    let pool = get_pool().await;
    let query  = sqlx::query("SELECT * from user_circuit_data where circuit_hash = ?")
                .bind(circuit_hash);

    // info!("{}", query.sql());
    let user_circuit_data = match query.fetch_one(pool).await{
        Ok(t) => get_user_circuit_data_from_mysql_row(t),
        Err(e) => Err(anyhow!(CustomError::Internal(e.to_string())))
    };
    user_circuit_data
}

pub async fn insert_user_circuit_data(circuit_hash: &str, vk_path: &str, reduction_circuit_id: Option<u64>, pis_len: u8, proving_scheme: ProvingSchemes) -> AnyhowResult<u64, Error>{
    let pool = get_pool().await;
    let query  = sqlx::query("INSERT into user_circuit_data(circuit_hash, vk_path, reduction_circuit_id, pis_len, proving_scheme) VALUES(?,?,?,?,?)")
                .bind(circuit_hash).bind(vk_path).bind(reduction_circuit_id).bind(pis_len).bind(proving_scheme.to_string());

    // info!("{}", query.sql());
    let row_affected = match query.execute(pool).await {
        Ok(t) => Ok(t.rows_affected()),
        Err(e) => Err(e)
    };
    row_affected
}


fn get_user_circuit_data_from_mysql_row(row: MySqlRow) -> AnyhowResult<UserCircuitData>{
    let proving_scheme = match ProvingSchemes::from_str(row.try_get_unchecked("proving_scheme")?) {
        Ok(ps) => Ok(ps),
        Err(e) => Err(anyhow!(CustomError::Internal(e)))
    };
    let user_circuit_data = UserCircuitData {
        circuit_hash : row.try_get_unchecked("circuit_hash")?,
        vk_path: row.try_get_unchecked("vk_path")?,
        reduction_circuit_id: row.try_get_unchecked("reduction_circuit_id")?,
        pis_len: row.try_get_unchecked("pis_len")?,
        proving_scheme: proving_scheme?
    };
    Ok(user_circuit_data)
}