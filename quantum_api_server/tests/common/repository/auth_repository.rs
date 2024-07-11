use anyhow::{anyhow, Result as AnyhowResult};
use quantum_db::error::error::CustomError;
use quantum_utils::error_line;
use sqlx::{MySql, Pool, Execute};
use tracing::info;

pub async fn insert_auth_token_random(pool: &Pool<MySql>) -> AnyhowResult<()>{
    let query = sqlx::query("INSERT INTO auth (auth_token, is_master) VALUES('random', 1)");    

    info!("{}", query.sql());

    let row_affected = match query.execute(&mut *pool.acquire().await.unwrap()).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}