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
        Ok(t) => get_user_circuit_data_from_mysql_row(&t),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    user_circuit_data
}

pub async fn get_user_circuits_by_circuit_status(pool: &Pool<MySql>, status: CircuitReductionStatus) -> AnyhowResult<Vec<UserCircuitData>> {
    let query  = sqlx::query("SELECT * from user_circuit_data where circuit_reduction_status = ?")
    .bind(status.as_u8());

    info!("{}", query.sql());
    info!("arguments: {}", status.as_u8());

    let db_rows = match query.fetch_all(pool).await{
        Ok(rows) => Ok(rows),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e)))),
    }?;

    let mut user_circuits = vec![];
    for row in db_rows.iter() {
        let user_circuit = get_user_circuit_data_from_mysql_row(row)?;
        user_circuits.push(user_circuit);
    }

    Ok(user_circuits)
}

pub async fn insert_user_circuit_data(pool: &Pool<MySql>, circuit_hash: &str, vk_path: &str, proving_scheme: ProvingSchemes, protocol_name: &str, bonsai_image_id: &str, circuit_reduction_status: CircuitReductionStatus) -> AnyhowResult<u64, AnyhowError>{
    let query  = sqlx::query("INSERT into user_circuit_data(circuit_hash, vk_path, proving_scheme, protocol_name, bonsai_image_id, circuit_reduction_status) VALUES(?,?,?,?,?,?)")
                .bind(circuit_hash).bind(vk_path).bind(proving_scheme.to_string()).bind(protocol_name).bind(bonsai_image_id).bind(circuit_reduction_status.as_u8());

    info!("{}", query.sql());
    info!("arguments: {}, {}, {:?}, {}, {:?}, {}", circuit_hash, vk_path, proving_scheme.to_string(), protocol_name, bonsai_image_id, circuit_reduction_status.as_u8());

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


fn get_user_circuit_data_from_mysql_row(row: &MySqlRow) -> AnyhowResult<UserCircuitData, AnyhowError>{
    let proving_scheme = match ProvingSchemes::from_str(row.try_get_unchecked("proving_scheme").map_err(|err| anyhow!(error_line!(err)))?) {
        Ok(ps) => Ok(ps),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };

    let circuit_reduction_status_u8: u8 = row.try_get_unchecked("circuit_reduction_status").map_err(|err| anyhow!(error_line!(err)))?;
    let circuit_reduction_status =  CircuitReductionStatus::from(circuit_reduction_status_u8);

    let user_circuit_data = UserCircuitData {
        circuit_hash : row.try_get_unchecked("circuit_hash")?,
        vk_path: row.try_get_unchecked("vk_path")?,
        proving_scheme: proving_scheme?,
        bonsai_image_id: row.try_get_unchecked("bonsai_image_id")?,
        protocol_name: row.try_get_unchecked("protocol_name")?,
        circuit_reduction_status,
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