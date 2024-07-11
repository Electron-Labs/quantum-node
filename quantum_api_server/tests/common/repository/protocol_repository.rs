use anyhow::{anyhow, Result as AnyhowResult};
use quantum_db::error::error::CustomError;
use quantum_utils::error_line;
use sqlx::{MySql, Pool, Execute};
use tracing::info;

pub async fn delete_protocol_from_protocol_name(pool: &Pool<MySql>, protocol_name: &str) -> AnyhowResult<()>{
    let query = sqlx::query("DELETE FROM protocol WHERE protocol_name=?").bind(protocol_name);

    info!("{}", query.sql());

    let row_affected = match query.execute(&mut *pool.acquire().await.unwrap()).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn insert_electron_protocol(pool: &Pool<MySql>) -> AnyhowResult<()>{
    let query = sqlx::query("INSERT INTO protocol (protocol_name, auth_token) VALUES ('electron', 'b3047d47c5d6551744680f5c3ba77de90acb84055eefdcbb')");

    info!("{}", query.sql());

    let row_affected = match query.execute(&mut *pool.acquire().await.unwrap()).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}