use quantum_types::enums::{circuit_reduction_status::CircuitReductionStatus, task_type::TaskType};
use sqlx::{Error, MySql, Pool};

// use crate::connection::get_pool;

pub async fn create_circuit_reduction_task(pool: &Pool<MySql>,user_circuit_hash: &str, task_type: TaskType, circuit_reduction_status: CircuitReductionStatus) -> Result<u64, Error>{
    let query  = sqlx::query("INSERT into task(user_circuit_hash, task_type, ciruit_reduction_status) VALUES(?,?,?)")
                .bind(user_circuit_hash).bind(task_type.as_u8()).bind(circuit_reduction_status.as_u8());

    // info!("{}", query.sql());
    let row_affected = match query.execute(pool).await {
        Ok(t) => Ok(t.rows_affected()),
        Err(e) => Err(e)
    };
    row_affected
}