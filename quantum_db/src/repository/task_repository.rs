use quantum_types::{enums::{task_status::TaskStatus, task_type::TaskType}, types::db::task::Task};
use quantum_utils::error_line;
use sqlx::{mysql::MySqlRow, MySql, Pool, Execute};
use sqlx::Row;
use anyhow::{anyhow, Error, Result as AnyhowResult};
use tracing::info;

use crate::error::error::CustomError;
// use crate::connection::get_pool;

pub async fn create_circuit_reduction_task(pool: &Pool<MySql>,user_circuit_hash: &str, task_type: TaskType, task_status: TaskStatus) -> AnyhowResult<u64, Error>{
    let query  = sqlx::query("INSERT into task(user_circuit_hash, task_type, task_status) VALUES(?,?,?)")
                .bind(user_circuit_hash).bind(task_type.as_u8()).bind(task_status.as_u8());

    info!("{}", query.sql());
    info!("arguments: {}, {}, {}", user_circuit_hash, task_type.as_u8(), task_status.as_u8());

    let row_affected = match query.execute(pool).await {
        Ok(t) => Ok(t.rows_affected()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}



pub async fn get_unpicked_task(pool: &Pool<MySql>) -> Result<Option<Task>, Error> {
    // oldest_entry(task_status: TaskStatus::NotPicked)
    let query  = sqlx::query("SELECT * from task where task_status = ? order by id LIMIT 1")
                .bind(TaskStatus::NotPicked.as_u8());

    info!("{}", query.sql());
    let reduction_circuit = match query.fetch_optional(pool).await{
        Ok(t) => get_task_from_mysql_row(t),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    reduction_circuit
}

fn get_task_from_mysql_row(row: Option<MySqlRow>) -> AnyhowResult<Option<Task>> {
    let task = match row {
        Some(r) => {
            let task_type: u8 = r.try_get_unchecked("task_type")?;
            let task_status: u8 = r.try_get_unchecked("task_status")?;
            Some(Task{
                id: r.try_get_unchecked("id")?,
                user_circuit_hash: r.try_get_unchecked("user_circuit_hash")?,
                task_type: TaskType::from(task_type),
                proof_id: r.try_get_unchecked("proof_id")?,
                task_status: TaskStatus::from(task_status)
            })
        },
        None => None
    };
    Ok(task)
}

pub async fn update_task_status(pool: &Pool<MySql>, task_id: u64, task_status: TaskStatus) -> AnyhowResult<()> {
    let query  = sqlx::query("UPDATE task set task_status = ? where id = ?")
                .bind(task_status.as_u8()).bind(task_id);

    info!("{}", query.sql());
    info!("arguments: {}, {}", task_status.as_u8(), task_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn get_aggregation_waiting_tasks_num(pool: &Pool<MySql>) -> Result<u64, Error> {
    let query  = sqlx::query("SELECT Count(*) as reduced_proof_count from task where task_status = ? and task_type = ? ")
                .bind(TaskStatus::Completed.as_u8()).bind(TaskType::ProofGeneration.as_u8());

    info!("{}", query.sql());
    let reduction_circuit = match query.fetch_one(pool).await{
        Ok(t) =>{ 
            let id: u64 = t.try_get_unchecked("reduced_proof_count")?;
            Ok(id)
        }
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    reduction_circuit
} 

pub async fn create_proof_task(pool: &Pool<MySql>, user_circuit_hash: &str, task_type: TaskType, task_status: TaskStatus, proof_id: &str) -> AnyhowResult<u64, Error> { 
    let query  = sqlx::query("INSERT into task(user_circuit_hash, task_type, task_status, proof_id) VALUES(?,?,?,?)")
                .bind(user_circuit_hash).bind(task_type.as_u8()).bind(task_status.as_u8()).bind(proof_id);

    info!("{}", query.sql());
    info!("arguments: {}, {}, {}, {}", user_circuit_hash, task_type.as_u8(), task_status.as_u8(), proof_id);
    
    let row_affected = match query.execute(pool).await {
        Ok(t) => Ok(t.rows_affected()),
        Err(e) => {
            info!("error in db: {:?}", error_line!(e));
            Err(anyhow!(CustomError::DB(error_line!(e))))
        }
    };
    row_affected
}