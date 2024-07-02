use anyhow::{anyhow, Result as AnyhowResult};
use quantum_db::error::error::CustomError;
use quantum_utils::error_line;
use sqlx::{query::Query, Execute, MySql, Pool};
use tracing::info;

pub async fn insert_dummy_data_reduction_circuit(pool: &Pool<MySql>, circuit_hash: &str) -> AnyhowResult<()>{
    let query: Query<MySql,_> = sqlx::query("INSERT INTO reduction_circuit VALUES (?, 'proving/key/path', 'vk/path', 2 , 'Groth16')").bind(circuit_hash);

    info!("{}", query.sql());

    let row_affected = match query.execute(&mut *pool.acquire().await.unwrap()).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn delete_all_reduction_circuit_data(pool: &Pool<MySql>) -> AnyhowResult<()>{
    let query = sqlx::query("DELETE from reduction_circuit");

    info!("{}", query.sql());

    let row_affected = match query.execute(&mut *pool.acquire().await.unwrap()).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}