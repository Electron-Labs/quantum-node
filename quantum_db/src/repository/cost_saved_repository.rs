use anyhow::{anyhow, Result as AnyhowResult};
use quantum_utils::error_line;
use sqlx::{Any, Execute, Pool};
use tracing::info;

pub async fn udpate_cost_saved_data(pool: &Pool<Any>, total_gas_saved: u64, total_usd_saved: f64) -> AnyhowResult<()>{
    let query = sqlx::query("UPDATE cost_saved SET total_gas_saved = total_gas_saved + ?, total_usd_saved = total_usd_saved + ?")
                                             .bind(total_gas_saved.to_string()).bind(total_usd_saved.to_string());
    info!("{}", query.sql());
    info!("arguments: {}, {}", total_gas_saved, total_usd_saved.to_string());

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(error_line!(e)))
    };
    row_affected
}