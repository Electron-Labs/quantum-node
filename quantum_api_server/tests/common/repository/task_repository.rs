use anyhow::{anyhow, Result as AnyhowResult};
use quantum_db::error::error::CustomError;
use quantum_utils::error_line;
use sqlx::{mysql::MySqlRow, Execute, MySql, Pool};
use tracing::info;


pub async fn get_task_data_count_from_circuit_hash(pool: &Pool<MySql>, circuit_hash: &str) -> AnyhowResult<usize>{
    let query = sqlx::query("SELECT COUNT(*) FROM task WHERE user_circuit_hash = ?").bind(circuit_hash);

    info!("{}", query.sql());
    
    let result = match query.fetch_all(&mut *pool.acquire().await.unwrap()).await{
        Ok(res) => Ok(res.len()),
        Err(err) => Err(anyhow!(CustomError::DB(error_line!(err))))
    };
    result
}


pub async fn delete_all_task_data(pool: &Pool<MySql>) -> AnyhowResult<()>{
    let query  = sqlx::query("DELETE from task");

    info!("{}", query.sql());

    let row_affected = match query.execute(&mut *pool.acquire().await.unwrap()).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}