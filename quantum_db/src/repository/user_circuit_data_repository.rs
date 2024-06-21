use std::str::FromStr;

use quantum_types::enums::circuit_reduction_status::CircuitReductionStatus;
use quantum_types::enums::proving_schemes::ProvingSchemes;
use quantum_utils::error_line;
use quantum_types::types::db::user_circuit_data::UserCircuitData;
use sqlx::mysql::MySqlRow;
use sqlx::{Error, MySql, Pool, Row, Execute};
use tracing::info;

// use crate::connection::get_pool;
use crate::error::error::CustomError;
use anyhow::{anyhow, Result as AnyhowResult, Error as AnyhowError};

pub async fn get_user_circuit_data_by_circuit_hash(pool: &Pool<MySql>, circuit_hash: &str) -> AnyhowResult<UserCircuitData>{
    let query  = sqlx::query("SELECT * from user_circuit_data where circuit_hash = ?")
                .bind(circuit_hash);

    info!("{}", query.sql());
    info!("arguments: {}", circuit_hash);
    
    let user_circuit_data = match query.fetch_one(pool).await{
        Ok(t) => get_user_circuit_data_from_mysql_row(t),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    user_circuit_data
}

pub async fn insert_user_circuit_data(pool: &Pool<MySql>, circuit_hash: &str, vk_path: &str, reduction_circuit_id: Option<String>, 
    pis_len: u8, proving_scheme: ProvingSchemes, circuit_reduction_status: CircuitReductionStatus, protocol_name: &str) -> AnyhowResult<u64, AnyhowError>{
    let query  = sqlx::query("INSERT into user_circuit_data(circuit_hash, vk_path, reduction_circuit_id, pis_len, proving_scheme, circuit_reduction_status, protocol_name) VALUES(?,?,?,?,?,?,?)")
                .bind(circuit_hash).bind(vk_path).bind(reduction_circuit_id.clone()).bind(pis_len).bind(proving_scheme.to_string())
                .bind(circuit_reduction_status.as_u8()).bind(protocol_name);

    info!("{}", query.sql());
    info!("arguments: {}, {}, {:?}, {}, {}, {}, {}", circuit_hash, vk_path, reduction_circuit_id, pis_len, proving_scheme.to_string(), circuit_reduction_status.as_u8(), protocol_name);
    
    let row_affected = match query.execute(pool).await {
        Ok(t) => Ok(t.rows_affected()),
        //start from here by printing   
        Err(e) => {
            println!("insert user error: {:?}", e);
            Err(anyhow!(error_line!(e)))
        }
    };
    row_affected
}


fn get_user_circuit_data_from_mysql_row(row: MySqlRow) -> AnyhowResult<UserCircuitData, AnyhowError>{
    let proving_scheme = match ProvingSchemes::from_str(row.try_get_unchecked("proving_scheme").map_err(|err| anyhow!(error_line!(err)))?) {
        Ok(ps) => Ok(ps),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    let circuit_status_as_u8: u8 = row.try_get_unchecked("circuit_reduction_status")?;
    let circuit_status =  CircuitReductionStatus::from(circuit_status_as_u8);
    let user_circuit_data = UserCircuitData {
        circuit_hash : row.try_get_unchecked("circuit_hash")?,
        vk_path: row.try_get_unchecked("vk_path")?,
        reduction_circuit_id: row.try_get_unchecked("reduction_circuit_id")?,
        pis_len: row.try_get_unchecked("pis_len")?,
        proving_scheme: proving_scheme?,
        circuit_reduction_status: circuit_status,
        protocol_name: row.try_get_unchecked("protocol_name")?
    };
    Ok(user_circuit_data)
}

pub async fn update_user_circuit_data_reduction_status(pool: &Pool<MySql>, user_circuit_hash: &str, status: CircuitReductionStatus) -> AnyhowResult<()> {
    let query  = sqlx::query("UPDATE user_circuit_data set circuit_reduction_status = ? where circuit_hash = ?")
                .bind(status.as_u8()).bind(user_circuit_hash);

    info!("{}", query.sql());
    info!("arguments: {}, {}", status.as_u8(), user_circuit_hash);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_user_circuit_data_redn_circuit(pool: &Pool<MySql>, user_circuit_hash: &str, reduction_circuit_id: &str) -> AnyhowResult<()> {
    let query  = sqlx::query("UPDATE user_circuit_data set reduction_circuit_id = ? where circuit_hash = ?")
                .bind(reduction_circuit_id).bind(user_circuit_hash);

    info!("{}", query.sql());
    info!("arguments: {}, {}", reduction_circuit_id, user_circuit_hash);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}