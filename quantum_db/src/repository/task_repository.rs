use quantum_types::{enums::{task_status::{self, TaskStatus}, task_type::TaskType}, types::db::task::Task};
use sqlx::{Error, MySql, Pool};
use anyhow::Result as AnyhowResult;
// use crate::connection::get_pool;

pub async fn create_circuit_reduction_task(pool: &Pool<MySql>,user_circuit_hash: &str, task_type: TaskType, task_status: TaskStatus) -> Result<u64, Error>{
    let query  = sqlx::query("INSERT into task(user_circuit_hash, task_type, task_status) VALUES(?,?,?)")
                .bind(user_circuit_hash).bind(task_type.as_u8()).bind(task_status.as_u8());

    // info!("{}", query.sql());
    let row_affected = match query.execute(pool).await {
        Ok(t) => Ok(t.rows_affected()),
        Err(e) => Err(e)
    };
    row_affected
}

pub async fn get_aggregation_waiting_tasks_num(pool: &Pool<MySql>) -> Result<u64, Error> {
    // Num(task_type: TaskType::ProofGeneration + task_status: TaskSatus::Completed) >= BatchNum
    todo!()
}

pub async fn get_unpicked_circuit_reduction_task(pool: &Pool<MySql>) -> Result<Option<Task>, Error> {
    // oldest_entry(task_status: TaskStatus::NotPicked)
    todo!()
}

pub async fn update_task_status(pool: &Pool<MySql>, task_id: u64, task_status: TaskStatus) -> AnyhowResult<()> {
    todo!()
}