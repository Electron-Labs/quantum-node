use anyhow::{anyhow, Result as AnyhowResult};
use quantum_utils::error_line;
use sqlx::{Execute, MySql, Pool};
use tracing::info;

pub async fn insert_cost_saved_data(pool: &Pool<MySql>, total_gas_saved: u64, total_usd_saved: f64) -> AnyhowResult<()>{
    let query = sqlx::query("INSERT INTO cost_saved(total_gas_saved, total_usd_saved) VALUES (?,?)")
                                             .bind(total_gas_saved).bind(total_usd_saved);
    info!("{}", query.sql());
    info!("arguments: {}, {}", total_gas_saved, total_usd_saved);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(error_line!(e)))
    };
    row_affected
}