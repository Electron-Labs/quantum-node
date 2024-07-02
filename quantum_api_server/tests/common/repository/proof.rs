use anyhow::{anyhow, Result as AnyhowResult};
use quantum_db::error::error::CustomError;
use quantum_utils::error_line;
use sqlx::{pool, query, Execute, MySql, Pool};
use tracing::info;


pub async fn delete_all_proof_data(pool: &Pool<MySql>) -> AnyhowResult<()>{
    let query  = sqlx::query("DELETE FROM proof");

    info!("{}", query.sql());

    let rows_affected = match query.execute(&mut *pool.acquire().await.unwrap()).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    rows_affected
}

pub async fn update_proof_status_to_verified(pool: &Pool<MySql>, proof_hash: &str) -> AnyhowResult<()>{
    let query = sqlx::query("UPDATE proof SET proof_status=7 where proof_hash=?").bind(proof_hash);

    info!("{}", query.sql());

    let rows_affected = match query.execute(&mut *pool.acquire().await.unwrap()).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    rows_affected
}

pub async fn update_superproof_id(pool: &Pool<MySql>, proof_hash: &str, superproof_id: u32) -> AnyhowResult<()>{
    let query = sqlx::query("UPDATE proof SET superproof_id = ? WHERE proof_hash=?").bind(superproof_id).bind(proof_hash);

    info!("{}", query.sql());

    let rows_affected = match query.execute(&mut *pool.acquire().await.unwrap()).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    rows_affected
}