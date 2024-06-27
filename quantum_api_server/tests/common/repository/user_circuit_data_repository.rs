use anyhow::{anyhow, Result as AnyhowResult};
use quantum_db::error::error::CustomError;
use quantum_utils::error_line;
use sqlx::{Execute, MySql, Pool};
use tracing::info;

pub async fn delete_all_user_circuit_data_redn_circuit(pool: &Pool<MySql>) -> AnyhowResult<()> {
    let query  = sqlx::query("DELETE from user_circuit_data");

    info!("{}", query.sql());

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}