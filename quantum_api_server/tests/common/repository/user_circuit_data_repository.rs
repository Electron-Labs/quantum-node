use anyhow::{anyhow, Result as AnyhowResult};
use quantum_db::error::error::CustomError;
use quantum_types::types::db::reduction_circuit;
use quantum_utils::error_line;
use sqlx::{Execute, MySql, Pool};
use tracing::info;

pub async fn delete_all_user_circuit_data(pool: &Pool<MySql>) -> AnyhowResult<()> {
    let query  = sqlx::query("DELETE from user_circuit_data");

    info!("{}", query.sql());

    let row_affected = match query.execute(&mut *pool.acquire().await.unwrap()).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn insert_random_protocol_user_circuit_data(pool: &Pool<MySql>, circuit_hash: &str) -> AnyhowResult<()> {
    let query = sqlx::query("INSERT INTO user_circuit_data VALUES (?,?,?,?,?,?,?)")
                                            .bind(circuit_hash).bind(format!("./storage/{}/user_data/vkey.bin", circuit_hash))
                                            .bind("NULL").bind(2).bind("RANDOM PROTOCOL").bind(1).bind("electron");
    info!("{}", query.sql());

    let row_affected = match query.execute(&mut *pool.acquire().await.unwrap()).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_circuit_redn_status_user_circuit_data_completed(pool: &Pool<MySql>, circuit_hash: &str) -> AnyhowResult<()> {
    
    let query = sqlx::query("UPDATE user_circuit_data SET circuit_reduction_status  = 3 WHERE circuit_hash = ?").bind(circuit_hash);

    info!("{}", query.sql());

    let row_affected = match query.execute(&mut *pool.acquire().await.unwrap()).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_reduction_circuit_id_user_circuit_data_completed(pool: &Pool<MySql>, circuit_hash: &str, reduction_circuit_id: &str) -> AnyhowResult<()> {
    
    let query = sqlx::query("UPDATE user_circuit_data SET reduction_circuit_id = ? WHERE circuit_hash = ?").bind(reduction_circuit_id).bind(circuit_hash);

    info!("{}", query.sql());

    let row_affected = match query.execute(&mut *pool.acquire().await.unwrap()).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}