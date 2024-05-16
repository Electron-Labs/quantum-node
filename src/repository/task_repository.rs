use crate::{connection::get_pool, enums::{circuit_reduction_status::CircuitReductionStatus, task_type::TaskType}};

pub async fn create_circuit_reduction_task(user_circuit_hash: &str, task_type: TaskType, circuit_reduction_status: CircuitReductionStatus) {
    let pool = get_pool().await;
    let query  = sqlx::query("INSERT into task(user_circuit_hash, task_type, ciruit_reduction_status) VALUES(?,?,?)")
                .bind(user_circuit_hash).bind(task_type.as_u8()).bind(circuit_reduction_status.as_u8());

    // info!("{}", query.sql());
    let row_affected = query.execute(pool).await.unwrap();
}